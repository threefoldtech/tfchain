package main

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/centrifuge/go-substrate-rpc-client/v4/signature"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/stellar/go/keypair"
	"github.com/vedhavyas/go-subkey"
	subkeyEd25519 "github.com/vedhavyas/go-subkey/ed25519"
)

const (
	network = 42
)

func main() {
	_, pkey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		panic(err)
	}

	str := types.HexEncodeToString(pkey.Seed())
	krp, err := keyringPairFromSecret(str, network)
	if err != nil {
		panic(err)
	}

	b, err := hex.DecodeString(strings.Trim(krp.URI, "0x"))
	if err != nil {
		panic(err)
	}
	var dest [32]byte
	copy(dest[:], b)

	k, err := keypair.FromRawSeed(dest)
	if err != nil {
		panic(err)
	}

	fmt.Printf("\nTF Chain raw seed: %s \n", krp.URI)
	fmt.Printf("TF Chain public key (SS58 address): %s \n\n", krp.Address)
	fmt.Printf("Stellar Secret: %s \n", k.Seed())
	fmt.Printf("Stellar Address: %s \n", k.Address())
}

func keyringPairFromSecret(seedOrPhrase string, network uint8) (signature.KeyringPair, error) {
	scheme := subkeyEd25519.Scheme{}
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
