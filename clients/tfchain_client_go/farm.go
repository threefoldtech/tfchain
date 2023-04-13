package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// NodeCertification is a substrate enum
type NodeCertification struct {
	IsDiy       bool
	IsCertified bool
}

// Decode implementation for the enum type
func (p *NodeCertification) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		p.IsDiy = true
	case 1:
		p.IsCertified = true
	default:
		return fmt.Errorf("unknown NodeCertification value %d", b)
	}

	return nil
}

// Decode implementation for the enum type
func (p NodeCertification) Encode(encoder scale.Encoder) (err error) {
	if p.IsDiy {
		err = encoder.PushByte(0)
	} else if p.IsCertified {
		err = encoder.PushByte(1)
	}

	return
}

// NodeCertification is a substrate enum
type FarmCertification struct {
	IsNotCertified bool
	IsGold         bool
}

// Decode implementation for the enum type
func (p *FarmCertification) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		p.IsNotCertified = true
	case 1:
		p.IsGold = true
	default:
		return fmt.Errorf("unknown FarmCertification value")
	}

	return nil
}

// Decode implementation for the enum type
func (p FarmCertification) Encode(encoder scale.Encoder) (err error) {
	if p.IsNotCertified {
		err = encoder.PushByte(0)
	} else if p.IsGold {
		err = encoder.PushByte(1)
	}

	return
}

// Farm type
type Farm struct {
	Versioned
	ID                   types.U32
	Name                 string
	TwinID               types.U32
	PricingPolicyID      types.U32
	CertificationType    FarmCertification
	PublicIPs            []PublicIP
	DedicatedFarm        bool
	FarmingPoliciesLimit OptionFarmingPolicyLimit
}

type FarmingPolicyLimit struct {
	FarmingPolicyID   types.U32
	Cu                types.OptionU64
	Su                types.OptionU64
	End               types.OptionU64
	NodeCount         types.OptionU32
	NodeCertification bool
}

// OptionFarmingPolicyLimit type
type OptionFarmingPolicyLimit struct {
	HasValue bool
	AsValue  FarmingPolicyLimit
}

// Encode implementation
func (m OptionFarmingPolicyLimit) Encode(encoder scale.Encoder) (err error) {
	var i byte
	if m.HasValue {
		i = 1
	}
	err = encoder.PushByte(i)
	if err != nil {
		return err
	}

	if m.HasValue {
		err = encoder.Encode(m.AsValue)
	}

	return
}

// Decode implementation
func (m *OptionFarmingPolicyLimit) Decode(decoder scale.Decoder) (err error) {
	var i byte
	if err := decoder.Decode(&i); err != nil {
		return err
	}

	switch i {
	case 0:
		return nil
	case 1:
		m.HasValue = true
		return decoder.Decode(&m.AsValue)
	default:
		return fmt.Errorf("unknown value for Option")
	}
}

// PublicIP structure
type PublicIP struct {
	IP         string
	Gateway    string
	ContractID types.U64
}

// PublicIPInput structure
type PublicIPInput struct {
	IP      string
	Gateway string
}

// GetFarm gets a farm with ID
func (s *Substrate) GetFarm(id uint32) (*Farm, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := types.Encode(id)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "Farms", bytes, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return nil, errors.Wrap(ErrNotFound, "farm not found")
	}

	version, err := s.getVersion(*raw)
	if err != nil {
		return nil, err
	}

	var farm Farm

	switch version {
	case 4:
		fallthrough
	case 3:
		fallthrough
	case 2:
		fallthrough
	case 1:
		if err := types.Decode(*raw, &farm); err != nil {
			return nil, errors.Wrap(err, "failed to load object")
		}
	default:
		return nil, ErrUnknownVersion
	}

	return &farm, nil
}

// GetFarm gets a farm with ID
func (s *Substrate) GetFarmByName(name string) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	bytes, err := types.Encode(name)
	if err != nil {
		return 0, errors.Wrap(err, "substrate: encoding error building query arguments")
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "FarmIdByName", bytes, nil)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return 0, errors.Wrap(ErrNotFound, "farm not found")
	}

	var id uint32
	if err := types.Decode(*raw, &id); err != nil {
		return 0, errors.Wrap(err, "failed to decode farm id")
	}

	return id, nil
}

// CreateFarm creates a farm
// takes in a name and public ip list
func (s *Substrate) CreateFarm(identity Identity, name string, publicIps []PublicIPInput) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	if name == "" {
		return fmt.Errorf("name cannot be empty")
	}

	c, err := types.NewCall(meta, "TfgridModule.create_farm",
		name, publicIps,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	if _, err := s.Call(cl, meta, identity, c); err != nil {
		return errors.Wrap(err, "failed to create farm")
	}

	return nil
}
