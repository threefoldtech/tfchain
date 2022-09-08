import argparse
from datetime import datetime
import logging
import os
from os.path import dirname, join
import re
from shutil import rmtree
import subprocess
import tempfile
import time


SUBSTRATE_NODE_DIR = dirname(os.getcwd())
TFCHAIN_EXE = join(SUBSTRATE_NODE_DIR, "target", "release", "tfchain")

RE_NODE_STARTED = re.compile("Running JSON-RPC WS server")

DEFAULT_TIMEOUT_IN_SECONDS = 600

def wait_till_node_ready(log_file, timeout_in_seconds=DEFAULT_TIMEOUT_IN_SECONDS):
    start = datetime.now()
    while True:
        elapsed = datetime.now() - start
        
        if elapsed.total_seconds() >= DEFAULT_TIMEOUT_IN_SECONDS:
            raise Exception(f"Timeout on starting the node! See {log_file}")
        
        with open(log_file, "r") as fd:
            for line in reversed(fd.readlines()):
                if RE_NODE_STARTED.search(line):
                    return
                

def execute_command(cmd, log_file=None):
    if log_file is None:
        log_file = tempfile.mktemp()
        
    fd = open(log_file, 'w')
    logging.info("Running command\n\t> %s\nand saving output in file %s", " ".join([f"{arg}" for arg in cmd]), log_file)
    p = subprocess.Popen(cmd, stdout=fd, stderr=fd)
    
    return p, fd

'''
./target/release/tfchain 
    --base-path /tmp/alice
    --chain local
    --alice
    --port 30333
    --ws-port 9945
    --rpc-port 9933
    --node-key 0000000000000000000000000000000000000000000000000000000000000001
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0'
    --validator
    --rpc-methods Unsafe
    --rpc-cors all
'''
'''
./target/release/tfchain
    --base-path /tmp/bob
    --chain local
    --bob
    --port 30334
    --ws-port 9946
    --rpc-port 9934
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0'
    --validator
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
    --rpc-methods Unsafe
    --rpc-cors all
'''
def run_node(log_file, base_path, predefined_account, port, ws_port, rpc_port, node_key=None, bootnodes=None):
    
    
    
    cmd = [ TFCHAIN_EXE, 
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

def setup_multi_node_network():
    nodes = {}

    nodes["alice"] = run_node("node_alice.log", "/tmp/alice", "alice", 30333, 9945, 9933, "0000000000000000000000000000000000000000000000000000000000000001")
    wait_till_node_ready("node_alice.log")
    nodes["bob"] = run_node("node_bob.log", "/tmp/bob", "bob", 30334, 9946, 9934, node_key=None, bootnodes="/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp")
    wait_till_node_ready("node_bob.log")
    
    return nodes

def tear_down_multi_node_network(nodes):
    for (account, (process, log_file)) in nodes.items():
        logging.info("Terminating node %s", account)
        process.terminate()
        return_code = process.wait()
        logging.info("Node for %s has terminated.", account)
        if log_file is not None:
            log_file.close()
    

def main():
    parser = argparse.ArgumentParser(description='TODO')
    
    #parser.add_argument('integers', type=int, help='an integer for the accumulator')

    args = parser.parse_args()

    logging.basicConfig(format="%(asctime)s %(levelname)s %(message)s", level=logging.DEBUG)

    nodes = setup_multi_node_network()
    
    time.sleep(60)
    
    tear_down_multi_node_network(nodes)
    
    

if __name__ == "__main__":
    main()