from cmath import exp
from datetime import datetime
import json
import logging
from random import randbytes
from re import sub
import time
from unittest import expectedFailure
from substrateinterface import SubstrateInterface, Keypair

GIGABYTE = 1024*1024*1024

TIMEOUT_WAIT_FOR_BLOCK = 6
DEFAULT_SIGNER = "Alice"


class TfChainClient:
    def __init__(self):
        self._wait_for_finalization = False
        self._wait_for_inclusion = True
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

    def _setup_predefined_account(self, name, port=9945, create_twin=False):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        insert_key_params = [
            "tft!", f"//{name}", self._predefined_keys[name].public_key.hex()]
        substrate.rpc_request("author_insertKey", insert_key_params)

        insert_key_params = [
            "smct", f"//{name}", self._predefined_keys[name].public_key.hex()]
        substrate.rpc_request("author_insertKey", insert_key_params)

        if create_twin:
            self.user_accept_tc(port=port, who=name)
            self.create_twin(port=port, who=name)

    def _check_signer(self, who):
        assert isinstance(who, str), "who should be a string"
        _who = who.title()
        assert _who in self._predefined_keys, f"{who} is not a predefined account, use one of {self._predefined_keys.keys()}"
        return _who

    def _check_events(self, events, expected_events):
        assert isinstance(
            expected_events, list), "expected_events should be a list"
        for expected_event in expected_events:
            assert isinstance(
                expected_event, dict), "all expected_events should be dictionaries"

        logging.info("Events: %s", json.dumps(events))

        for expected_event in expected_events:
            check = False
            for event in events:
                check = all(item in event.keys(
                ) and event[item] == expected_event[item] for item in expected_event.keys())
                if check:
                    logging.info("Found event %s", expected_event)
                    break
            if not check:
                raise Exception(
                    f"Expected the event {expected_event} in {events}, no match found!")

    def _sign_extrinsic_submit_check_response(self, substrate, call, who, expected_events=[]):
        _who = who.title()
        assert _who in self._predefined_keys, f"{who} is not a predefined account, use one of {self._predefined_keys.keys()}"

        logging.info("Sending signed transaction: %s", call)
        signed_call = substrate.create_signed_extrinsic(
            call, self._predefined_keys[_who])

        response = substrate.submit_extrinsic(
            signed_call, wait_for_finalization=self._wait_for_finalization, wait_for_inclusion=self._wait_for_inclusion)
        if response.error_message:
            raise Exception(response.error_message)

        events = [event.value["event"] for event in response.triggered_events]
        self._check_events([event.value["event"]
                           for event in response.triggered_events], expected_events)

    def setup_alice(self, create_twin=False, port=9945):
        self._setup_predefined_account(
            "Alice", port=port, create_twin=create_twin)

    def setup_bob(self, create_twin=False, port=9946):
        self._setup_predefined_account(
            "Bob", port=port, create_twin=create_twin)

    def setup_charlie(self, create_twin=False, port=9947):
        self._setup_predefined_account(
            "Charlie", port=port, create_twin=create_twin)

    def setup_dave(self, create_twin=False, port=9948):
        self._setup_predefined_account(
            "Dave", port=port, create_twin=create_twin)

    def setup_eve(self, create_twin=False, port=9949):
        self._setup_predefined_account(
            "Eve", port=port, create_twin=create_twin)

    def setup_ferdie(self, create_twin=False, port=9950):
        self._setup_predefined_account(
            "Ferdie", port=port, create_twin=create_twin)

    def set_wait_for_finalization(self, value):
        assert isinstance(value, bool), "value should be a boolean"
        self._wait_for_finalization = value

    def user_accept_tc(self, port=9945, who=DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "user_accept_tc",
                                      {
                                          "document_link": "garbage",
                                          "document_hash": "garbage"
                                      })

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def create_twin(self, ip="::1", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(ip, str), "ip should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call(
            "TfgridModule", "create_twin", {"ip": ip})

        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinStored"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def update_twin(self, ip="::1", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(ip, str), "ip should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "update_twin", {
            "ip": ip})

        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinUpdated"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def delete_twin(self, twin_id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(twin_id, int), "twin_id should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_twin", {
            "twin_id": twin_id})
        
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinDeleted"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def get_twin(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "Twins", [id])
        return q.value

    def balance_data(self, twin_id=1, port=9945):
        assert isinstance(twin_id, int), "twin_id should be an integer"

        twin = self.get_twin(id=twin_id, port=port)
        if twin is None:
            raise Exception(f"The twin with id {twin_id} was not found.")

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        account_info = substrate.query(
            "System", "Account", [str(twin["account_id"])])
        assert account_info is not None, f"Failed fetching the account data for {str(twin['account_id'])}"
        assert "data" in account_info, f"Could not find balance data in the account info {account_info}"

        logging.info(dir(account_info["data"]))
        return account_info["data"].value

    def get_block_number(self, port=9945):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        q = substrate.query("System", "Number", [])
        return q.value

    def wait_x_blocks(self, x=1, port=9945):
        start_time = datetime.now()
        stop_at_block = self.get_block_number(port=port) + x
        logging.info("Waiting %s blocks. Current is %s", x, stop_at_block - x)
        timeout_for_x_blocks = TIMEOUT_WAIT_FOR_BLOCK * (x+1)
        while self.get_block_number(port=port) < stop_at_block:
            elapsed_time = datetime.now() - start_time
            if elapsed_time.total_seconds() >= timeout_for_x_blocks:
                raise Exception(f"Timeout on waiting for {x} blocks")
            time.sleep(6)

    def create_farm(self, name="myfarm", public_ips=[], port=9945, who=DEFAULT_SIGNER):
        assert isinstance(name, str), "name should be a string"
        assert isinstance(public_ips, list), "public_ips should be a list"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "create_farm",
                                      {
                                          "name": f"{name}",
                                          "public_ips": public_ips
                                      })
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmStored"
        }]

        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_farm(self, id=1, name="", pricing_policy_id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(id, int), "id should be an integer"
        assert isinstance(name, str), "name should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "update_farm",
                                      {
                                          "id": id,
                                          "name": f"{name}",
                                          "pricing_policy_id": pricing_policy_id
                                      })
        
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmUpdated"
        }]

        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def get_farm(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        q = substrate.query("TfgridModule", "Farms", [id])
        return q.value

    def add_farm_ip(self, id=1, ip="", gateway="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(id, int), "id should be an integer"
        assert isinstance(ip, str), "ip should be a string"
        assert isinstance(port, int), "port should be an integer"
        assert isinstance(gateway, str), "gateway should be a string"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "add_farm_ip",
                                      {
                                          "id": id,
                                          "ip": ip,
                                          "gateway": gateway
                                      })
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmUpdated"
        }]

        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def remove_farm_ip(self, id=1, ip="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(id, int), "id should be an integer"
        assert isinstance(ip, str), "ip should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "remove_farm_ip",
                                      {
                                          "id": id,
                                          "ip": ip
                                      })
        
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmUpdated"
        }]

        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def create_node(self, farm_id=1, hru=0, sru=0, cru=0, mru=0, longitude="", latitude="", country="", city="", interfaces=[],
                    secure_boot=False, virtualized=False, serial_number="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(farm_id, int), "farm_id should be an integer"
        assert isinstance(hru, int), "hru should be an integer"
        assert isinstance(sru, int), "sru should be an integer"
        assert isinstance(cru, int), "cru should be an integer"
        assert isinstance(mru, int), "mru should be an integer"
        assert isinstance(longitude, str), "longitude should be a string"
        assert isinstance(latitude, str), "latitude should be a string"
        assert isinstance(country, str), "country should be a string"
        assert isinstance(city, str), "city should be a string"
        assert isinstance(interfaces, list), "interfaces should be a list"
        assert isinstance(secure_boot, bool), "secure_boot should be a boolean"
        assert isinstance(virtualized, bool), "virtualized should be a boolean"
        assert isinstance(
            serial_number, str), "serial_number should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "farm_id": farm_id,
            "resources": {
                "hru": hru * GIGABYTE,
                "sru": sru * GIGABYTE,
                "cru": cru,
                "mru": mru * GIGABYTE
            },
            "location": {
                "longitude": f"{longitude}",
                "latitude": f"{latitude}"
            },
            "country": country,
            "city": city,
            "interfaces": interfaces,
            "secure_boot": secure_boot,
            "virtualized": virtualized,
            "serial_number": serial_number
        }

        call = substrate.compose_call(
            "TfgridModule", "create_node", params)

        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeStored"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_node(self, node_id=1, farm_id=1, hru=0, sru=0, cru=0, mru=0, longitude="", latitude="", country="", city="",
                    secure_boot=False, virtualized=False, serial_number="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(node_id, int), "node_id should be an integer"
        assert isinstance(farm_id, int), "farm_id should be an integer"
        assert isinstance(hru, int), "hru should be an integer"
        assert isinstance(sru, int), "sru should be an integer"
        assert isinstance(cru, int), "cru should be an integer"
        assert isinstance(mru, int), "mru should be an integer"
        assert isinstance(longitude, str), "longitude should be a string"
        assert isinstance(latitude, str), "latitude should be a string"
        assert isinstance(country, str), "country should be a string"
        assert isinstance(city, str), "city should be a string"
        assert isinstance(secure_boot, bool), "secure_boot should be a boolean"
        assert isinstance(virtualized, bool), "virtualized should be a boolean"
        assert isinstance(
            serial_number, str), "serial_number should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "node_id": node_id,
            "farm_id": farm_id,
            "resources": {
                "hru": hru * GIGABYTE,
                "sru": sru * GIGABYTE,
                "cru": cru,
                "mru": mru * GIGABYTE
            },
            "location": {
                "longitude": f"{longitude}",
                "latitude": f"{latitude}"
            },
            "country": country,
            "city": city,
            "interfaces": [],
            "secure_boot": secure_boot,
            "virtualized": virtualized,
            "serial_number": serial_number
        }

        call = substrate.compose_call(
            "TfgridModule", "update_node", params)

        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeUpdated"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def add_node_public_config(self, farm_id=1, node_id=1, ipv4="", gw4="", ipv6=None, gw6=None, domain=None, port=9945,
                               who=DEFAULT_SIGNER):
        assert isinstance(farm_id, int), "farm_id should be an integer"
        assert isinstance(node_id, int), "node_id should be an integer"
        assert isinstance(ipv4, str), "ipv4 should be a string"
        assert isinstance(gw4, str), "gw4 should be a string"
        assert ipv6 is None or isinstance(
            ipv6, str), "ipv6 should be None or a string"
        assert gw6 is None or isinstance(
            gw4, str), "gw6 should be None or a string"
        assert domain is None or isinstance(
            domain, str), "domain should be None or a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        ip4_config = {
            "ip": ipv4,
            "gw": gw4
        }

        ip6_config = None if ipv6 is None and gw6 is None else {
            "ip": ipv6,
            "gw": gw6
        }

        public_config = {
            "ip4": ip4_config,
            "ip6": ip6_config,
            "domain": domain
        }

        call = substrate.compose_call("TfgridModule", "add_node_public_config",
                                      {
                                          "farm_id": farm_id,
                                          "node_id": node_id,
                                          "public_config": public_config
                                      })
        
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodePublicConfigStored"
        }]

        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def delete_node(self, id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(id, int), "id should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_node", {
            "id": id})

        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeDeleted"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def get_node(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "Nodes", [id])
        return q.value

    def create_node_contract(self, node_id=1, deployment_data=randbytes(32), deployment_hash=randbytes(32), public_ips=0,
                             solution_provider_id=None, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(node_id, int), "node_id should be an integer"
        assert isinstance(public_ips, int), "public_ips should be an integer"
        assert solution_provider_id is None or isinstance(
            solution_provider_id, int), "solution_provider_id should be None or an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "node_id": node_id,
            "deployment_data": deployment_data,
            "deployment_hash": deployment_hash,
            "public_ips": public_ips,
            "solution_provider_id": solution_provider_id
        }

        call = substrate.compose_call(
            "SmartContractModule", "create_node_contract", params)

        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractCreated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_node_contract(self, contract_id=1, deployment_data=randbytes(32), deployment_hash=randbytes(32), port=9945, who=DEFAULT_SIGNER):
        assert isinstance(contract_id, int), "contract_id should be an integer"
        assert isinstance(
            deployment_data, bytes), "deployment_data should be bytes"
        assert isinstance(
            deployment_hash, bytes), "deployment_hash should be bytes"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "update_node_contract", {
            "contract_id": contract_id,
            "deployment_data": deployment_data,
            "deployment_hash": deployment_data
        })
        
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractUpdated"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def create_rent_contract(self, node_id, solution_provider_id=None, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(node_id, int), "node_id should be an integer"
        assert solution_provider_id is None or isinstance(
            solution_provider_id, int), "solution_provider_id should be None or an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "create_rent_contract",
                                      {
                                          "node_id": node_id,
                                          "solution_provider_id": solution_provider_id
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractCreated"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def create_name_contract(self, name="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(name, str), "name should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "create_name_contract",
                                      {
                                          "name": name
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractCreated"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def _cancel_contract(self, contract_id=1, type="Name", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(contract_id, int), "name should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "cancel_contract",
                                      {
                                          "contract_id": contract_id
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": f"{type}ContractCanceled"
        }]
        self._sign_extrinsic_submit_check_response(substrate, call, who, expected_events=expected_events)

    def cancel_name_contract(self, contract_id=1, port=9945, who=DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id, type="Name", port=port, who=who)
    
    def cancel_rent_contract(self, contract_id=1, port=9945, who=DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id, type="Rent", port=port, who=who)
    
    def cancel_node_contract(self, contract_id=1, port=9945, who=DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id, type="Node", port=port, who=who)    

    def report_contract_resources(self, contract_id=1, hru=0, sru=0, cru=0, mru=0, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(hru, int), "hru should be an integer"
        assert isinstance(sru, int), "sru should be an integer"
        assert isinstance(cru, int), "cru should be an integer"
        assert isinstance(mru, int), "mru should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "contract_resources": [{
                "contract_id": contract_id,
                "used": {
                    "hru": hru * GIGABYTE,
                    "sru": sru * GIGABYTE,
                    "cru": cru,
                    "mru": mru * GIGABYTE
                }
            }]
        }

        call = substrate.compose_call(
            "SmartContractModule", "report_contract_resources", params)
        
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "UpdatedUsedResources"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def events(self, port=9945):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("System", "Events", [])
        return q.value

