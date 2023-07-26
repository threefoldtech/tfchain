package substrate

import (
	"context"
	"fmt"
	"reflect"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
)

// Resources type
type Resources struct {
	HRU types.U64 `json:"hru"`
	SRU types.U64 `json:"sru"`
	CRU types.U64 `json:"cru"`
	MRU types.U64 `json:"mru"`
}

// Location type
type Location struct {
	City      string `json:"city"`
	Country   string `json:"country"`
	Latitude  string `json:"latitude"`
	Longitude string `json:"longitude"`
}

// Role type
type Role struct {
	IsNode    bool `json:"is_node"`
	IsGateway bool `json:"is_gateway"`
}

type NodePower struct {
	State  PowerState `json:"state"`
	Target Power      `json:"target"`
}

type PowerState struct {
	IsUp              bool        `json:"is_up"`
	IsDown            bool        `json:"is_down"`
	AsDownBlockNumber BlockNumber `json:"as_down_block_number"`
}

// Decode implementation for the enum type
func (r *PowerState) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsUp = true
	case 1:
		r.IsDown = true
		if err := decoder.Decode(&r.AsDownBlockNumber); err != nil {
			return errors.Wrap(err, "failed to get deleted state")
		}
	default:
		return fmt.Errorf("unknown PowerState value")
	}

	return nil
}

// Encode implementation
func (r PowerState) Encode(encoder scale.Encoder) (err error) {
	if r.IsUp {
		err = encoder.PushByte(0)
	} else if r.IsDown {
		if err = encoder.PushByte(1); err != nil {
			return err
		}
		err = encoder.Encode(r.AsDownBlockNumber)
	}
	return
}

type Power struct {
	IsUp   bool `json:"is_up"`
	IsDown bool `json:"is_down"`
}

// Decode implementation for the enum type
func (r *Power) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsUp = true
	case 1:
		r.IsDown = true
	default:
		return fmt.Errorf("unknown Power value: '%d'", b)
	}

	return nil
}

// Encode implementation
func (r Power) Encode(encoder scale.Encoder) (err error) {
	if r.IsUp {
		err = encoder.PushByte(0)
	} else if r.IsDown {
		err = encoder.PushByte(1)
	} else {
		err = fmt.Errorf("invalid Power value")
	}
	return err
}

// Decode implementation for the enum type
func (r *Role) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsNode = true
	case 1:
		r.IsGateway = true
	default:
		return fmt.Errorf("unknown Role value")
	}

	return nil
}

// Encode implementation
func (r Role) Encode(encoder scale.Encoder) (err error) {
	if r.IsNode {
		err = encoder.PushByte(0)
	} else if r.IsGateway {
		err = encoder.PushByte(1)
	}

	return
}

type IP struct {
	IP string `json:"ip"`
	GW string `json:"gw"`
}

type OptionIP struct {
	HasValue bool `json:"has_value"`
	AsValue  IP   `json:"as_value"`
}

