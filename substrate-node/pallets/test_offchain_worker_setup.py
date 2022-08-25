from re import sub
from substrateinterface import SubstrateInterface, Keypair, KeypairType

GIGABYTE = 1024*1024*1024

substrate = SubstrateInterface(
    url="ws://127.0.0.1:9946",
    ss58_format=42,
    type_registry_preset='polkadot'
)

key_bob = Keypair.create_from_uri("//Bob")

bob_insert_key_params = ["tft!", "//Bob", key_bob.public_key.hex()]
substrate.rpc_request("author_insertKey", bob_insert_key_params)

bob_insert_key_params = ["smct", "//Bob", key_bob.public_key.hex()]
substrate.rpc_request("author_insertKey", bob_insert_key_params)

