package main

import (
	"bytes"
	"fmt"

	gsrpc "github.com/leesmet/go-substrate-rpc-client"
	"github.com/leesmet/go-substrate-rpc-client/scale"
	"github.com/leesmet/go-substrate-rpc-client/types"
)

// Client is a struct that holds the api client
type Client struct {
	api *gsrpc.SubstrateAPI
}

type Twin struct {
	TwinID   types.U64
	Pubkey   types.AccountID
	PeerID   []types.U8
	Entities []entityProof
}

type entityProof struct {
	entityID  types.U64
	signature []types.U8
}

// NewClient creates a new substrate api client
func NewClient(url string) (*Client, error) {
	if url == "" {
		url = "ws://localhost:9944"
	}
	api, err := gsrpc.NewSubstrateAPI(url)
	if err != nil {
		return nil, err
	}

	return &Client{
		api: api,
	}, nil
}

// GetTwin gets a twin by id from storage
func (c *Client) GetTwin(twinID uint64) (Twin, error) {
	meta, err := c.api.RPC.State.GetMetadataLatest()
	if err != nil {
		return Twin{}, err
	}

	buf := bytes.NewBuffer(nil)
	enc := scale.NewEncoder(buf)
	if err := enc.Encode(twinID); err != nil {
		return Twin{}, err
	}
	key := buf.Bytes()

	key, err = types.CreateStorageKey(meta, "TemplateModule", "Twins", key, nil)
	if err != nil {
		return Twin{}, err
	}

	var twin Twin
	ok, err := c.api.RPC.State.GetStorageLatest(key, &twin)
	if err != nil || !ok {
		return Twin{}, err
	}

	return twin, nil
}

func byteSliceToString(bs []types.U8) string {
	b := make([]byte, len(bs))
	for i, v := range bs {
		b[i] = byte(v)
	}
	return string(b)
}

func main() {
	c, err := NewClient("")
	if err != nil {
		panic(err)
	}

	t, err := c.GetTwin(0)
	if err != nil {
		panic(err)
	}

	fmt.Printf("Twin: %v \n", t.TwinID)
	fmt.Printf("Peer ID: %v", byteSliceToString(t.PeerID))
}
