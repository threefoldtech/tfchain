package main

import (
	"fmt"
	"math/big"
	"os"
	"strconv"
	"strings"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

func init() {
	zerolog.SetGlobalLevel(zerolog.Disabled)
}

const (
	mainnetURL = "wss://tfchain.grid.tf/ws"
	tftDivisor = 10_000_000
)

type WalletInfo struct {
	Address  string      `toml:"address"`
	TwinID   uint32      `toml:"twin_id,omitempty"`
	Balance  BalanceInfo `toml:"balance"`
	IsHoster bool        `toml:"is_hoster"`
	Farm     *FarmInfo   `toml:"farm,omitempty"`
	Nodes    []NodeInfo  `toml:"nodes,omitempty"`
}

type BalanceInfo struct {
	Free     string `toml:"free"`
	Reserved string `toml:"reserved"`
	FreeUTFT string `toml:"free_utft"`
}

type FarmInfo struct {
	ID            uint32 `toml:"id"`
	Name          string `toml:"name"`
	DedicatedFarm bool   `toml:"dedicated_farm"`
}

type NodeInfo struct {
	ID      uint32 `toml:"id"`
	FarmID  uint32 `toml:"farm_id"`
	TwinID  uint32 `toml:"twin_id"`
	City    string `toml:"city"`
	Country string `toml:"country"`
	CRU     uint64 `toml:"cru"`
	MRU     uint64 `toml:"mru"`
	SRU     uint64 `toml:"sru"`
	HRU     uint64 `toml:"hru"`
}

func main() {
	key := os.Getenv("TFCHAIN_KEY")
	if key == "" {
		fmt.Fprintln(os.Stderr, "error: TFCHAIN_KEY environment variable is not set")
		os.Exit(1)
	}

	key = strings.TrimSpace(key)

	identity, err := createIdentity(key)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: failed to create identity: %v\n", err)
		os.Exit(1)
	}

	mgr := substrate.NewManager(mainnetURL)
	sub, err := mgr.Substrate()
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: failed to connect to TFChain: %v\n", err)
		os.Exit(1)
	}
	defer sub.Close()

	// Check for subcommand
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "sendto":
			cmdSendTo(sub, identity, os.Args[2:])
			return
		default:
			fmt.Fprintf(os.Stderr, "error: unknown command: %s\n", os.Args[1])
			os.Exit(1)
		}
	}

	// Default: show wallet info
	info, err := getWalletInfo(sub, identity)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: failed to get wallet info: %v\n", err)
		os.Exit(1)
	}

	printTOML(info)
}

func cmdSendTo(sub *substrate.Substrate, identity substrate.Identity, args []string) {
	if len(args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: tfwallet sendto <address> tft <amount>")
		os.Exit(1)
	}

	destAddr := args[0]
	if args[1] != "tft" {
		fmt.Fprintln(os.Stderr, "error: only 'tft' is supported")
		os.Exit(1)
	}
	amountStr := args[2]

	// Parse amount (supports decimal like 10.5)
	amountUTFT, err := parseTFTAmount(amountStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: invalid amount: %v\n", err)
		os.Exit(1)
	}

	// Parse destination address
	destAccount, err := substrate.FromAddress(destAddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: invalid destination address: %v\n", err)
		os.Exit(1)
	}

	// Transfer
	err = sub.Transfer(identity, amountUTFT, destAccount)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: transfer failed: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("transferred = %q\n", amountStr)
	fmt.Printf("to = %q\n", destAddr)
	fmt.Printf("amount_utft = %d\n", amountUTFT)
}

// parseTFTAmount parses a TFT amount string (e.g., "10", "10.5", "0.001") to uTFT
func parseTFTAmount(s string) (uint64, error) {
	s = strings.TrimSpace(s)

	// Handle decimal
	parts := strings.Split(s, ".")
	if len(parts) > 2 {
		return 0, fmt.Errorf("invalid number format")
	}

	whole := parts[0]
	if whole == "" {
		whole = "0"
	}

	wholeVal, err := strconv.ParseUint(whole, 10, 64)
	if err != nil {
		return 0, err
	}

	result := wholeVal * tftDivisor

	if len(parts) == 2 {
		decimal := parts[1]
		// Pad or truncate to 7 decimal places
		if len(decimal) > 7 {
			decimal = decimal[:7]
		}
		for len(decimal) < 7 {
			decimal += "0"
		}
		decVal, err := strconv.ParseUint(decimal, 10, 64)
		if err != nil {
			return 0, err
		}
		result += decVal
	}

	return result, nil
}

