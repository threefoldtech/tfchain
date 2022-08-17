from re import sub
from substrateinterface import SubstrateInterface, Keypair, KeypairType

GIGABYTE = 1024*1024*1024

substrate = SubstrateInterface(
    url="ws://127.0.0.1:9946",
    ss58_format=42,
    type_registry_preset='polkadot'
)

key_alice = Keypair.create_from_uri("//Alice")

alice_insert_key_params = ["tft!", "//Alice", key_alice.public_key.hex()]
substrate.rpc_request("author_insertKey", alice_insert_key_params)

alice_insert_key_params = ["smct", "//Alice", key_alice.public_key.hex()]
substrate.rpc_request("author_insertKey", alice_insert_key_params)

