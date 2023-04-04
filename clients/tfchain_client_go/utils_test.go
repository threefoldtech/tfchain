package substrate

import (
	"crypto/md5"
	"encoding/hex"
	"errors"
	"os"
	"testing"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/require"
)

type AccountUser string

const (
	AccountAlice      = "alice"
	AccountAliceStash = "alice_stash"
	AccountBob        = "bob"
)

var (
	someDocumentUrl     = "somedocument"
	testName            = "test-substrate"
	AliceMnemonics      = "//Alice"
	AliceStashMnemonics = "//Alice//stash"
	BobMnemonics        = "//Bob"
	AliceAddress        = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
	AliceStashAddress   = "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY"
	BobAddress          = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"

	Accounts = map[AccountUser]struct {
		Phrase  string
		Address string
	}{
		AccountAlice: {
			Phrase:  AliceMnemonics,
			Address: AliceAddress,
		},
		AccountAliceStash: {
			Phrase:  AliceStashMnemonics,
			Address: AliceStashAddress,
		},
		AccountBob: {
			Phrase:  BobMnemonics,
			Address: BobAddress,
		},
	}
)

func startLocalConnection(t *testing.T) *Substrate {
	var mgr Manager
	if _, ok := os.LookupEnv("CI"); ok {
		mgr = NewManager("ws://127.0.0.1:9944")
	} else {
		mgr = NewManager("wss://tfchain.dev.grid.tf")
	}

	cl, err := mgr.Substrate()

	require.NoError(t, err)

	return cl
}

func assertCreateTwin(t *testing.T, cl *Substrate, user AccountUser) uint32 {
	u := Accounts[user]

	identity, err := NewIdentityFromSr25519Phrase(u.Phrase)
	require.NoError(t, err)

	account, err := FromAddress(u.Address)
	require.NoError(t, err)

	termsAndConditions, err := cl.SignedTermsAndConditions(account)
	require.NoError(t, err)

	if len(termsAndConditions) == 0 {
		hash := md5.New()
		hash.Write([]byte(someDocumentUrl))
		h := hex.EncodeToString(hash.Sum(nil))
		err = cl.AcceptTermsAndConditions(identity, someDocumentUrl, h)
		require.NoError(t, err)
	}

	twnID, err := cl.GetTwinByPubKey(account.PublicKey())

	if err != nil {
		address := "relay.io"
		pk := "pk"
		twnID, err = cl.CreateTwin(identity, address, []byte(pk))
		require.NoError(t, err)
	}

	return twnID
}

func assertCreateFarm(t *testing.T, cl *Substrate) (uint32, uint32) {

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	twnID := assertCreateTwin(t, cl, AccountBob)

	id, err := cl.GetFarmByName(testName)
	if err == nil {
		return id, twnID
	}

	if errors.Is(err, ErrNotFound) {
		err = cl.CreateFarm(identity, testName, []PublicIPInput{})
		require.NoError(t, err)
	}

	id, err = cl.GetFarmByName(testName)
	require.NoError(t, err)

	return id, twnID
}

func assertCreateNode(t *testing.T, cl *Substrate, farmID uint32, twinID uint32, identity Identity) uint32 {
	nodeID, err := cl.GetNodeByTwinID(twinID)
	if err == nil {
		return nodeID
	} else if !errors.Is(err, ErrNotFound) {
		require.NoError(t, err)
	}
	// if not found create a node.
	nodeID, err = cl.CreateNode(identity,
		Node{
			FarmID: types.U32(farmID),
			TwinID: types.U32(twinID),
			Location: Location{
				City:      "SomeCity",
				Country:   "SomeCountry",
				Latitude:  "51.049999",
				Longitude: "3.733333",
			},
			Resources: Resources{
				SRU: types.U64(1024 * Gigabyte),
				MRU: types.U64(16 * Gigabyte),
				CRU: types.U64(8),
				HRU: types.U64(1024 * Gigabyte),
			},
			BoardSerial: OptionBoardSerial{
				HasValue: true,
				AsValue:  "some_serial",
			},
		},
	)
	require.NoError(t, err)

	return nodeID
}