"""
TODO:
add_validator
remove_validator
add_validator_again
add_stellar_payout_v2address
set_farm_certification
set_node_certification
report_uptime
create_entity
update_entity
delete_entity
add_twin_entity
delete_twin_entity
create_pricing_policy
update_pricing_policy
create_farming_policy
delete_node_farm
set_farm_dedicated
force_reset_farm_ip
set_connection_price
add_node_certifier
remove_node_certifier
update_farming_policy
attach_policy_to_farm
set_zos_version
add_reports
add_nru_reports
report_contract_resources
create_solution_provider
approve_solution_provider
add_bridge_validator
remove_bridge_validator
propose_or_vote_mint_transaction
propose_burn_transaction_or_add_sig
set_burn_transaction_executed
create_refund_transaction_or_add_sig
set_refund_transaction_executed
set_prices
set_allowed_origin
set_min_tft_price
set_max_tft_price
schedule
cancel
schedule_named
cancel_named
schedule_after
schedule_named_after
burn_tft
execute
propose
vote
close
disapprove_proposal
add_member
remove_member
swap_member
reset_members
change_key
set_prime
clear_prime
set_code
create_validator
activate_validator_node
change_validator_node_account
bond
approve_validator
remove_validator
propose
vote
veto
close
batch
as_derivative
batch_all
dispatch_as
force_batch
"""