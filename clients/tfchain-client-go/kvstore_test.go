package substrate

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestKVStore(t *testing.T) {
	pairs := map[string]string{"key1": "value1", "key2": "value2", "key3": "value3"}

	// Alice identity
	id, err := NewIdentityFromSr25519Phrase("//Alice")
	require.NoError(t, err)

	sub := startLocalConnection(t)
	defer sub.Close()

	t.Run("kvstore set values", func(t *testing.T) {
		for key, val := range pairs {
			err = sub.KVStoreSet(id, key, val)
			assert.NoError(t, err)
		}
	})

	t.Run("kvstore get values", func(t *testing.T) {
		for key, val := range pairs {
			value, err := sub.KVStoreGet(id, key)
			assert.NoError(t, err)
			assert.Equal(t, []byte(val), value)
		}
	})

	t.Run("kvstore list keys", func(t *testing.T) {
		values, err := sub.KVStoreList(id)
		assert.NoError(t, err)
		assert.EqualValues(t, values, pairs)
	})

	t.Run("kvstore delete", func(t *testing.T) {
		for key := range pairs {
			err = sub.KVStoreDelete(id, key)
			assert.NoError(t, err)
		}

		values, err := sub.KVStoreList(id)
		assert.NoError(t, err)
		assert.Empty(t, values)
	})
}
