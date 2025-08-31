package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// PricingPolicy struct represents a PricingPolicy
type PricingPolicy struct {
	Versioned
	ID                     types.U32 `json:"id"`
	Name                   string    `json:"name"`
	SU                     Policy    `json:"su"`
	CU                     Policy    `json:"cu"`
	NU                     Policy    `json:"nu"`
	IPU                    Policy    `json:"ipu"`
	UniqueName             Policy    `json:"unique_name"`
	DomainName             Policy    `json:"domain_name"`
	FoundationAccount      AccountID `json:"foundation_name"`
	CertifiedSalesAccount  AccountID `json:"certified_sales_account"`
	DedicatedNodesDiscount types.U8  `json:"dedication_nodes_discount"`
}

// GetPricingPolicies gets pricing policy from tfgrid module
func (s *Substrate) GetPricingPolicy(id uint32) (pricingPolicy PricingPolicy, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	bytes, err := Encode(id)
	if err != nil {
		return pricingPolicy, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "PricingPolicies", bytes)
	if err != nil {
		return pricingPolicy, errors.Wrap(err, "failed to create substrate query key")
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &pricingPolicy)
	if err != nil {
		return pricingPolicy, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok {
		return pricingPolicy, errors.Wrap(ErrNotFound, "pricing policy not found")
	}

	return
}

// GetTFTPrice gets the TFT price
func (s *Substrate) GetTFTPrice() (price types.U32, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	key, err := types.CreateStorageKey(meta, "TFTPriceModule", "TftPrice")
	if err != nil {
		return price, errors.Wrap(err, "failed to create substrate query key")
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &price)
	if err != nil {
		return price, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok {
		return price, errors.Wrap(ErrNotFound, "price not found")
	}

	return
}

// GetAverageTFTPrice gets the average TFT price
func (s *Substrate) GetAverageTFTPrice() (price types.U32, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	key, err := types.CreateStorageKey(meta, "TFTPriceModule", "AverageTftPrice")
	if err != nil {
		return price, errors.Wrap(err, "failed to create substrate query key")
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &price)
	if err != nil {
		return price, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok {
		return price, errors.Wrap(ErrNotFound, "average price not found")
	}

	return
}

// GetTFTBillingRate returns the current billing rate (in mUSD) used on-chain,
// computed as AverageTftPrice clamped between MinTftPrice and MaxTftPrice.
func (s *Substrate) GetTFTBillingRate() (rate types.U32, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return
	}

	// Build keys
	keyAvg, err := types.CreateStorageKey(meta, "TFTPriceModule", "AverageTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (AverageTftPrice)")
	}
	keyMin, err := types.CreateStorageKey(meta, "TFTPriceModule", "MinTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (MinTftPrice)")
	}
	keyMax, err := types.CreateStorageKey(meta, "TFTPriceModule", "MaxTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (MaxTftPrice)")
	}

	// Read latest values
	var avg, min, max types.U32
	ok, err := cl.RPC.State.GetStorageLatest(keyAvg, &avg)
	if err != nil {
		return rate, errors.Wrap(err, "failed to read AverageTftPrice")
	}
	if !ok {
		return rate, errors.Wrap(ErrNotFound, "AverageTftPrice not found")
	}

	ok, err = cl.RPC.State.GetStorageLatest(keyMin, &min)
	if err != nil {
		return rate, errors.Wrap(err, "failed to read MinTftPrice")
	}
	if !ok {
		return rate, errors.Wrap(ErrNotFound, "MinTftPrice not found")
	}

	ok, err = cl.RPC.State.GetStorageLatest(keyMax, &max)
	if err != nil {
		return rate, errors.Wrap(err, "failed to read MaxTftPrice")
	}
	if !ok {
		return rate, errors.Wrap(ErrNotFound, "MaxTftPrice not found")
	}

	// Clamp
	rate = avg
	if rate < min {
		rate = min
	}
	if rate > max {
		rate = max
	}
	return
}

// GetTFTBillingRateAt returns the billing rate (in mUSD) at a specific block number,
// computed as AverageTftPrice clamped between MinTftPrice and MaxTftPrice at that block.
func (s *Substrate) GetTFTBillingRateAt(block uint64) (rate types.U32, err error) {
	cl, _, err := s.GetClient()
	if err != nil {
		return
	}

	// Resolve block hash
	bh, err := cl.RPC.Chain.GetBlockHash(block)
	if err != nil {
		return rate, errors.Wrap(err, "failed to resolve block hash")
	}

	// Metadata at block
	metaAtBlock, err := cl.RPC.State.GetMetadata(bh)
	if err != nil {
		return rate, errors.Wrap(err, "failed to get metadata at block")
	}

	// Keys at block
	keyAvg, err := types.CreateStorageKey(metaAtBlock, "TFTPriceModule", "AverageTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (AverageTftPrice)")
	}
	keyMin, err := types.CreateStorageKey(metaAtBlock, "TFTPriceModule", "MinTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (MinTftPrice)")
	}
	keyMax, err := types.CreateStorageKey(metaAtBlock, "TFTPriceModule", "MaxTftPrice")
	if err != nil {
		return rate, errors.Wrap(err, "failed to create storage key (MaxTftPrice)")
	}

	// Read at block
	var avg, min, max types.U32
	raw, err := cl.RPC.State.GetStorageRaw(keyAvg, bh)
	if err != nil {
		return rate, errors.Wrap(err, "failed to get AverageTftPrice at block")
	}
	if len(*raw) == 0 {
		return rate, errors.Wrap(ErrNotFound, "AverageTftPrice not found at block")
	}
	if err := Decode(*raw, &avg); err != nil {
		return rate, errors.Wrap(err, "failed to decode AverageTftPrice")
	}

	raw, err = cl.RPC.State.GetStorageRaw(keyMin, bh)
	if err != nil {
		return rate, errors.Wrap(err, "failed to get MinTftPrice at block")
	}
	if len(*raw) == 0 {
		return rate, errors.Wrap(ErrNotFound, "MinTftPrice not found at block")
	}
	if err := Decode(*raw, &min); err != nil {
		return rate, errors.Wrap(err, "failed to decode MinTftPrice")
	}

	raw, err = cl.RPC.State.GetStorageRaw(keyMax, bh)
	if err != nil {
		return rate, errors.Wrap(err, "failed to get MaxTftPrice at block")
	}
	if len(*raw) == 0 {
		return rate, errors.Wrap(ErrNotFound, "MaxTftPrice not found at block")
	}
	if err := Decode(*raw, &max); err != nil {
		return rate, errors.Wrap(err, "failed to decode MaxTftPrice")
	}

	// Clamp
	rate = avg
	if rate < min {
		rate = min
	}
	if rate > max {
		rate = max
	}
	return
}