// Encode implementation
func (m OptionIP) Encode(encoder scale.Encoder) (err error) {
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
func (m *OptionIP) Decode(decoder scale.Decoder) (err error) {
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

type OptionDomain struct {
	HasValue bool   `json:"has_value"`
	AsValue  string `json:"as_value"`
}

// Encode implementation
func (m OptionDomain) Encode(encoder scale.Encoder) (err error) {
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
func (m *OptionDomain) Decode(decoder scale.Decoder) (err error) {
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

// PublicConfig type
type PublicConfig struct {
	IP4    IP           `json:"ip4"`
	IP6    OptionIP     `json:"ip6"`
	Domain OptionDomain `json:"domain"`
}

// OptionPublicConfig type
type OptionPublicConfig struct {
	HasValue bool         `json:"has_value"`
	AsValue  PublicConfig `json:"as_value"`
}

// Encode implementation
func (m OptionPublicConfig) Encode(encoder scale.Encoder) (err error) {
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
func (m *OptionPublicConfig) Decode(decoder scale.Decoder) (err error) {
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

type Interface struct {
	Name string   `json:"name"`
	Mac  string   `json:"mac"`
	IPs  []string `json:"ips"`
}

// OptionBoardSerial type
type OptionBoardSerial struct {
	HasValue bool   `json:"has_value"`
	AsValue  string `json:"as_value"`
}

// Encode implementation
func (m OptionBoardSerial) Encode(encoder scale.Encoder) (err error) {
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
func (m *OptionBoardSerial) Decode(decoder scale.Decoder) (err error) {
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

// Node type
type Node struct {
	Versioned
	ID              types.U32          `json:"id"`
	FarmID          types.U32          `json:"farm_id"`
	TwinID          types.U32          `json:"twin_id"`
	Resources       Resources          `json:"resources"`
	Location        Location           `json:"location"`
	PublicConfig    OptionPublicConfig `json:"public_config"`
	Created         types.U64          `json:"created"`
	FarmingPolicy   types.U32          `json:"farming_policy"`
	Interfaces      []Interface        `json:"interfaces"`
	Certification   NodeCertification  `json:"certification"`
	SecureBoot      bool               `json:"secure_boot"`
	Virtualized     bool               `json:"virtualized"`
	BoardSerial     OptionBoardSerial  `json:"board_serial"`
	ConnectionPrice types.U32          `json:"connection_price"`
}

// Eq compare changes on node settable fields
func (n *Node) Eq(o *Node) bool {
	return n.FarmID == o.FarmID &&
		n.TwinID == o.TwinID &&
		reflect.DeepEqual(n.Resources, o.Resources) &&
		reflect.DeepEqual(n.Location, o.Location) &&
		reflect.DeepEqual(n.Interfaces, o.Interfaces) &&
		n.SecureBoot == o.SecureBoot &&
		n.Virtualized == o.Virtualized &&
		reflect.DeepEqual(n.BoardSerial, o.BoardSerial)
}

type NodeExtra struct {
	Secure       bool              `json:"secure"`
	Virtualized  bool              `json:"virtualized"`
	SerialNumber OptionBoardSerial `json:"serial_number"`
}

// GetNodeByTwinID gets a node by twin id
func (s *Substrate) GetNodeByTwinID(twin uint32) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}
	bytes, err := Encode(twin)
	if err != nil {
		return 0, err
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "NodeIdByTwinID", bytes, nil)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}
	var id types.U32
	ok, err := cl.RPC.State.GetStorageLatest(key, &id)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok || id == 0 {
		return 0, errors.Wrap(ErrNotFound, "node not found")
	}

	return uint32(id), nil
}

// GetNode with id
func (s *Substrate) GetNode(id uint32) (*Node, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := Encode(id)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "Nodes", bytes, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	return s.getNode(cl, key)
}

type ScannedNode struct {
	ID   uint32 `json:"id"`
	Node Node   `json:"node"`
	Err  error  `json:"err"`
}

func (s *Substrate) ScanNodes(ctx context.Context, from, to uint32) (<-chan ScannedNode, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}
	ch := make(chan ScannedNode)

	getNode := func(id uint32) (*Node, error) {
		bytes, err := Encode(id)
		if err != nil {
			return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
		}

		key, err := types.CreateStorageKey(meta, "TfgridModule", "Nodes", bytes, nil)
		if err != nil {
			return nil, errors.Wrap(err, "failed to create substrate query key")
		}

		return s.getNode(cl, key)
	}

	go func(from, to uint32) {
		defer close(ch)

		for ; from <= to; from++ {
			var scanned ScannedNode
			scanned.ID = from
			node, err := getNode(from)
			if err != nil {
				scanned.Err = err
			} else {
				scanned.Node = *node
			}

			select {
			case <-ctx.Done():
				return
			case ch <- scanned:
			}
		}

	}(from, to)

	return ch, nil
}

// GetNodes gets nodes' IDs using farm id
func (s *Substrate) GetNodes(farmID uint32) ([]uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return []uint32{}, err
	}

	bytes, err := Encode(farmID)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "NodesByFarmID", bytes, nil)
	if err != nil {
		return []uint32{}, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return []uint32{}, errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return []uint32{}, errors.Wrapf(ErrNotFound, "nodes for farm ID %d is not found", farmID)
	}

	var nodes []uint32

	if err := Decode(*raw, &nodes); err != nil {
		return []uint32{}, errors.Wrap(err, "failed to load object")
	}

	return nodes, nil
}

func (s *Substrate) getNode(cl Conn, key types.StorageKey) (*Node, error) {
	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return nil, errors.Wrap(ErrNotFound, "node not found")
	}

	var node Node
	if err := Decode(*raw, &node); err != nil {
		return nil, errors.Wrap(err, "failed to load object")
	}

	return &node, nil
}

// CreateNode creates a node, this ignores public_config since
// this is only setable by the farmer
func (s *Substrate) CreateNode(identity Identity, node Node) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	if node.TwinID == 0 {
		return 0, fmt.Errorf("twin id is required")
	}

	c, err := types.NewCall(meta, "TfgridModule.create_node",
		node.FarmID,
		node.Resources,
		node.Location,
		node.Interfaces,
		node.SecureBoot,
		node.Virtualized,
		node.BoardSerial,
	)

	if err != nil {
		return 0, errors.Wrap(err, "failed to create call")
	}

	if _, err := s.Call(cl, meta, identity, c); err != nil {
		return 0, errors.Wrap(err, "failed to create node")
	}

	return s.GetNodeByTwinID(uint32(node.TwinID))

}

