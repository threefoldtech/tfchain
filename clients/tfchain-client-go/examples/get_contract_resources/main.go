package main

import (
	"log"

	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

func main() {
	mnemonicPhrase := "//Alice"
	identity, err := substrate.NewIdentityFromSr25519Phrase(mnemonicPhrase)
	if err != nil {
		log.Fatalf("Failed to create identity: %v", err)
	}
	manager := substrate.NewManager("wss://tfchain.dev.grid.tf")

	// Get a substrate client instance
	client, err := manager.Substrate()
	if err != nil {
		log.Fatalf("Failed to create substrate client: %v", err)
	}
	defer client.Close()

	// Call GetNodeContractResources with the contract ID
	err = client.BillContractForBlock(identity, 217268)
	if err != nil {
		log.Fatalf("Failed to bill contract for block: %v", err)
	}
}

// 	// Print the results
// 	fmt.Println("Contract Resources:")
// 	fmt.Printf("Contract ID: %d\n", resources.ContractID)
// 	fmt.Println("Used Resources:")
// 	fmt.Printf("  HRU: %d\n", resources.Used.HRU)
// 	fmt.Printf("  SRU: %d\n", resources.Used.SRU)
// 	fmt.Printf("  CRU: %d\n", resources.Used.CRU)
// 	fmt.Printf("  MRU: %d\n", resources.Used.MRU)
// }
