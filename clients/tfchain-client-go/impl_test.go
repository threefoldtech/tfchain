package substrate

import (
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFailoverMechanism(t *testing.T) {
	t.Run("should failover to next URL when current node is unhealthy", func(t *testing.T) {
		// Create manager with multiple URLs
		urls := []string{"ws://fail1", getUrlBasedOnEnv()}
		mgr := NewManager(urls...)

		// Get initial substrate client
		sub, err := mgr.Substrate()
		require.NoError(t, err)
		defer sub.Close()

		// Store initial Client
		initialClient := sub.cl.Client

		// Force connection to become unhealthy by closing it
		sub.cl.Client.Close()

		// Try to use the connection - should trigger failover
		_, err = sub.Time()
		require.NoError(t, err)

		// Check that we're now using a different URL
		newClient := sub.cl.Client
		assert.NotEqual(t, initialClient, newClient)
	})

	t.Run("should try all URLs in rotation", func(t *testing.T) {
		urls := []string{
			"ws://fail1",
			"ws://fail2",
			getUrlBasedOnEnv(),
		}

		mgr := NewManager(urls...)
		sub, err := mgr.Substrate()
		require.NoError(t, err)
		defer sub.Close()

		// The final URL should be the working one
		assert.Equal(t, getUrlBasedOnEnv(), sub.cl.Client.URL())
	})

	t.Run("should reuse connection if healthy", func(t *testing.T) {
		sub := startLocalConnection(t)
		defer sub.Close()

		initialClient := sub.cl.Client

		// Use the connection multiple times
		for i := 0; i < 3; i++ {
			_, err := sub.Time()
			require.NoError(t, err)
			assert.Equal(t, initialClient, sub.cl.Client)
		}
	})

	t.Run("should handle all nodes being down", func(t *testing.T) {
		urls := []string{"ws://fail1", "ws://fail2"}
		mgr := NewManager(urls...)
		_, err := mgr.Substrate()
		assert.Error(t, err)
	})

	t.Run("should handle concurrent failover attempts", func(t *testing.T) {
		urls := []string{getUrlBasedOnEnv(), getUrlBasedOnEnv()}
		mgr := NewManager(urls...)
		sub1, err := mgr.Substrate()
		require.NoError(t, err)
		defer sub1.Close()

		sub2, err := mgr.Substrate()
		require.NoError(t, err)
		defer sub2.Close()

		// Force both connections to fail
		sub1.cl.Client.Close()
		sub2.cl.Client.Close()

		// Create WaitGroup to ensure all goroutines complete before test ends
		var wg sync.WaitGroup
		wg.Add(2)

		// Try to use both connections concurrently
		errs := make(chan error, 2)
		go func() {
			defer wg.Done()
			_, err := sub1.Time()
			errs <- err
		}()

		go func() {
			defer wg.Done()
			_, err := sub2.Time()
			errs <- err
		}()

		// Wait for both operations to complete
		go func() {
			wg.Wait()
			close(errs)
		}()

		// Check errors from both goroutines
		for err := range errs {
			require.NoError(t, err)
		}
	})
}