func createIdentity(key string) (substrate.Identity, error) {
	if strings.HasPrefix(key, "0x") {
		identity, err := substrate.NewIdentityFromSr25519Phrase(key)
		if err == nil {
			return identity, nil
		}
		return substrate.NewIdentityFromEd25519Phrase(key)
	}

	identity, err := substrate.NewIdentityFromSr25519Phrase(key)
	if err == nil {
		return identity, nil
	}

	return substrate.NewIdentityFromEd25519Phrase(key)
}

func getWalletInfo(sub *substrate.Substrate, identity substrate.Identity) (*WalletInfo, error) {
	info := &WalletInfo{
		Address: identity.Address(),
	}

	account, err := sub.GetAccount(identity)
	if err != nil && err != substrate.ErrAccountNotFound {
		return nil, fmt.Errorf("failed to get account: %w", err)
	}

	freeBalance := account.Data.Free
	reservedBalance := account.Data.Reserved

	info.Balance = BalanceInfo{
		Free:     formatTFT(freeBalance),
		Reserved: formatTFT(reservedBalance),
		FreeUTFT: freeBalance.String(),
	}

	twinID, err := sub.GetTwinByPubKey(identity.PublicKey())
	if err == nil && twinID > 0 {
		info.TwinID = twinID

		twin, err := sub.GetTwin(twinID)
		if err == nil && twin != nil {
			nodeID, err := sub.GetNodeByTwinID(twinID)
			if err == nil && nodeID > 0 {
				info.IsHoster = true

				node, err := sub.GetNode(nodeID)
				if err == nil && node != nil {
					farm, err := sub.GetFarm(uint32(node.FarmID))
					if err == nil && farm != nil {
						info.Farm = &FarmInfo{
							ID:            uint32(farm.ID),
							Name:          farm.Name,
							DedicatedFarm: farm.DedicatedFarm,
						}

						nodeIDs, err := sub.GetNodes(uint32(farm.ID))
						if err == nil {
							for _, nid := range nodeIDs {
								n, err := sub.GetNode(nid)
								if err == nil && n != nil {
									info.Nodes = append(info.Nodes, NodeInfo{
										ID:      uint32(n.ID),
										FarmID:  uint32(n.FarmID),
										TwinID:  uint32(n.TwinID),
										City:    n.Location.City,
										Country: n.Location.Country,
										CRU:     uint64(n.Resources.CRU),
										MRU:     uint64(n.Resources.MRU),
										SRU:     uint64(n.Resources.SRU),
										HRU:     uint64(n.Resources.HRU),
									})
								}
							}
						}
					}
				}
			}
		}
	}

	return info, nil
}

func formatTFT(amount types.U128) string {
	if amount.Int == nil {
		return "0.0000000"
	}

	divisor := big.NewInt(tftDivisor)
	whole := new(big.Int).Div(amount.Int, divisor)
	remainder := new(big.Int).Mod(amount.Int, divisor)

	return fmt.Sprintf("%s.%07d", whole.String(), remainder.Int64())
}

func printTOML(info *WalletInfo) {
	fmt.Printf("address = %q\n", info.Address)
	if info.TwinID > 0 {
		fmt.Printf("twin_id = %d\n", info.TwinID)
	}
	fmt.Printf("is_hoster = %t\n", info.IsHoster)
	fmt.Println()

	fmt.Println("[balance]")
	fmt.Printf("free = %q\n", info.Balance.Free)
	fmt.Printf("reserved = %q\n", info.Balance.Reserved)
	fmt.Printf("free_utft = %q\n", info.Balance.FreeUTFT)

	if info.Farm != nil {
		fmt.Println()
		fmt.Println("[farm]")
		fmt.Printf("id = %d\n", info.Farm.ID)
		fmt.Printf("name = %q\n", info.Farm.Name)
		fmt.Printf("dedicated_farm = %t\n", info.Farm.DedicatedFarm)
	}

	if len(info.Nodes) > 0 {
		fmt.Println()
		for i, node := range info.Nodes {
			fmt.Println("[[nodes]]")
			fmt.Printf("id = %d\n", node.ID)
			fmt.Printf("farm_id = %d\n", node.FarmID)
			fmt.Printf("twin_id = %d\n", node.TwinID)
			fmt.Printf("city = %q\n", node.City)
			fmt.Printf("country = %q\n", node.Country)
			fmt.Printf("cru = %d\n", node.CRU)
			fmt.Printf("mru = %d\n", node.MRU)
			fmt.Printf("sru = %d\n", node.SRU)
			fmt.Printf("hru = %d\n", node.HRU)
			if i < len(info.Nodes)-1 {
				fmt.Println()
			}
		}
	}
}
