from random import randbytes
from re import sub
from substrateinterface import SubstrateInterface, Keypair

GIGABYTE = 1024*1024*1024

DEFAULT_SIGNER = "Alice"

class TfChainClient:
    def __init__(self):
        self._signer = DEFAULT_SIGNER
        self._create_keys()
    
    def _connect_to_server(self, url):
        return SubstrateInterface(url=url, ss58_format=42, type_registry_preset='polkadot')
    
    def _create_keys(self):
        self._predefined_keys = {}
        self._predefined_keys["Alice"] = Keypair.create_from_uri("//Alice")
        self._predefined_keys["Bob"] = Keypair.create_from_uri("//Bob")
        self._predefined_keys["Charlie"] = Keypair.create_from_uri("//Charlie")
        self._predefined_keys["Dave"] = Keypair.create_from_uri("//Dave")
        self._predefined_keys["Eve"] = Keypair.create_from_uri("//Eve")
        self._predefined_keys["Ferdie"] = Keypair.create_from_uri("//Ferdie")
    
    def _setup_predefined_account(self, name, url):
        substrate = self._connect_to_server(url)

        insert_key_params = ["tft!", f"//{name}", self._predefined_keys[name].public_key.hex()]
        substrate.rpc_request("author_insertKey", insert_key_params)

        insert_key_params = ["smct", f"//{name}", self._predefined_keys[name].public_key.hex()]
        substrate.rpc_request("author_insertKey", insert_key_params)
    
    def _check_signer(self, who):
        _who = who.title()
        assert _who in self._predefined_keys, f"{who} is not a predefined account, use one of {self._predefined_keys.keys()}"
        return _who
        
    def setup_alice(self, port=9945):
        self._setup_predefined_account("Alice", f"ws://127.0.0.1:{port}")

    def setup_bob(self, port=9946):
        self._setup_predefined_account("Bob", f"ws://127.0.0.1:{port}")
        
    def set_signer(self, who=DEFAULT_SIGNER):
        self._signer = self._check_signer(who)
        
    def user_accept_tc(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call_user_accept_tc = substrate.compose_call("TfgridModule", "user_accept_tc", { "document_link": "garbage", "document_hash": "garbage" })
        call_user_accept_tc_signed = substrate.create_signed_extrinsic(call_user_accept_tc, self._predefined_keys[self._signer])

        response = substrate.submit_extrinsic(call_user_accept_tc_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)
                    
    def create_twin(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call_create_twin = substrate.compose_call("TfgridModule", "create_twin", { "ip": "::1" })
        call_create_twin_signed = substrate.create_signed_extrinsic(call_create_twin, self._predefined_keys[self._signer])
        response = substrate.submit_extrinsic(call_create_twin_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)
    
    def create_farm(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        
        call_create_farm = substrate.compose_call("TfgridModule", "create_farm", { "name": "myfarm", "public_ips": [] })
        call_create_farm_signed = substrate.create_signed_extrinsic(call_create_farm, self._predefined_keys[self._signer])
        response = substrate.submit_extrinsic(call_create_farm_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)
            
    def create_node(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        
        params = { 
                  "farm_id": "1", 
                  "resources": { 
                      "hru": 1024 * GIGABYTE,
                      "sru": 512 * GIGABYTE,
                      "cru": 8,
                      "mru": 16 * GIGABYTE
                      },
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
                  }
            
        call_create_node = substrate.compose_call("TfgridModule", "create_node", params)
        call_create_node_signed = substrate.create_signed_extrinsic(call_create_node, self._predefined_keys[self._signer])
        response = substrate.submit_extrinsic(call_create_node_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)
            
    def create_node_contract(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        
        params = {
            "node_id": 1,
            "deployment_data": randbytes(32),
            "deployment_hash": randbytes(32),
            "public_ips": 0,
            "solution_provider_id": None
        }

        call_create_contract = substrate.compose_call("SmartContractModule", "create_node_contract", params)
        call_create_contract_signed = substrate.create_signed_extrinsic(call_create_contract, self._predefined_keys[self._signer])
        response = substrate.submit_extrinsic(call_create_contract_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)

    def report_contract_resources(self, port=9945, who=DEFAULT_SIGNER):
        self.set_signer(who)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        
        params = {
            "contract_resources": [{
                "contract_id": 1,
                "used": {
                    "hru": 0,
                    "sru": 20 * GIGABYTE,
                    "cru": 2,
                    "mru": 4 * GIGABYTE
                    }
                }]
            }

        call_create_contract = substrate.compose_call("SmartContractModule","report_contract_resources", params)
        call_create_contract_signed = substrate.create_signed_extrinsic(call_create_contract, self._predefined_keys[self._signer])
        response = substrate.submit_extrinsic(call_create_contract_signed, wait_for_finalization=True)
        if response.error_message:
            raise Exception(response.error_message)
