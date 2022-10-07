from datetime import datetime
import json
import logging
from random import randbytes
import time

from SubstrateNetwork import PREDEFINED_KEYS
from substrateinterface import SubstrateInterface, Keypair

GIGABYTE = 1024*1024*1024

TIMEOUT_WAIT_FOR_BLOCK = 6

DEFAULT_SIGNER = "Alice"
DEFAULT_PORT = 9945

FARM_CERTIFICATION_NOTCERTIFIED = "NotCertified"
FARM_CERTIFICATION_GOLD = "Gold"
FARM_CERTIFICATION_TYPES = [
    FARM_CERTIFICATION_NOTCERTIFIED, FARM_CERTIFICATION_GOLD]

NODE_CERTIFICATION_DIY = "Diy"
NODE_CERTIFICATION_CERTIFIED = "Certified"
NODE_CERTIFICATION_TYPES = [
    NODE_CERTIFICATION_DIY, NODE_CERTIFICATION_CERTIFIED]

UNIT_BYTES = "Bytes"
UNIT_KILOBYTES = "Kilobytes"
UNIT_MEGABYTES = "Mebabytes"
UNIT_GIGABYTES = "Gigabytes"
UNIT_TERRABYTES = "Terrabytes"
UNIT_TYPES = [UNIT_BYTES, UNIT_KILOBYTES,
              UNIT_MEGABYTES, UNIT_GIGABYTES, UNIT_TERRABYTES]


