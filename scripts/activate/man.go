package main

import (
	"fmt"
	"log"
	"math/big"
	"strings"

	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

const (
	// Your mnemonic phrase - REPLACE THIS WITH YOUR ACTUAL MNEMONIC
	MNEMONIC = "hen broken hedgehog page tribe motion rate mix mammal arctic alien clump"

	// Alice's well-known address for devnet - used for sending tokens
	ALICE_ADDRESS = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

	// Local tfchain URL
	CHAIN_URL = "ws://127.0.0.1:9944"

	// Amount to transfer (9999 TFT in units of uTFT)
	TRANSFER_AMOUNT = 9999 * 10_000_000 // 9999 TFT in uTFT (micro TFT)

	// Terms and conditions dummy data
	TERMS_LINK = "https://manual.grid.tf/knowledge_base/legal/terms_conditions_all3.html"
	TERMS_HASH = "e4174baeb17012f3b610b8b0d2354d87"
)

func main() {
	fmt.Println("Starting TFChain account activation script...")

	// Step 1: Connect to TFChain
	fmt.Println("Connecting to TFChain...")
	manager := substrate.NewManager(CHAIN_URL)
	substrateConn, err := manager.Substrate()
	if err != nil {
		log.Fatalf("Failed to connect to substrate: %v", err)
	}
	defer substrateConn.Close()
	fmt.Println("✓ Connected to TFChain")

	// Step 2: Create identity from mnemonic
	fmt.Println("Creating identity from mnemonic...")
	identity, err := substrate.NewIdentityFromSr25519Phrase(MNEMONIC)
	if err != nil {
		log.Fatalf("Failed to create identity from mnemonic: %v", err)
	}
	fmt.Printf("✓ Identity created with address: %s\n", identity.Address())

	// Step 3: Create Alice's identity for sending tokens
	fmt.Println("Setting up Alice's identity for token transfer...")
	aliceIdentity, err := substrate.NewIdentityFromSr25519Phrase("//Alice")
	if err != nil {
		log.Fatalf("Failed to create Alice identity: %v", err)
	}
	fmt.Printf("✓ Alice identity created with address: %s\n", aliceIdentity.Address())

	// Step 4: Transfer tokens from Alice to user account
	fmt.Println("Transferring 9999 TFT from Alice to your account...")
	myAccount, err := substrate.FromAddress(identity.Address())
	if err != nil {
		log.Fatalf("Failed to parse my account address: %v", err)
	}

	err = substrateConn.Transfer(aliceIdentity, TRANSFER_AMOUNT, myAccount)
	if err != nil {
		log.Fatalf("Failed to transfer tokens: %v", err)
	}
	fmt.Printf("✓ Successfully transferred %d uTFT (%.3f TFT) to your account\n", TRANSFER_AMOUNT, float64(TRANSFER_AMOUNT)/10_000_000)

	// Step 5: Check account balance
	fmt.Println("Checking account balance...")
	accountInfo, err := substrateConn.GetAccount(identity)
	if err != nil {
		log.Fatalf("Failed to get account info: %v", err)
	}

	balance := accountInfo.Data.Free
	balanceFloat := new(big.Float).SetInt(balance.Int)
	balanceFloat.Quo(balanceFloat, big.NewFloat(10_000_000)) // Convert from uTFT to TFT
	fmt.Printf("✓ Current account balance: %s TFT\n", balanceFloat.String())

	// Step 6: Accept Terms and Conditions
	fmt.Println("Accepting Terms and Conditions...")

	err = substrateConn.AcceptTermsAndConditions(identity, TERMS_LINK, TERMS_HASH)
	if err != nil {
		log.Fatalf("Failed to accept terms and conditions: %v", err)
	}
	fmt.Println("✓ Terms and conditions accepted")

	// Step 7: Create Twin (this activates the account)
	fmt.Println("Creating twin to activate account...")

	// Create twin with optional relay and public key
	twinID, err := substrateConn.CreateTwin(identity, "", []byte{})
	if err != nil {
		log.Fatalf("Failed to create twin: %v", err)
	}

	fmt.Printf("✓ Account successfully activated!\n")
	fmt.Printf("✓ Twin created with ID: %d\n", twinID)

	// Step 8: Verify twin creation
	fmt.Println("Verifying twin creation...")
	twin, err := substrateConn.GetTwin(twinID)
	if err != nil {
		log.Fatalf("Failed to get twin: %v", err)
	}

	fmt.Printf("✓ Twin verification successful:\n")
	fmt.Printf("  - Twin ID: %d\n", twin.ID)
	fmt.Printf("  - Account: %s\n", twin.Account.String())
	fmt.Printf("  - Relay: %+v\n", twin.Relay)

	// Final summary
	fmt.Println("\n" + strings.Repeat("=", 50))
	fmt.Println("ACTIVATION COMPLETE!")
	fmt.Println(strings.Repeat("=", 50))
	fmt.Printf("Your account address: %s\n", identity.Address())
	fmt.Printf("Account balance: %s TFT\n", balanceFloat.String())
	fmt.Printf("Twin ID: %d\n", twinID)
	fmt.Println("Your account is now fully activated on TFChain!")
	fmt.Println(strings.Repeat("=", 50))
}
