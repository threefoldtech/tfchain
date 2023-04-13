package substrate

import (
	"net"

	gsrpc "github.com/centrifuge/go-substrate-rpc-client/v4"
	"github.com/centrifuge/go-substrate-rpc-client/v4/client"
	"github.com/centrifuge/go-substrate-rpc-client/v4/rpc"
	"github.com/pkg/errors"
)

type retryingClient struct {
	client.Client
}

func newRetryingClient(cl client.Client) retryingClient {
	return retryingClient{cl}
}

func (c *retryingClient) Call(result interface{}, method string, args ...interface{}) error {
	err := c.Client.Call(result, method, args...)
	// if connection is closed, retrying should reconnect
	if errors.Is(err, net.ErrClosed) {
		err = c.Client.Call(result, method, args...)
	}
	return err
}

func newSubstrateAPI(url string) (*gsrpc.SubstrateAPI, error) {
	cl, err := client.Connect(url)
	if err != nil {
		return nil, err
	}
	rcl := newRetryingClient(cl)
	newRPC, err := rpc.NewRPC(&rcl)
	if err != nil {
		return nil, err
	}

	return &gsrpc.SubstrateAPI{
		RPC:    newRPC,
		Client: &rcl,
	}, nil
}
