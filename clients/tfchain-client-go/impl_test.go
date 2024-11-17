package substrate

import (
	"context"
	"fmt"
	"math/rand"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestPoolInitialization(t *testing.T) {
	urls := []string{"ws://127.0.0.1:9944"}
	mgr := NewManager(urls...)
	defer mgr.Close()

	time.Sleep(100 * time.Millisecond)

	mgrImpl := mgr.(*manager)
	mgrImpl.mu.RLock()
	defer mgrImpl.mu.RUnlock()

	assert.LessOrEqual(t, len(mgrImpl.pool), mgrImpl.config.MinPoolSize)
	assert.Greater(t, len(mgrImpl.pool), 0)
}

func TestConnectionReuse(t *testing.T) {
	mgr := NewManager("ws://127.0.0.1:9944")
	defer mgr.Close()

	// Wait for pool initialization
	time.Sleep(100 * time.Millisecond)

	// Get first connection
	sub1, err := mgr.GetConnection(context.Background())
	require.NoError(t, err)

	// Store connection details for comparison
	conn1 := sub1.conn
	url1 := sub1.conn.url

	// Release it back to pool properly
	sub1.Release()

	// Small delay to ensure connection is properly released
	time.Sleep(10 * time.Millisecond)

	// Get another connection
	sub2, err := mgr.GetConnection(context.Background())
	require.NoError(t, err)
	defer sub2.Release()

	// Should be the same underlying connection
	assert.Equal(t, conn1, sub2.conn)
	assert.Equal(t, url1, sub2.conn.url)
}

func TestConcurrentAccess(t *testing.T) {
	mgr := NewManager("ws://127.0.0.1:9944")
	defer mgr.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	var wg sync.WaitGroup

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()

			sub, err := mgr.GetConnection(ctx)
			if err != nil {
				return
			}
			defer sub.Release()
			_, err = sub.Time()
			assert.NoError(t, err)
			time.Sleep(10 * time.Millisecond)
		}()
	}

	wg.Wait()
}

func TestFailover(t *testing.T) {
	mgr := NewManager("ws://fail1", "ws://127.0.0.1:9944")
	defer mgr.Close()

	sub1, err := mgr.GetConnection(context.Background())
	require.NoError(t, err)
	defer sub1.Release()
	sub2, err := mgr.GetConnection(context.Background())
	require.NoError(t, err)
	defer sub2.Release()
	assert.Equal(t, sub1.conn.url, "ws://127.0.0.1:9944")
	assert.Equal(t, sub2.conn.url, "ws://127.0.0.1:9944")
}

func TestHealthChecking(t *testing.T) {
	mgr := NewManager("ws://127.0.0.1:9944")
	defer mgr.Close()

	sub, err := mgr.GetConnection(context.Background())
	require.NoError(t, err)
	defer sub.Release()

	// Simulate connection failure
	old := sub.conn.conn
	old.Client.Close()
	// simulate usage of the client
	_, err = sub.Time()
	assert.NoError(t, err)
	assert.NotEqual(t, old, sub.conn.conn)
}

func TestStressWithFailures(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping stress test in short mode")
	}

	// Use test-specific configuration
	config := ManagerConfig{
		MaxPoolSize: 30,
		MinPoolSize: 3,
		MaxIdleTime: time.Minute,
		// HealthCheckInterval: time.Second,
		ConnectionTimeout: time.Second,
	}

	mgr := NewManagerWithConfig(config, "ws://127.0.0.1:9944")
	defer mgr.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	var (
		wg     sync.WaitGroup
		mu     sync.Mutex
		errors []error
	)

	for i := 0; i < 30; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()

			retryBackoff := time.Millisecond * 100
			maxBackoff := time.Second

			for ctx.Err() == nil {
				sub, err := mgr.GetConnection(ctx)
				if err != nil {
					mu.Lock()
					errors = append(errors, fmt.Errorf("goroutine %d: %w", id, err))
					mu.Unlock()

					jitter := time.Duration(rand.Int63n(int64(retryBackoff)))
					time.Sleep(retryBackoff + jitter)
					retryBackoff *= 2
					if retryBackoff > maxBackoff {
						retryBackoff = maxBackoff
					}
					continue
				}

				// Reset backoff on success
				retryBackoff = time.Millisecond * 100

				// Simulate work
				_, err = sub.Time()
				assert.NoError(t, err)
				time.Sleep(time.Duration(rand.Intn(250)+50) * time.Millisecond)

				if id%2 == 0 && rand.Float32() < 0.1 {
					sub.conn.conn.Client.Close()
				}

				sub.Release()
			}
		}(i)
	}

	wg.Wait()

	// Log and check errors
	for _, err := range errors {
		t.Logf("Error: %v", err)
	}

	assert.Less(t, len(errors), 10,
		"Too many errors occurred during stress test: %d", len(errors))
}
