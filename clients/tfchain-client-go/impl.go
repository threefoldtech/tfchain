package substrate

import (
	"context"
	"fmt"
	"math/rand"
	"slices"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cenkalti/backoff"
	gsrpc "github.com/centrifuge/go-substrate-rpc-client/v4"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
)

var (
	ErrInvalidVersion         = fmt.Errorf("invalid version")
	ErrUnknownVersion         = fmt.Errorf("unknown version")
	ErrNotFound               = fmt.Errorf("object not found")
	ErrNoConnectionsAvailable = fmt.Errorf("no healthy connections available")
	ErrMaxPoolSizeReached     = fmt.Errorf("max pool size reached")
)

const (
	AcceptableDelay = 2 * 6 * time.Second
)

// Versioned base for all types
type Versioned struct {
	Version uint32 `json:"version"`
}

type Conn = *gsrpc.SubstrateAPI
type Meta = *types.Metadata

// Pool connection
type poolConn struct {
	conn     Conn
	meta     Meta
	url      string
	lastUsed atomic.Int64 // Unix timestamp
	inUse    atomic.Bool
}

func (pc *poolConn) isHealthy() bool {
	if pc == nil || pc.conn == nil || pc.meta == nil {
		return false
	}
	_, err := getTime(pc.conn, pc.meta)
	return err == nil
}

func (pc *poolConn) close() {
	if pc != nil && pc.conn != nil {
		pc.conn.Client.Close()
		pc.conn = nil
		pc.meta = nil
		log.Debug().Str("url", pc.url).Msg("closed connection")
	}
}

type Manager interface {
	GetConnection(ctx context.Context) (*Substrate, error)
	Close() error

	// Deprecated methods
	Raw() (Conn, Meta, error)
	Substrate() (*Substrate, error)
}

type ManagerConfig struct {
	// Maximum number of connections in the pool
	MaxPoolSize int
	// Minimum number of connections to maintain
	MinPoolSize int
	// Maximum time a connection can be idle before being closed (if the pool has more than MinPoolSize)
	MaxIdleTime time.Duration
	// Interval between health checks
	// After thinking about it, we don't need to periodically check the health of the connections
	// because this creates a lot of overhead
	// so instead we just check the health when we need to and do the maintanance in demand
	// HealthCheckInterval time.Duration
	// Timeout for creating new connections
	ConnectionTimeout time.Duration
}

// Default configuration
var DefaultConfig = ManagerConfig{
	MaxPoolSize: 5,
	MinPoolSize: 2,
	MaxIdleTime: 30 * time.Minute,
	// HealthCheckInterval: 120 * time.Second,
	ConnectionTimeout: 10 * time.Second,
}

type manager struct {
	urls      []string
	pool      []*poolConn
	mu        sync.RWMutex
	ctx       context.Context
	cancel    context.CancelFunc
	wg        sync.WaitGroup
	config    ManagerConfig
	checkChan chan struct{}
}

func NewManager(urls ...string) Manager {
	return NewManagerWithConfig(DefaultConfig, urls...)
}

func NewManagerWithConfig(config ManagerConfig, urls ...string) Manager {
	if len(urls) == 0 {
		panic("at least one URL required")
	}

	// Validate and adjust configuration
	if config.MaxPoolSize < 1 {
		config.MaxPoolSize = DefaultConfig.MaxPoolSize
	}
	if config.MinPoolSize < 1 || config.MinPoolSize > config.MaxPoolSize {
		config.MinPoolSize = min(DefaultConfig.MinPoolSize, config.MaxPoolSize)
	}
	if config.MaxIdleTime <= 0 {
		config.MaxIdleTime = DefaultConfig.MaxIdleTime
	}
	// if config.HealthCheckInterval <= 0 {
	// 	config.HealthCheckInterval = DefaultConfig.HealthCheckInterval
	// }
	if config.ConnectionTimeout <= 0 {
		config.ConnectionTimeout = DefaultConfig.ConnectionTimeout
	}

	ctx, cancel := context.WithCancel(context.Background())
	m := &manager{
		urls:      shuffle(urls),
		pool:      make([]*poolConn, 0, config.MaxPoolSize),
		ctx:       ctx,
		cancel:    cancel,
		config:    config,
		checkChan: make(chan struct{}, 1),
	}

	m.initializePool()
	m.wg.Add(1)
	go m.healthChecker()

	return m
}

func (m *manager) initializePool() {
	log.Debug().Msg("initializing connection pool")
	for i := 0; i < m.config.MinPoolSize; i++ {
		select {
		case m.checkChan <- struct{}{}:
		default:
		}
	}
}

