package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestPrice(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	price, err := cl.GetTFTPrice()
	require.Greater(t, int(price), 0)
	require.NoError(t, err)

	price, err = cl.GetAverageTFTPrice()
	require.Greater(t, int(price), 0)
	require.NoError(t, err)

	pricingPolicy, err := cl.GetPricingPolicy(1)
	require.Equal(t, pricingPolicy.Name, "threefold_default_pricing_policy")
	require.Equal(t, int(pricingPolicy.ID), 1)
	require.NoError(t, err)
}
