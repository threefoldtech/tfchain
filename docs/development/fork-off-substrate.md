## Fork off substrate

The fork-off-substrate tool is a handy tool that allows you to download and run a copy of any Substrate-based blockchain on your local machine. You can use it to experiment with different features and parameters of the blockchain for learning or development purposes.

In this guide, we will show you how to use the tool to create a local fork of the TFchain network.

### Prerequisites

Before you start, you need to have the following items:

*   The executable binary and the runtime WASM blob of the target blockchain. You can either compile them from the source code of the blockchain project, or copy them from a trusted node that runs the blockchain. For TFchain, you can find the source code [here](https://github.com/threefoldtech/tfchain) and the instructions on how to compile it [here](https://github.com/threefoldtech/tfchain/tree/development/docs/development). Find the Wasm binary file in the target directory. The file name should be `tfchain_runtime.compact.wasm` and the file path should be something like this:

```bash
./substrate-node/target/debug/wbuild/tfchain-runtime/tfchain_runtime.compact.wasm
```

### Steps

*   Install the `fork-off-substrate` tool dependencies on your computer, go to `tfchain` directory then follow these steps:
    ```bash
    cd ./tools/fork-off-substrate
    npm i
    ```
*   Create a folder called data inside the top folder (fork-off-substrate).
    ```bash
    mkdir data
    ```
*   Copy the executable/binary of your substrate based node inside the data folder and rename it to `binary`
    ```bash
    cp ../../substrate-node/target/debug/tfchain ./data/binary
    ```
*   Copy the runtime WASM blob of your substrate based blockchain to the data folder and rename it to `runtime.wasm`.
    ```bash
    cp ../../substrate-node/target/debug/wbuild/tfchain-runtime/tfchain_runtime.compact.wasm ./data/runtime.wasm
    ```
*   Run a full node for your blockchain locally (Recommended but should be fully synced) or have an external endpoint handy (but should be running with `--rpc-methods Unsafe` flag)
    ```bash
    ../../substrate-node/target/debug/tfchain --chain ../../substrate-node/chainspecs/dev/chainSpecRaw.json --rpc-external --rpc-methods Unsafe
    ```
*   Run the script

    *   If using a local node, simply run the script using
        ```bash
        npm start
        ```

    *   If you are using an external/non-default endpoint, you need to provide it to the script via the HTTP\_RPC\_ENDPOINT environment variable
        ```bash
        HTTP_RPC_ENDPOINT=https://<EXTERNAL END POINT> npm start
        ```
*   You should have the genesis file for the forked chain inside the data folder. It will be called `fork.json`.
*   You can now run a new chain using this genesis file
    ```bash
    ./data/binary --chain ./data/fork.json --alice
    ```

for more information about this tool, you can read this [blog post](https://mudit.blog/fork-substrate-blockchain/).