class TfChainClient:
    def __init__(self):
        self._setup()

    def _setup(self):
        self._wait_for_finalization = False
        self._wait_for_inclusion = True
        self._pallets_offchain_workers = ["tft!", "smct"]

    def _connect_to_server(self, url: str):
        return SubstrateInterface(url=url, ss58_format=42, type_registry_preset='polkadot')

    def _check_events(self, events: list = [], expected_events: list = []):
        logging.info("Events: %s", json.dumps(events))

        # This was a sudo call that failed
        for event in events:
            if event["event_id"] == "Sudid" and "Err" in event["attributes"]:
                raise Exception(event["attributes"])

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

    def _sign_extrinsic_submit_check_response(self, substrate, call, who: str, expected_events: list = []):
        _who = who.title()
        if _who == "Sudo":
            call = substrate.compose_call("Sudo", "sudo", {
                "call": call
            })
            _who = "Alice"
        else:
            assert _who in PREDEFINED_KEYS.keys(
            ), f"{who} is not a predefined account, use one of {PREDEFINED_KEYS.keys()}"

        logging.info("Sending signed transaction: %s", call)
        signed_call = substrate.create_signed_extrinsic(
            call, PREDEFINED_KEYS[_who])

        response = substrate.submit_extrinsic(
            signed_call, wait_for_finalization=False, wait_for_inclusion=True)
        logging.info("Reponse is %s", response)
        if response.error_message:
            raise Exception(response.error_message)

        self._check_events([event.value["event"]
                           for event in response.triggered_events], expected_events)

    def setup_predefined_account(self, who: str, port: int = DEFAULT_PORT):
        logging.info("Setting up predefined account %s (%s)", who,
                     PREDEFINED_KEYS[who].ss58_address)
        self.user_accept_tc(port=port, who=who)
        self.create_twin(port=port, who=who)

    def user_accept_tc(self, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "user_accept_tc",
                                      {
                                          "document_link": "garbage",
                                          "document_hash": "garbage"
                                      })
        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def create_twin(self, ip: str = "::1", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call(
            "TfgridModule", "create_twin", {"ip": ip})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinStored"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_twin(self, ip: str = "::1", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "update_twin", {
            "ip": ip})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinUpdated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def delete_twin(self, twin_id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_twin", {
            "twin_id": twin_id})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "TwinDeleted"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_twin(self, id: int = 1, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "Twins", [id])
        logging.info(q.value)
        return q.value

    def balance_data(self, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        assert who in PREDEFINED_KEYS.keys(
        ), f"{who} is not a predefined account"
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        account_info = substrate.query(
            "System", "Account", [PREDEFINED_KEYS[who].ss58_address])
        assert account_info is not None, f"Failed fetching the account data for {who} ({PREDEFINED_KEYS[who].ss58_address})"
        assert "data" in account_info, f"Could not find balance data in the account info {account_info}"

        logging.info(account_info)
        return account_info["data"].value

    def get_block_number(self, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        q = substrate.query("System", "Number", [])
        return q.value

    def wait_x_blocks(self, x: int = 1, port: int = DEFAULT_PORT):
        block_to_wait_for = self.get_block_number(port=port) + x
        self.wait_till_block(block_to_wait_for, port=port)

    def wait_till_block(self, x: int = 1, port: int = DEFAULT_PORT):
        start_time = datetime.now()
        current_block = self.get_block_number(port=port)
        logging.info("Waiting till block %s. Current is %s", x, current_block)
        timeout_for_x_blocks = TIMEOUT_WAIT_FOR_BLOCK * (x-current_block+1)
        while self.get_block_number(port=port) < x:
            elapsed_time = datetime.now() - start_time
            if elapsed_time.total_seconds() >= timeout_for_x_blocks:
                raise Exception(f"Timeout on waiting for {x} blocks")
            time.sleep(6)

    def create_farm(self, name: str = "myfarm", public_ips: list = [], port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
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

    def update_farm(self, id: int = 1, name: str = "", pricing_policy_id: int = 1, port: int = DEFAULT_PORT,
                    who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_farm(self, id: int = 1, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")
        q = substrate.query("TfgridModule", "Farms", [id])
        return q.value

    def add_farm_ip(self, id: int = 1, ip: str = "", gateway: str = "", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def remove_farm_ip(self, id: int = 1, ip: str = "", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def create_node(self, farm_id: int = 1, hru: int = 0, sru: int = 0, cru: int = 0, mru: int = 0,
                    longitude: str = "", latitude: str = "", country: str = "", city: str = "", interfaces: list = [],
                    secure_boot: bool = False, virtualized: bool = False, serial_number: str = "", port: int = DEFAULT_PORT,
                    who: str = DEFAULT_SIGNER):
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

    def update_node(self, node_id: int = 1, farm_id: int = 1, hru: int = 0, sru: int = 0, cru: int = 0, mru: int = 0,
                    longitude: str = "", latitude: str = "", country: str = "", city: str = "",
                    secure_boot: bool = False, virtualized: bool = False, serial_number: str = "", port: int = DEFAULT_PORT,
                    who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def add_node_public_config(self, farm_id: int = 1, node_id: int = 1, ipv4: str = "", gw4: str = "",
                               ipv6: str | None = None, gw6: str | None = None, domain: str | None = None,
                               port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def delete_node(self, id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_node", {
            "id": id})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeDeleted"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_node(self, id: int = 1, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "Nodes", [id])
        return q.value

    def create_deployment_contract(self, farm_id: int = 1, deployment_data: bytes = randbytes(32),
                             deployment_hash: bytes = randbytes(32), public_ips: int = 0, hru: int = 0, sru: int = 0,
                             cru: int = 0, mru: int = 0, solution_provider_id: int | None = None, 
                             rent_contract_id: int | None = None, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "farm_id": farm_id,
            "deployment_data": deployment_data,
            "deployment_hash": deployment_hash,
            "public_ips": public_ips,
            "resources": {
                "hru": hru * GIGABYTE,
                "sru": sru * GIGABYTE,
                "cru": cru,
                "mru": mru * GIGABYTE
            },
            "solution_provider_id": solution_provider_id,
            "rent_contract_id": rent_contract_id
        }
        call = substrate.compose_call(
            "SmartContractModule", "create_deployment_contract", params)
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractCreated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_deployment_contract(self, contract_id: int = 1, deployment_data: bytes = randbytes(32),
                             deployment_hash: bytes = randbytes(32), hru: int = 0, sru: int = 0, cru: int = 0,
                             mru: int = 0, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "update_deployment_contract", {
            "contract_id": contract_id,
            "deployment_data": deployment_data,
            "deployment_hash": deployment_hash,
            "extra_resources": None if hru == 0 and sru == 0 and cru == 0 and mru == 0 else {
                "hru": hru * GIGABYTE,
                "sru": sru * GIGABYTE,
                "cru": cru,
                "mru": mru * GIGABYTE
            }
        })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractUpdated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def create_rent_contract(self, node_id: int = 1, solution_provider_id: int | None = None, port: int = DEFAULT_PORT,
                             who: str = DEFAULT_SIGNER):
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
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def create_name_contract(self, name: str = "", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "create_name_contract",
                                      {
                                          "name": name
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "ContractCreated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def _cancel_contract(self, contract_id: int = 1, type: str = "Name", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "cancel_contract",
                                      {
                                          "contract_id": contract_id
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": f"{type}ContractCanceled"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def cancel_name_contract(self, contract_id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id,
                              type="Name", port=port, who=who)

    def cancel_rent_contract(self, contract_id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id,
                              type="Rent", port=port, who=who)

    def cancel_node_contract(self, contract_id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        self._cancel_contract(contract_id=contract_id,
                              type="Node", port=port, who=who)

    def add_nru_reports(self, contract_id: int = 1, nru: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        block_number = self.get_block_number(port=port)
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        reports = [{
            "contract_id": contract_id,
            "nru": nru * GIGABYTE,
            "timestamp": block_number,
            "window": 6 * block_number
        }]
        call = substrate.compose_call(
            "SmartContractModule", "add_nru_reports", {"reports": reports})
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "NruConsumptionReportReceived"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def add_stellar_payout_v2address(self, farm_id: int = 1, stellar_address: str = "", port: int = DEFAULT_PORT,
                                     who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "farm_id": farm_id,
            "stellar_address": stellar_address
        }
        call = substrate.compose_call(
            "TfgridModule", "add_stellar_payout_v2address", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmPayoutV2AddressRegistered"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_farm_payout_v2address(self, farm_id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query(
            "TfgridModule", "FarmPayoutV2AddressByFarmID", [farm_id])
        return q.value

    def set_farm_certification(self, farm_id: int = 1, certification: str = FARM_CERTIFICATION_NOTCERTIFIED,
                               port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "farm_id": farm_id,
            "certification": f"{certification}"
        }
        call = substrate.compose_call(
            "TfgridModule", "set_farm_certification", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmCertificationSet"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def set_node_certification(self, node_id: int = 1, certification: str = NODE_CERTIFICATION_DIY, port: int = DEFAULT_PORT,
                               who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "node_id": node_id,
            "node_certification": f"{certification}"
        }
        call = substrate.compose_call(
            "TfgridModule", "set_node_certification", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeCertificationSet"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def add_node_certifier(self, account_name: str = "", port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "add_node_certifier", {
                                      "who": f"{PREDEFINED_KEYS[account_name].ss58_address}"})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeCertifierAdded"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def remove_node_certifier(self, account_name: str = "", port: int = DEFAULT_PORT, who=DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "remove_node_certifier", {
                                      "who": f"{PREDEFINED_KEYS[account_name].ss58_address}"})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeCertifierRemoved"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def report_uptime(self, uptime: int, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call(
            "TfgridModule", "report_uptime", {"uptime": uptime})
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "NodeUptimeReported"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def create_pricing_policy(self, name: str = "", unit: str = UNIT_GIGABYTES, su: int = 0, cu: int = 0, nu: int = 0,
                              ipu: int = 0, unique_name: int = "", domain_name: int = "",
                              foundation_account: str = "", certified_sales_account: str = "",
                              discount_for_dedication_nodes: int = 0, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "name": f"{name}",
            "su": {"value": su, "unit": unit},
            "cu": {"value": cu, "unit": unit},
            "nu": {"value": nu, "unit": unit},
            "ipu": {"value": ipu, "unit": unit},
            "unique_name": {"value": unique_name, "unit": unit},
            "domain_name": {"value": domain_name, "unit": unit},
            "foundation_account": f"{PREDEFINED_KEYS[foundation_account].ss58_address}",
            "certified_sales_account": f"{PREDEFINED_KEYS[certified_sales_account].ss58_address}",
            "discount_for_dedication_nodes": discount_for_dedication_nodes
        }
        call = substrate.compose_call(
            "TfgridModule", "create_pricing_policy", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "PricingPolicyStored"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_pricing_policy(self, id: int = 1, name: str = "", unit: str = UNIT_GIGABYTES, su: int = 0, cu: int = 0,
                              nu: int = 0, ipu: int = 0, unique_name: int = "", domain_name: int = "",
                              foundation_account: str = "", certified_sales_account: str = "",
                              discount_for_dedication_nodes: int = 0, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "id": id,
            "name": f"{name}",
            "su": {"value": su, "unit": unit},
            "cu": {"value": cu, "unit": unit},
            "nu": {"value": nu, "unit": unit},
            "ipu": {"value": ipu, "unit": unit},
            "unique_name": {"value": unique_name, "unit": unit},
            "domain_name": {"value": domain_name, "unit": unit},
            "foundation_account": f"{PREDEFINED_KEYS[foundation_account].ss58_address}",
            "certified_sales_account": f"{PREDEFINED_KEYS[certified_sales_account].ss58_address}",
            "discount_for_dedication_nodes": discount_for_dedication_nodes
        }
        call = substrate.compose_call(
            "TfgridModule", "update_pricing_policy", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "PricingPolicyStored"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_pricing_policy(self, id: int = 1, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "PricingPolicies", [id])
        return q.value

    def create_farming_policy(self, name: str = "", su: int = 0, cu: int = 0, nu: int = 0, ipv4: int = 0,
                              minimal_uptime: int = 0, policy_end: int = 0, immutable: bool = False,
                              default: bool = False, node_certification: str = NODE_CERTIFICATION_DIY,
                              farm_certification: str = FARM_CERTIFICATION_NOTCERTIFIED, port: int = DEFAULT_PORT,
                              who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "name": f"{name}",
            "su": su,
            "cu": cu,
            "nu": nu,
            "ipv4": ipv4,
            "minimal_uptime": minimal_uptime,
            "policy_end": policy_end,
            "immutable": immutable,
            "default": default,
            "node_certification": f"{node_certification}",
            "farm_certification": f"{farm_certification}"
        }
        call = substrate.compose_call(
            "TfgridModule", "create_farming_policy", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmingPolicyStored"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def update_farming_policy(self, id: int = 1, name: str = "", su: int = 0, cu: int = 0, nu: int = 0, ipv4: int = 0,
                              minimal_uptime: int = 0, policy_end: int = 0, immutable: bool = False, default: bool = False,
                              node_certification: str = NODE_CERTIFICATION_DIY,
                              farm_certification: str = FARM_CERTIFICATION_NOTCERTIFIED, port: int = DEFAULT_PORT,
                              who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        params = {
            "id": id,
            "name": f"{name}",
            "su": su,
            "cu": cu,
            "nu": nu,
            "ipv4": ipv4,
            "minimal_uptime": minimal_uptime,
            "policy_end": policy_end,
            "immutable": immutable,
            "default": default,
            "node_certification": f"{node_certification}",
            "farm_certification": f"{farm_certification}"
        }
        call = substrate.compose_call(
            "TfgridModule", "update_farming_policy", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmingPolicyUpdated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_farming_policy(self, id: int = 1, port: int = DEFAULT_PORT):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("TfgridModule", "FarmingPoliciesMap", [id])
        return q.value

    def attach_policy_to_farm(self, farm_id: int = 1, farming_policy_id: int | None = None, cu: int | None = None,
                              su: int | None = None, end: int | None = None, node_count: int | None = 0,
                              node_certification: bool = False, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        limits = {
            "farming_policy_id": farming_policy_id,
            "cu": cu,
            "su": su,
            "end": end,
            "node_count": node_count,
            "node_certification": node_certification
        }
        params = {
            "farm_id": farm_id,
            "limits": limits if farming_policy_id is not None else None
        }
        call = substrate.compose_call(
            "TfgridModule", "attach_policy_to_farm", params)
        expected_events = [{
            "module_id": "TfgridModule",
            "event_id": "FarmingPolicySet"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def create_solution_provider(self, description: str = "", link: str = "", providers: dict = {}, port: int = DEFAULT_PORT,
                                 who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        providers = [{"who": PREDEFINED_KEYS[who].ss58_address,
                      "take": take} for who, take in providers.items()]
        call = substrate.compose_call("SmartContractModule", "create_solution_provider",
                                      {
                                          "description": f"{description}",
                                          "link": f"{link}",
                                          "providers": providers
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "SolutionProviderCreated"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)

    def get_solution_provider(self, id: int = 1, port: int = DEFAULT_PORT, who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        q = substrate.query("SmartContractModule", "SolutionProviders", [id])
        return q.value

    def approve_solution_provider(self, solution_provider_id: int = 1, approve: bool = True, port: int = DEFAULT_PORT,
                                  who: str = DEFAULT_SIGNER):
        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "approve_solution_provider",
                                      {
                                          "solution_provider_id": solution_provider_id,
                                          "approve": approve
                                      })
        expected_events = [{
            "module_id": "SmartContractModule",
            "event_id": "SolutionProviderApproved"
        }]
        self._sign_extrinsic_submit_check_response(
            substrate, call, who, expected_events=expected_events)
