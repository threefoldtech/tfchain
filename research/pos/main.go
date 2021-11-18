package main

import (
	"fmt"

	"github.com/robvanmieghem/misc/bitcoin"
	"golang.org/x/crypto/blake2b"
)

func main() {
	wantedSS58Address := "5CNposRewardAccount11111111111111111111111111111"
	decoded := bitcoin.DecodeBase58(wantedSS58Address)
	fmt.Println("wanted SS58:", wantedSS58Address)
	fmt.Printf("base58 decoded: %x\n", decoded)
	//last2 bytes should be the checksum
	checksumLength := 2
	addressBytes := decoded[1 : len(decoded)-checksumLength]
	fmt.Printf("Public key: %x\n", addressBytes)
	checksumPrefix := []byte("SS58PRE")
	addressFormat := append([]byte{decoded[0]}[:], addressBytes[:]...)
	checksum, _ := blake2b.New(64, []byte{})
	w := append(checksumPrefix[:], addressFormat[:]...)
	_, err := checksum.Write(w)
	if err != nil {
		panic(err)
	}

	h := checksum.Sum(nil)
	b := append(addressFormat[:], h[:checksumLength][:]...)
	validAddress := bitcoin.EncodeBase58(b)
	fmt.Println("Valid SS58:", validAddress)
}