func (m *manager) createConnection(ctx context.Context, url string) (*poolConn, error) {
	log.Debug().Str("url", url).Msg("attempting to create a new connection")
	ctx, cancel := context.WithTimeout(ctx, m.config.ConnectionTimeout)
	defer cancel()

	select {
	case <-ctx.Done():
		log.Error().Str("url", url).Msg("context done while creating connection")
		return nil, ctx.Err()
	default:
		if conn, meta, err := createSubstrateConn(url); err == nil {
			log.Debug().Str("url", url).Msg("created new connection")
			return &poolConn{
				conn:     conn,
				meta:     meta,
				url:      url,
				lastUsed: atomic.Int64{},
				inUse:    atomic.Bool{},
			}, nil
		} else {
			log.Error().Str("url", url).Err(err).Msg("failed to create connection")
		}
	}
	return nil, fmt.Errorf("failed to create connection to %s", url)
}

func (m *manager) GetConnection(ctx context.Context) (*Substrate, error) {
	log.Debug().Msg("getting a connection")
	conn, err := m.getHealthyConn()
	if err != nil {
		log.Error().Err(err).Msg("failed to get connection")
		return nil, fmt.Errorf("failed to get connection: %w", err)
	}
	log.Debug().Str("url", conn.url).Msg("successfully obtained connection")
	return newSubstrate(conn, m), nil
}

func (m *manager) getHealthyConn() (*poolConn, error) {
	log.Debug().Int("pool_size", len(m.pool)).Int("aquired_count", m.aquiredConnCount()).
		Msg("checking for healthy connections")

	// Try getting existing connection first
	if conn := m.getExistingConn(); conn != nil {
		return conn, nil
	}

	b := backoff.NewExponentialBackOff()
	b.MaxInterval = 2 * time.Second
	b.InitialInterval = 500 * time.Millisecond
	b.Multiplier = 2

	var conn *poolConn
	err := backoff.Retry(func() error {
		// Check if we can get an existing connection
		if c := m.getExistingConn(); c != nil {
			conn = c
			return nil
		}

		m.mu.RLock()
		poolSize := len(m.pool)
		m.mu.RUnlock()

		if poolSize >= m.config.MaxPoolSize {
			return backoff.Permanent(ErrMaxPoolSizeReached)
		}

		select {
		case m.checkChan <- struct{}{}:
			log.Debug().Msg("triggered connection check")
		default:
			log.Debug().Msg("connection check already pending")
		}

		// time.Sleep(50 * time.Millisecond)
		return ErrNoConnectionsAvailable
	}, b)

	if err != nil {
		return nil, err
	}

	return conn, nil
}

func (m *manager) healthChecker() {
	defer m.wg.Done()
	// ticker := time.NewTicker(m.config.HealthCheckInterval)
	// defer ticker.Stop()

	for {
		select {
		case <-m.ctx.Done():
			return
		// case <-ticker.C:
		// 	m.checkConnections()
		case <-m.checkChan:
			m.checkConnections()
		}
	}
}

func (m *manager) checkConnections() {
	m.mu.Lock()
	healthy := make([]*poolConn, 0, len(m.pool))
	for _, conn := range m.pool {
		if conn == nil {
			continue
		}

		if !conn.isHealthy() {
			log.Debug().Str("url", conn.url).Msg("closing unhealthy connection")
			conn.close()
			continue
		}

		// Check if connection is idle for too long if we have more than min pool size
		if !conn.inUse.Load() && time.Since(time.Unix(conn.lastUsed.Load(), 0)) > m.config.MaxIdleTime && len(m.pool) > m.config.MinPoolSize {
			log.Debug().Str("url", conn.url).Msg("closing idle connection")
			conn.close()
			continue
		}

		healthy = append(healthy, conn)
	}

	m.pool = healthy
	m.mu.Unlock()
	m.ensureMinConnections()

}

func (m *manager) ensureMinConnections() {
	log.Debug().Msg("ensuring minimum connections in the pool")
	inUseCount := m.aquiredConnCount()
	urls := shuffle(m.unusedURLs())
	urls = append(urls, m.urls...)

	for _, url := range urls {
		poolSize := len(m.pool)

		if poolSize < m.config.MinPoolSize || (poolSize < m.config.MaxPoolSize && poolSize == inUseCount) {
			if conn, err := m.createConnection(m.ctx, url); err == nil {
				m.mu.Lock()
				m.pool = append(m.pool, conn)
				m.mu.Unlock()
				log.Debug().Str("url", url).Msg("added new connection to pool")
			}
		} else {
			break
		}
	}
}

