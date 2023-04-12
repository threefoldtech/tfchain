package substrate

import (
	"fmt"
	"math/rand"
	"sync"
	"time"

	"github.com/cenkalti/backoff"
	gsrpc "github.com/centrifuge/go-substrate-rpc-client/v4"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
)

const (
	// acceptable delay is amount of blocks (in second) that a node can
	// be behind before we don't accept it. block time is 6 seconds, so
	// right now we only allow 2 blocks delay
	acceptableDelay = 2 * 6 * time.Second
)

var (
	//ErrInvalidVersion is returned if version 4bytes is invalid
	ErrInvalidVersion = fmt.Errorf("invalid version")
	//ErrUnknownVersion is returned if version number is not supported
	ErrUnknownVersion = fmt.Errorf("unknown version")
	//ErrNotFound is returned if an object is not found
	ErrNotFound = fmt.Errorf("object not found")
)

// Versioned base for all types
type Versioned struct {
	Version uint32
}

type Conn = *gsrpc.SubstrateAPI
type Meta = *types.Metadata

type Manager interface {
	Raw() (Conn, Meta, error)
	Substrate() (*Substrate, error)
}

type mgrImpl struct {
	urls []string

	r int
	m sync.Mutex
}

func NewManager(url ...string) Manager {
	if len(url) == 0 {
		panic("at least one url is required")
	}

	// the shuffle is needed so if one endpoints fails, and the next one
	// is tried, we will end up moving all connections to the "next" endpoint
	// which will get overloaded. Instead the shuffle helps to make the "next"
	// different for reach instace of the pool.
	rand.Shuffle(len(url), func(i, j int) {
		url[i], url[j] = url[j], url[i]
	})

	return &mgrImpl{
		urls: url,
		r:    rand.Intn(len(url)), // start with random url, then roundrobin
	}
}

// endpoint return the next endpoint to use
// in roundrobin fashion. need to be called
// while lock is acquired.
func (p *mgrImpl) endpoint() string {
	defer func() {
		p.r = (p.r + 1) % len(p.urls)
	}()

	return p.urls[p.r]
}

// Substrate return a new wrapped substrate connection
// the connection must be closed after you are done using it
func (p *mgrImpl) Substrate() (*Substrate, error) {
	cl, meta, err := p.Raw()
	if err != nil {
		return nil, err
	}

	return newSubstrate(cl, meta, p.put)
}

// Raw returns a RPC substrate client. plus meta. The returned connection
// is not tracked by the pool, nor reusable. It's the caller responsibility
// to close the connection when done
func (p *mgrImpl) Raw() (Conn, Meta, error) {
	// right now this pool implementation just tests the connection
	// makes sure that it is still active, otherwise, tries again
	// until the connection is restored.
	// A better pool implementation can be done later were multiple connections
	// can be handled
	// TODO: thread safety!
	p.m.Lock()
	defer p.m.Unlock()

	boff := backoff.WithMaxRetries(
		backoff.NewConstantBackOff(200*time.Millisecond),
		2*uint64(len(p.urls)),
	)

	var (
		cl   *gsrpc.SubstrateAPI
		meta *types.Metadata
		err  error
	)

	err = backoff.RetryNotify(func() error {
		endpoint := p.endpoint()
		log.Debug().Str("url", endpoint).Msg("connecting")
		cl, err = newSubstrateAPI(endpoint)
		if err != nil {
			return errors.Wrapf(err, "error connecting to substrate at '%s'", endpoint)
		}

		meta, err = cl.RPC.State.GetMetadataLatest()
		if err != nil {
			return errors.Wrapf(err, "error getting latest metadata at '%s'", endpoint)
		}

		t, err := getTime(cl, meta)
		if err != nil {
			return errors.Wrapf(err, "error getting node time at '%s'", endpoint)
		}

		if time.Since(t) > acceptableDelay {
			return fmt.Errorf("node '%s' is behind acceptable delay with timestamp '%s'", endpoint, t)
		}

		return nil

	}, boff, func(err error, _ time.Duration) {
		log.Error().Err(err).Msg("failed to connect to endpoint, retrying")
	})

	return cl, meta, err
}

// TODO: implement reusable connections instead of
// closing the connection.
func (p *mgrImpl) put(cl *Substrate) {
	// naive put implementation for now
	// we just immediately kill the connection
	if cl.cl != nil {
		cl.cl.Client.Close()
	}
	cl.cl = nil
	cl.meta = nil
}

// Substrate client
type Substrate struct {
	cl   Conn
	meta Meta

	close func(s *Substrate)
}

// NewSubstrate creates a substrate client
func newSubstrate(cl Conn, meta Meta, close func(*Substrate)) (*Substrate, error) {
	return &Substrate{cl: cl, meta: meta, close: close}, nil
}

func (s *Substrate) Close() {
	s.close(s)
}

func (s *Substrate) GetClient() (Conn, Meta, error) {
	return s.cl, s.meta, nil
}

func (s *Substrate) getVersion(b types.StorageDataRaw) (uint32, error) {
	var ver Versioned
	if err := types.Decode(b, &ver); err != nil {
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
	if err := types.Decode(*raw, &stamp); err != nil {
		return t, errors.Wrap(err, "failed to get node time")
	}

	return stamp.Time, nil
}
