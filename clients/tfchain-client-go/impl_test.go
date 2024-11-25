package substrate

import (
	"testing"
	"time"

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

		// Try to use both connections concurrently
		done := make(chan bool)
		go func() {
			_, err := sub1.Time()
			assert.NoError(t, err)
			done <- true
		}()

		go func() {
			_, err := sub2.Time()
			assert.NoError(t, err)
			done <- true
		}()

		// Wait for both operations to complete
		for i := 0; i < 2; i++ {
			select {
			case <-done:
			case <-time.After(5 * time.Second):
				t.Fatal("timeout waiting for concurrent failover")
			}
		}
	})
}
