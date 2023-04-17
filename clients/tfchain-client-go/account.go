package substrate

import (
	"bytes"
	"crypto/ed25519"
	"encoding/json"
	"fmt"
	"math/big"
	"net/http"
	"time"

	"github.com/cenkalti/backoff"
	"github.com/centrifuge/go-substrate-rpc-client/v4/signature"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/jbenet/go-base58"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
	"github.com/vedhavyas/go-subkey"
	subkeyEd25519 "github.com/vedhavyas/go-subkey/ed25519"
	subkeySr25519 "github.com/vedhavyas/go-subkey/sr25519"
)

const (
	network = 42

	// TFT in uTFT
	TFT = 10_000_000 // uTFT
	// ReactivateThreshold is threshold to reactivate the account if funds went below that
	ReactivateThreshold = TFT * 0.001 // uTFT
)

var (
	reactivateAt = big.NewInt(ReactivateThreshold)
)

// AccountID type
type AccountID types.AccountID

// Balance
type Balance struct {
	Free       types.U128 `json:"free"`
	Reserved   types.U128 `json:"reserverd"`
	MiscFrozen types.U128 `json:"misc_frozen"`
	FreeFrozen types.U128 `json:"free_frozen"`
}

type AccountInfo struct {
	Nonce       types.U32 `json:"nonce"`
	Consumers   types.U32 `json:"consumers"`
	Providers   types.U32 `json:"providers"`
	Sufficients types.U32 `json:"sufficients"`
	Data        Balance   `json:"data"`
}

// PublicKey gets public key from account id
func (a AccountID) PublicKey() []byte {
	return a[:]
}

// String return string representation of account
func (a AccountID) String() string {
	address, _ := subkey.SS58Address(a[:], network)
	return address
}

// MarshalJSON implementation
func (a AccountID) MarshalJSON() ([]byte, error) {
	address, err := subkey.SS58Address(a[:], network)
	if err != nil {
		return nil, err
	}

	return json.Marshal(address)
}

// FromAddress creates an AccountID from a SS58 address
func FromAddress(address string) (account AccountID, err error) {
	bytes := base58.Decode(address)
	if len(bytes) != 3+len(account) {
		return account, fmt.Errorf("invalid address length")
	}
	if bytes[0] != network {
		return account, fmt.Errorf("invalid address format")
	}

	copy(account[:], bytes[1:len(account)+1])
	return
}

func FromKeyBytes(address []byte) (string, error) {
	return subkey.SS58Address(address, network)
}

// keyringPairFromSecret creates KeyPair based on seed/phrase and network
// Leave network empty for default behavior
func keyringPairFromSecret(seedOrPhrase string, network uint8, scheme subkey.Scheme) (signature.KeyringPair, error) {
	kyr, err := subkey.DeriveKeyPair(scheme, seedOrPhrase)

	if err != nil {
		return signature.KeyringPair{}, err
	}

	ss58Address, err := kyr.SS58Address(network)
	if err != nil {
		return signature.KeyringPair{}, err
	}

	var pk = kyr.Public()

	return signature.KeyringPair{
		URI:       seedOrPhrase,
		Address:   ss58Address,
		PublicKey: pk,
	}, nil
}

var (
	ErrAccountNotFound = fmt.Errorf("account not found")
)

/*
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"kycSignature": "", "data": {"name": "", "email": ""}, "substrateAccountID": "5DAprR72N6s7AWGwN7TzV9MyuyGk9ifrq8kVxoXG9EYWpic4"}' \
  https://api.substrate01.threefold.io/activate
*/

func (s *Substrate) activateAccount(identity Identity, activationURL string) error {
	var buf bytes.Buffer
	if err := json.NewEncoder(&buf).Encode(map[string]string{
		"substrateAccountID": identity.Address(),
	}); err != nil {
		return errors.Wrap(err, "failed to build required body")
	}

	response, err := http.Post(activationURL, "application/json", &buf)
	if err != nil {
		return errors.Wrap(err, "failed to call activation service")
	}

	defer response.Body.Close()

	if response.StatusCode == http.StatusOK || response.StatusCode == http.StatusConflict {
		// it went fine.
		return nil
	}

	return fmt.Errorf("failed to activate account: %s", response.Status)
}

// EnsureAccount makes sure account is available on blockchain
// if not, it uses activation service to create one.
// EnsureAccount is safe to call on already activated accounts and it's mostly
// a NO-OP operation unless the account funds are very low, it will then make
// sure to reactivate the account (fund it) if the free tokes are <= ReactivateThreefold uTFT
func (s *Substrate) EnsureAccount(identity Identity, activationURL, termsAndConditionsLink, terminsAndConditionsHash string) (info AccountInfo, err error) {
	log.Debug().Str("account", identity.Address()).Msg("ensuring account")
	cl, meta, err := s.GetClient()
	if err != nil {
		return info, err
	}
	info, err = s.getAccount(cl, meta, identity)
	// if account does not exist OR account has tokes less that reactivateAt
	// then we activate the account.
	if errors.Is(err, ErrAccountNotFound) || info.Data.Free.Cmp(reactivateAt) <= 0 {
		// account activation
		log.Info().Uint64("funds", info.Data.Free.Uint64()).Str("account", identity.Address()).Msg("activating account")
		if err = s.activateAccount(identity, activationURL); err != nil {
			return
		}

		// after activation this can take up to 10 seconds
		// before the account is actually there !

		exp := backoff.NewExponentialBackOff()
		exp.MaxElapsedTime = 10 * time.Second
		exp.MaxInterval = 3 * time.Second

		err = backoff.Retry(func() error {
			info, err = s.getAccount(cl, meta, identity)
			return err
		}, exp)
	}

	account, err := FromAddress(identity.Address())
	if err != nil {
		return info, errors.Wrap(err, "failed to get account id for identity")
	}

	conditions, err := s.SignedTermsAndConditions(account)
	if err != nil {
		return info, err
	}

	if len(conditions) > 0 {
		return info, nil
	}

	return info, s.AcceptTermsAndConditions(identity, termsAndConditionsLink, terminsAndConditionsHash)
}