// UpdateNode updates a node, this ignores public_config and only keep the value
// set by the farmer
func (s *Substrate) UpdateNode(identity Identity, node Node) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	if node.ID == 0 {
		return 0, fmt.Errorf("node id is required")
	}
	if node.TwinID == 0 {
		return 0, fmt.Errorf("twin id is required")
	}

	c, err := types.NewCall(meta, "TfgridModule.update_node",
		node.ID,
		node.FarmID,
		node.Resources,
		node.Location,
		node.Interfaces,
		node.SecureBoot,
		node.Virtualized,
		node.BoardSerial,
	)

	if err != nil {
		return 0, errors.Wrap(err, "failed to create call")
	}

	callResponse, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return 0, errors.Wrap(err, "failed to update node")
	}

	log.Debug().Str("hash", callResponse.Hash.Hex()).Msg("update call hash")

	return s.GetNodeByTwinID(uint32(node.TwinID))
}

// UpdateNodeUptime updates the node uptime to given value
func (s *Substrate) UpdateNodeUptime(identity Identity, uptime uint64) (hash types.Hash, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return hash, err
	}

	c, err := types.NewCall(meta, "TfgridModule.report_uptime", uptime)

	if err != nil {
		return hash, errors.Wrap(err, "failed to create call")
	}

	hash, err = s.CallOnce(cl, meta, identity, c)
	if err != nil {
		return hash, errors.Wrap(err, "failed to update node uptime")
	}

	return hash, nil
}

// UpdateNodeUptime updates the node uptime to given value
func (s *Substrate) UpdateNodeUptimeV2(identity Identity, uptime, timestampHint uint64) (hash types.Hash, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return hash, err
	}

	c, err := types.NewCall(meta, "TfgridModule.report_uptime_v2", uptime, timestampHint)

	if err != nil {
		return hash, errors.Wrap(err, "failed to create call")
	}

	hash, err = s.CallOnce(cl, meta, identity, c)
	if err != nil {
		return hash, errors.Wrap(err, "failed to update node uptime")
	}

	return hash, nil
}

// GetNode with id
func (s *Substrate) GetLastNodeID() (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "NodeID")
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup node id")
	}

	if len(*raw) == 0 {
		return 0, errors.Wrap(ErrNotFound, "no value for last nodeid")
	}

	var v types.U32
	if err := Decode(*raw, &v); err != nil {
		return 0, err
	}

	return uint32(v), nil
}

// SetNodePowerState updates the node uptime to given value
func (s *Substrate) SetNodePowerState(identity Identity, up bool) (hash types.Hash, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return hash, err
	}

	power := Power{
		IsUp:   up,
		IsDown: !up,
	}

	c, err := types.NewCall(meta, "TfgridModule.change_power_state", power)

	if err != nil {
		return hash, errors.Wrap(err, "failed to create call")
	}

	callResponse, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return hash, errors.Wrap(err, "failed to update node power state")
	}

	return callResponse.Hash, nil
}

// GetPowerTarget returns the power target for a node
func (s *Substrate) GetPowerTarget(nodeID uint32) (power NodePower, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return power, err
	}

	bytes, err := Encode(nodeID)
	if err != nil {
		return power, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "NodePower", bytes)
	if err != nil {
		return power, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return power, errors.Wrap(err, "failed to lookup power target")
	}

	// If the result is empty, return the default power state
	if len(*raw) == 0 {
		return NodePower{
			State:  PowerState{IsUp: true, IsDown: false},
			Target: Power{IsUp: true, IsDown: false},
		}, nil
	}

	if err := Decode(*raw, &power); err != nil {
		return power, errors.Wrap(err, "failed to load object")
	}

	return power, nil
}

// SetDedicatedNodePrice sets an extra price on a node expressed in mUSD
// This price will be distributed back to the farmer if the node is rented
// Setting this price also makes the node only available to rent as dedicated
func (s *Substrate) SetDedicatedNodePrice(identity Identity, nodeId uint32, price uint64) (hash types.Hash, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return hash, err
	}

	c, err := types.NewCall(meta, "SmartContractModule.set_dedicated_node_extra_fee", nodeId, price)

	if err != nil {
		return hash, errors.Wrap(err, "failed to create call")
	}

	callResponse, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return hash, errors.Wrap(err, "failed to update node extra price")
	}

	return callResponse.Hash, nil
}

// GetDedicatedeNodePrice returns the price of a dedicated node
func (s *Substrate) GetDedicatedNodePrice(nodeID uint32) (uint64, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	bytes, err := Encode(nodeID)
	if err != nil {
		return 0, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "SmartContractModule", "DedicatedNodesExtraFee", bytes)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}

	var price types.U64

	ok, err := cl.RPC.State.GetStorageLatest(key, &price)
	if err != nil {
		return 0, err
	}

	if !ok {
		return 0, nil
	}

	return uint64(price), nil
}