func (m *manager) Close() error {
	m.cancel()
	m.wg.Wait()

	m.mu.Lock()
	defer m.mu.Unlock()

	for _, conn := range m.pool {
		conn.close()
	}
	m.pool = nil
	return nil
}

// Helper methods
func (m *manager) unusedURLs() []string {
	m.mu.RLock()
	defer m.mu.RUnlock()

	// get all urls that are not in the pool
	used := make([]string, 0, len(m.pool))
	for _, conn := range m.pool {
		used = append(used, conn.url)
	}
	unused := make([]string, 0, len(m.urls))
	for _, url := range m.urls {
		if !slices.Contains(used, url) {
			unused = append(unused, url)
		}
	}
	return unused
}

func (m *manager) aquiredConnCount() int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	count := 0
	for _, conn := range m.pool {
		if conn.inUse.Load() {
			count++
		}
	}
	return count
}

func (m *manager) getExistingConn() *poolConn {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, conn := range m.pool {
		if conn.isHealthy() && !conn.inUse.Load() {
			if conn.inUse.CompareAndSwap(false, true) {
				conn.lastUsed.Store(time.Now().Unix())
				return conn
			}
		}
	}
	return nil
}

func shuffle(urls []string) []string {
	result := make([]string, len(urls))
	copy(result, urls)
	rand.Shuffle(len(result), func(i, j int) {
		result[i], result[j] = result[j], result[i]
	})
	return result
}

// Deprecated methods implementation
func (m *manager) Raw() (Conn, Meta, error) {
	conn, err := m.GetConnection(context.Background())
	if err != nil {
		return nil, nil, err
	}
	return conn.conn.conn, conn.conn.meta, nil
}

func (m *manager) Substrate() (*Substrate, error) {
	return m.GetConnection(context.Background())
}

type Substrate struct {
	conn   *poolConn
	mgr    *manager
	mu     sync.Mutex
	closed bool
}

func newSubstrate(conn *poolConn, mgr *manager) *Substrate {
	return &Substrate{
		conn: conn,
		mgr:  mgr,
	}
}

func createSubstrateConn(url string) (Conn, Meta, error) {
	cl, err := newSubstrateAPI(url)
	if err != nil {
		return nil, nil, err
	}

	meta, err := cl.RPC.State.GetMetadataLatest()
	if err != nil {
		cl.Client.Close()
		return nil, nil, err
	}

	t, err := getTime(cl, meta)
	if err != nil || time.Since(t) > AcceptableDelay {
		cl.Client.Close()
		return nil, nil, fmt.Errorf("node health check failed")
	}

	return cl, meta, nil
}

func (s *Substrate) GetClient() (Conn, Meta, error) {
	if s.closed {
		log.Error().Msg("attempted to get client from closed substrate")
		return nil, nil, fmt.Errorf("substrate connection closed")
	}

	if s.conn.isHealthy() {
		conn := s.conn.conn
		meta := s.conn.meta
		s.conn.lastUsed.Store(time.Now().Unix())
		return conn, meta, nil
	}
	s.conn.inUse.Store(false)

	conn, err := s.mgr.getHealthyConn()
	if err != nil {
		log.Error().Err(err).Msg("failed to get healthy connection for client")
		return nil, nil, err
	}

	s.mu.Lock()

	s.conn = conn
	s.mu.Unlock()

	log.Debug().Str("url", conn.url).Msg("swapped connection")
	return conn.conn, conn.meta, nil
}

func (s *Substrate) Release() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.closed {
		return
	}
	s.closed = true

	if s.conn != nil {
		s.conn.inUse.Store(false)
		log.Debug().Str("url", s.conn.url).Msg("releasing connection to pool")
		s.conn = nil
	}
}

func (s *Substrate) getVersion(b types.StorageDataRaw) (uint32, error) {
	var ver Versioned
	if err := Decode(b, &ver); err != nil {
		return 0, errors.Wrapf(ErrInvalidVersion, "failed to load version (reason: %s)", err)
	}

	return ver.Version, nil
}

func (s *Substrate) Time() (t time.Time, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return t, err
	}

	return getTime(cl, meta)
}

// deprecated methods
func (s *Substrate) Close() {
	s.Release()
}

func getTime(cl Conn, meta Meta) (t time.Time, err error) {
	key, err := types.CreateStorageKey(meta, "Timestamp", "Now", nil)
	if err != nil {
		return t, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return t, errors.Wrap(err, "failed to lookup entity")
	}

	var stamp types.Moment
	if err := Decode(*raw, &stamp); err != nil {
		return t, errors.Wrap(err, "failed to get node time")
	}

	return stamp.Time, nil
}
