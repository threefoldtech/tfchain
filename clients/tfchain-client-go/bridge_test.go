package substrate

import (
	"math/big"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	StellarAccountAddress = "GCPVVC4MWKV7ZGQCMHHMALNWVLN2II43RC3FDLCRFCZJBAJCZHNE4VKK"
	StellarAccountSecret  = "SCAXZWWCWKOK4WVUVAYUNBADS7RAFTOKW4KTLMDVMEGEMWFU62G434IY"
)

func TestSwapToStellar(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	err = cl.SwapToStellar(identity, StellarAccountAddress, *big.NewInt(50000000))
	require.NoError(t, err)
}
