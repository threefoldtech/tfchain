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

// GetPricingPolicies gets pricing policies from tfgrid module
func (s *Substrate) GetPricingPolicies(id uint32) (pricingPolicy PricingPolicy, err error) {
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
