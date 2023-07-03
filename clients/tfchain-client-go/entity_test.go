package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestEntity(t *testing.T) {
	var entityID uint32

	cl := startLocalConnection(t)
	defer cl.Close()

	entityID = assertCreateEntity(t, cl)

	entity, err := cl.GetEntity(entityID)

	require.NoError(t, err)
	require.Equal(t, testName, entity.Name)
}
