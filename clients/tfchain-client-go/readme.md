[![test-substrate-client](https://github.com/threefoldtech/substrate-client/actions/workflows/test.yml/badge.svg)](https://github.com/threefoldtech/substrate-client/actions/workflows/test.yml)

# **TFchain go client**

*   This library is a go implementation of a client for the TFChain.
*   Internally, our own [fork](https://github.com/threefoldtech/go-substrate-rpc-client) of <https://github.com/centrifuge/go-substrate-rpc-client> is used to make substrate rpc calls.
*   Used in multiple repos like [zos](https://github.com/threefoldtech/zos), [rmb-go](https://github.com/threefoldtech/rmb_go), and [terraform-provider-grid](https://github.com/threefoldtech/terraform-provider-grid).

## **Usage**

To make substrate calls:

*   First, start a substrate connection against the desired url for the chain:

    ```go
    manager := NewManager("wss://tfchain.grid.tf/ws")
    substrateConnection, err := manager.Substrate()
    ```

*   These are the urls for different chain networks:

    *   devnet:  <wss://tfchain.dev.grid.tf/ws>
    *   testnet: <wss://tfchain.test.grid.tf/ws>
    *   qanet:   <wss://tfchain.qa.grid.tf/ws>
    *   mainnet: <wss://tfchain.grid.tf/ws>

*   It is the user's responsibility to close the connection.

    ```go
    defer substrateConnection.Close()
    ```

*   Then, a user could use the provided api calls to communicate with the chain. like:

    ```go
    contractID, err := substrateConnection.CreateNodeContract(identity, nodeID, body, hash, publicIPsCount, solutionProviderID)
    ```

*   Also, if a connection is closed for some reason like timing out, internally, it is reopened if nothing blocks.

*   All provided api calls are found under the Substrate struct.

## **Run tests**

To run the tests, you could either run it against a local docker image of the TFChain, or against devnet

*   ### **Run against local docker image**

    To run tests against a local docker image of tfchain, you need to set CI environment variable to anything actually.

    ```bash
    docker run -d -p 9944:9944 threefolddev/tfchain:2.2.0-rc8 --dev --rpc-external
    sleep 3
    export CI="true"
    go test . -v
    ```

*   ### **Run against devnet**

    ```bash
    unset CI
    go test . -v
    ```

### **Test Coverage**

*   30.6% of statements

## **Workflows**

*   ### **Test**

    *   This workflow runs all tests found in the root directory against a local docker image of the [TFChain](https://github.com/threefoldtech/tfchain) found [here](https://hub.docker.com/r/threefolddev/tfchain).

*   ### **Lint**

    *   This workflow ensures linting, so make sure to run these commands without any errors before pushing code:

        ```bash
        golangci-lint run
        ```

        ```bash
        gofmt -d -l .
        ```