// Identity is a user identity
type Identity interface {
	KeyPair() (subkey.KeyPair, error)
	Sign(data []byte) ([]byte, error)
	Type() string
	MultiSignature(sig []byte) types.MultiSignature
	Address() string
	PublicKey() []byte
	URI() string
}

type srIdentity struct {
	signature.KeyringPair
}
type edIdentity struct {
	signature.KeyringPair
}

func NewIdentityFromEd25519Key(sk ed25519.PrivateKey) (Identity, error) {
	str := types.HexEncodeToString(sk.Seed())
	krp, err := keyringPairFromSecret(str, network, subkeyEd25519.Scheme{})
	if err != nil {
		return nil, err
	}
	return &edIdentity{krp}, nil
}

func NewIdentityFromEd25519Phrase(phrase string) (Identity, error) {
	krp, err := keyringPairFromSecret(phrase, network, subkeyEd25519.Scheme{})
	if err != nil {
		return nil, err
	}

	return &edIdentity{krp}, nil
}

func (i *edIdentity) KeyPair() (subkey.KeyPair, error) {
	kyr, err := subkey.DeriveKeyPair(subkeyEd25519.Scheme{}, i.KeyringPair.URI)
	if err != nil {
		return nil, err
	}
	return kyr, nil
}

func (i edIdentity) Address() string {
	return i.KeyringPair.Address
}

func (i edIdentity) URI() string {
	return i.KeyringPair.URI
}

func (i edIdentity) PublicKey() []byte {
	return i.KeyringPair.PublicKey
}

func (i edIdentity) Sign(data []byte) ([]byte, error) {
	return signBytes(data, i.KeyringPair.URI, subkeyEd25519.Scheme{})
}

func (i edIdentity) MultiSignature(sig []byte) types.MultiSignature {
	return types.MultiSignature{IsEd25519: true, AsEd25519: types.NewSignature(sig)}
}

func (i edIdentity) Type() string {
	return "ed25519"
}

func NewIdentityFromSr25519Phrase(phrase string) (Identity, error) {
	krp, err := keyringPairFromSecret(phrase, network, subkeySr25519.Scheme{})
	if err != nil {
		return nil, err
	}
	return &srIdentity{krp}, nil
}

func (i *srIdentity) KeyPair() (subkey.KeyPair, error) {
	kyr, err := subkey.DeriveKeyPair(subkeySr25519.Scheme{}, i.KeyringPair.URI)
	if err != nil {
		return nil, err
	}
	return kyr, nil
}

func (i srIdentity) Address() string {
	return i.KeyringPair.Address
}

func (i srIdentity) URI() string {
	return i.KeyringPair.URI
}

func (i srIdentity) PublicKey() []byte {
	return i.KeyringPair.PublicKey
}

func (i srIdentity) Sign(data []byte) ([]byte, error) {
	return signBytes(data, i.KeyringPair.URI, subkeySr25519.Scheme{})
}

func (i srIdentity) MultiSignature(sig []byte) types.MultiSignature {
	return types.MultiSignature{IsSr25519: true, AsSr25519: types.NewSignature(sig)}
}

func (i srIdentity) Type() string {
	return "sr25519"
}

func (s *Substrate) getAccount(cl Conn, meta Meta, identity Identity) (info AccountInfo, err error) {
	key, err := types.CreateStorageKey(meta, "System", "Account", identity.PublicKey(), nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &info)
	if err != nil || !ok {
		if !ok {
			return info, ErrAccountNotFound
		}

		return
	}

	return
}

// GetAccount gets account info with secure key
func (s *Substrate) GetAccount(identity Identity) (info AccountInfo, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return info, err
	}

	return s.getAccount(cl, meta, identity)
}

// GetAccountPublicInfo gets the info for a given account ID
func (s *Substrate) GetAccountPublicInfo(account AccountID) (info AccountInfo, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	key, err := types.CreateStorageKey(meta, "System", "Account", account.PublicKey(), nil)
	if err != nil {
		return info, errors.Wrap(err, "failed to create substrate query key")
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &info)
	if err != nil || !ok {
		if !ok {
			return info, ErrAccountNotFound
		}
		return
	}

	return
}

// GetBalance gets the balance for a given account ID
func (s *Substrate) GetBalance(account AccountID) (balance Balance, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	key, err := types.CreateStorageKey(meta, "System", "Account", account[:], nil)
	if err != nil {
		return balance, errors.Wrap(err, "failed to create substrate query key")
	}

	var info AccountInfo
	ok, err := cl.RPC.State.GetStorageLatest(key, &info)
	balance = info.Data
	if err != nil || !ok {
		if !ok {
			return balance, ErrAccountNotFound
		}
		return
	}

	return
}
