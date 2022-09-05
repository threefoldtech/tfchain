from random import randbytes
from substrateinterface import SubstrateInterface, Keypair, KeypairType

GIGABYTE = 1024*1024*1024

substrate = SubstrateInterface(
    url="ws://127.0.0.1:9945",
    ss58_format=42,
    type_registry_preset='polkadot'
)

key_alice = Keypair.create_from_uri("//Alice")

alice_insert_key_params = ["tft!", "//Alice", key_alice.public_key.hex()]
substrate.rpc_request("author_insertKey", alice_insert_key_params)

alice_insert_key_params = ["smct", "//Alice", key_alice.public_key.hex()]
substrate.rpc_request("author_insertKey", alice_insert_key_params)


call_user_accept_tc = substrate.compose_call("TfgridModule", 
                              "user_accept_tc", 
                              {
                                  "document_link": "garbage",
                                  "document_hash": "garbage"
                              }
                              )
call_user_accept_tc_signed = substrate.create_signed_extrinsic(call_user_accept_tc, key_alice)

response = substrate.submit_extrinsic(call_user_accept_tc_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)    



call_create_twin = substrate.compose_call("TfgridModule", 
                                             "create_twin", 
                                             {
                                                 "ip": "::1"
                                             })
call_create_twin_signed = substrate.create_signed_extrinsic(call_create_twin, key_alice)
response = substrate.submit_extrinsic(call_create_twin_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)
    


call_create_farm = substrate.compose_call("TfgridModule", 
                                             "create_farm", 
                                             {
                                                 "name": "myfarm",
                                                 "public_ips": []
                                             })
call_create_farm_signed = substrate.create_signed_extrinsic(call_create_farm, key_alice)
response = substrate.submit_extrinsic(call_create_farm_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)
    


call_create_node = substrate.compose_call("TfgridModule", 
                                             "create_node", 
                                             {
                                                 "farm_id": "1",
                                                 "resources": {
                                                     "hru": 1024 * GIGABYTE,
                                                     "sru": 512 * GIGABYTE,
                                                     "cru": 8,
                                                     "mru": 16 * GIGABYTE},
                                                 "location": {
                                                     "longitude": "garbage",
                                                     "latitude": "garbage"
                                                 },
                                                 "country": "Belgium",
                                                 "city": "Ghent",
                                                 "interfaces": [],
                                                 "secure_boot": False,
                                                 "virtualized": False,
                                                 "serial_number": "garbage"
                                             })
call_create_node_signed = substrate.create_signed_extrinsic(call_create_node, key_alice)
response = substrate.submit_extrinsic(call_create_node_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)


call_create_contract = substrate.compose_call("SmartContractModule",
                                              "create_node_contract",
                                              {
                                                  "node_id": 1,
                                                  "deployment_data": randbytes(32),
                                                  "deployment_hash": randbytes(32),
                                                  "public_ips": 0,
                                                  "solution_provider_id": None
                                              })
call_create_contract_signed = substrate.create_signed_extrinsic(call_create_contract, key_alice)
response = substrate.submit_extrinsic(call_create_contract_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)    

call_create_contract = substrate.compose_call("SmartContractModule",
                                              "report_contract_resources",
                                              {
                                                  "contract_resources": [
                                                    {
                                                        "contract_id": 1,
                                                        "used": {
                                                            "hru": 0,
                                                            "sru": 20 * GIGABYTE,
                                                            "cru": 2,
                                                            "mru": 4 * GIGABYTE
                                                        }
                                                    }
                                                  ]
                                              })
call_create_contract_signed = substrate.create_signed_extrinsic(call_create_contract, key_alice)
response = substrate.submit_extrinsic(call_create_contract_signed, wait_for_finalization=True)
if response.error_message:
    print(response.error_message)    