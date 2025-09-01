package main

import (
	"flag"
	"fmt"
	"log"

	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

func main() {
	endpoint := flag.String("endpoint", "wss://tfchain.grid.tf/ws", "TFChain WebSocket endpoint")
	block := flag.Uint64("block", 0, "Optional block number for historical billing rate (0 = latest)")
	flag.Parse()

	mgr := substrate.NewManager(*endpoint)
	s, err := mgr.Substrate()
	if err != nil {
		log.Fatalf("connect failed: %v", err)
	}
	defer s.Close()

	// Historical billing rate if requested
	if *block > 0 {
		rateAt, err := s.GetTFTBillingRateAt(*block)
		if err != nil {
			log.Fatalf("GetTFTBillingRateAt(%d) failed: %v", *block, err)
		}
		fmt.Printf("Billing Rate at block %d (mUSD, clamped): %d\n", *block, uint32(rateAt))
	} else {
		// Latest billing rate
		rate, err := s.GetTFTBillingRate()
		if err != nil {
			log.Fatalf("GetTFTBillingRate failed: %v", err)
		}
		fmt.Printf("Latest Billing Rate (mUSD, clamped): %d\n", uint32(rate))
	}
}
