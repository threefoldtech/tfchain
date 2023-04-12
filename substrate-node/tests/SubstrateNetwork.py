import argparse
from datetime import datetime
import logging
import os
from os.path import dirname, isdir, isfile, join
import re
from shutil import rmtree
import signal
import subprocess
from substrateinterface import SubstrateInterface, Keypair
import tempfile
import time


SUBSTRATE_NODE_DIR = dirname(os.getcwd())
TFCHAIN_EXE = join(SUBSTRATE_NODE_DIR, "target", "release", "tfchain")

RE_NODE_STARTED = re.compile("Running JSON-RPC WS server")

TIMEOUT_STARTUP_IN_SECONDS = 600
TIMEOUT_TERMINATE_IN_SECONDS = 1

OUTPUT_TESTS = os.environ.get(
    "TEST_OUTPUT_DIR", join(os.getcwd(), "_output_tests"))

PREDEFINED_KEYS = {
    "Alice": Keypair.create_from_uri("//Alice"),
    "Bob": Keypair.create_from_uri("//Bob"),
    "Charlie": Keypair.create_from_uri("//Charlie"),
    "Dave": Keypair.create_from_uri("//Dave"),
    "Eve": Keypair.create_from_uri("//Eve"),
    "Ferdie": Keypair.create_from_uri("//Ferdie")
}


def wait_till_node_ready(log_file: str, timeout_in_seconds=TIMEOUT_STARTUP_IN_SECONDS):
    start = datetime.now()
    while True:
        elapsed = datetime.now() - start

        if elapsed.total_seconds() >= TIMEOUT_STARTUP_IN_SECONDS:
            raise Exception(f"Timeout on starting the node! See {log_file}")

        with open(log_file, "r") as fd:
            for line in reversed(fd.readlines()):
                if RE_NODE_STARTED.search(line):
                    return

def setup_offchain_workers(port: int, worker_account: str = "Alice"):
    logging.info("Setting up offchain workers")
    substrate = SubstrateInterface(url=f"ws://127.0.0.1:{port}", ss58_format=42, type_registry_preset='polkadot')
    
    insert_key_params = [
            "tft!", f"//{worker_account}", PREDEFINED_KEYS[worker_account].public_key.hex()]
    substrate.rpc_request("author_insertKey", insert_key_params)

def execute_command(cmd: list, log_file: str | None = None):
    if log_file is None:
        log_file = tempfile.mktemp()

    dir_of_log_file = dirname(log_file)
    if not isdir(dir_of_log_file):
        os.makedirs(dir_of_log_file)

    fd = open(log_file, 'w')
    logging.info("Running command\n\t> %s\nand saving output in file %s",
                 " ".join([f"{arg}" for arg in cmd]), log_file)
    p = subprocess.Popen(cmd, stdout=fd, stderr=fd)

    return p, fd


def run_node(log_file: str, base_path: str, predefined_account: str, port: int, ws_port: int, rpc_port: int, node_key: str | None = None, bootnodes: str | None = None):
    logging.info("Starting node with logfile %s", log_file)

    if not isfile(TFCHAIN_EXE):
        raise Exception(
            f"Executable {TFCHAIN_EXE} doesn't exist! Did you build the code?")

    cmd = [TFCHAIN_EXE,
           "--base-path", f"{base_path}",
           "--chain", "local",
           f"--{predefined_account.lower()}",
           "--port", f"{port}",
           "--ws-port", f"{ws_port}",
           "--rpc-port", f"{rpc_port}",
           "--telemetry-url", "wss://telemetry.polkadot.io/submit/ 0",
           "--validator",
           "--rpc-methods", "Unsafe",
           "--rpc-cors", "all"
           ]

    if node_key is not None:
        cmd.extend(["--node-key", f"{node_key}"])

    if bootnodes is not None:
        cmd.extend(["--bootnodes", f"{bootnodes}"])

    rmtree(base_path, ignore_errors=True)

    return execute_command(cmd, log_file)


class SubstrateNetwork:
    def __init__(self):
        self._nodes = {}

    def __del__(self):
        if len(self._nodes) > 0:
            self.tear_down_multi_node_network()

    def setup_multi_node_network(self, log_name: str = "", amt: int = 2):
        assert amt >= 2, "more then 2 nodes required for a multi node network"
        assert amt <= len(PREDEFINED_KEYS), "maximum amount of nodes reached"

        output_dir_network = join(OUTPUT_TESTS, log_name)

        rmtree(output_dir_network, ignore_errors=True)

        port = 30333
        ws_port = 9945
        rpc_port = 9933
        log_file_alice = join(output_dir_network, "node_alice.log")
        self._nodes["alice"] = run_node(log_file_alice, "/tmp/alice", "alice", port, ws_port,
                                        rpc_port, node_key="0000000000000000000000000000000000000000000000000000000000000001")
        wait_till_node_ready(log_file_alice)
        setup_offchain_workers(ws_port, "Alice")

        log_file = ""
        for x in range(1, amt):
            port += 1
            ws_port += 1
            rpc_port += 1
            name = list(PREDEFINED_KEYS.keys())[x].lower()
            log_file = join(output_dir_network, f"node_{name}.log")
            self._nodes[name] = run_node(log_file, f"/tmp/{name}", name, port, ws_port, rpc_port, node_key=None,
                                         bootnodes="/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp")
            wait_till_node_ready(log_file)
            setup_offchain_workers(ws_port, "Bob")

        logging.info("Network is up and running.")

    def tear_down_multi_node_network(self):
        for (account, (process, log_file)) in self._nodes.items():
            logging.info("Terminating node %s", account)
            process.terminate()
            process.wait(timeout=TIMEOUT_TERMINATE_IN_SECONDS)
            process.kill()
            logging.info("Node for %s has terminated.", account)
            if log_file is not None:
                log_file.close()
        self._nodes = {}
        logging.info("Teardown network completed!")


def main():
    parser = argparse.ArgumentParser(
        description="This tool allows you to start a multi node network.")

    parser.add_argument("--amount", required=False, type=int, default=2,
                        help=f"The amount of nodes to start. Should be minimum 2 and maximum {len(PREDEFINED_KEYS)}")
    args = parser.parse_args()

    logging.basicConfig(
        format="%(asctime)s %(levelname)s %(message)s", level=logging.DEBUG)

    network = SubstrateNetwork()
    network.setup_multi_node_network(args.amount)

    def handler(signum, frame):
        network.tear_down_multi_node_network()
        exit(0)

    signal.signal(signal.SIGINT, handler)
    logging.info("Press Ctrl-c to teardown the network.")
    while True:
        time.sleep(0.1)


if __name__ == "__main__":
    main()
