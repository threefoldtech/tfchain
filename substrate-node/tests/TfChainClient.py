import logging
from random import randbytes
from re import sub
from venv import create
from xml import dom
from xmlrpc.client import boolean
from substrateinterface import SubstrateInterface, Keypair

GIGABYTE = 1024*1024*1024

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

    def _sign_extrinsic_submit_check_response(self, substrate, call, who):
        _who = who.title()
        assert _who in self._predefined_keys, f"{who} is not a predefined account, use one of {self._predefined_keys.keys()}"

        logging.info("Sending signed transaction: %s", call)
        signed_call = substrate.create_signed_extrinsic(
            call, self._predefined_keys[_who])

        response = substrate.submit_extrinsic(
            signed_call, wait_for_finalization=self._wait_for_finalization, wait_for_inclusion=self._wait_for_inclusion)
        logging.info("Response: %s", response)
        if response.error_message:
            raise Exception(response.error_message)

    def setup_alice(self, create_twin=False, port=9945):
        self._setup_predefined_account("Alice", port=port, create_twin=create_twin)

    def setup_bob(self, create_twin=False, port=9946):
        self._setup_predefined_account("Bob", port=port, create_twin=create_twin)

    def setup_charlie(self, create_twin=False, port=9947):
        self._setup_predefined_account("Charlie", port=port, create_twin=create_twin)

    def setup_dave(self, create_twin=False, port=9948):
        self._setup_predefined_account("Dave", port=port, create_twin=create_twin)

    def setup_eve(self, create_twin=False, port=9949):
        self._setup_predefined_account("Eve", port=port, create_twin=create_twin)

    def setup_eve(self, create_twin=False, port=9950):
        self._setup_predefined_account("Ferdie", port=port, create_twin=create_twin)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def update_twin(self, ip="::1", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(ip, str), "ip should be a string"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "update_twin", {
            "ip": ip})

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def delete_twin(self, twin_id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(twin_id, int), "twin_id should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_twin", {
            "twin_id": twin_id})

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def get_twin(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        return substrate.query("TfgridModule", "Twins", [id])

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def get_farm(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        return substrate.query("TfgridModule", "Farms", [id])

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def delete_node(self, id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(id, int), "id should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("TfgridModule", "delete_node", {
            "id": id})

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def get_node(self, id=1, port=9945):
        assert isinstance(id, int), "id should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        return substrate.query("TfgridModule", "Nodes", [id])

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def create_name_contract(self, name="", port=9945, who=DEFAULT_SIGNER):
        assert isinstance(name, str), "name should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "create_name_contract",
                                      {
                                          "name": name
                                      })

        self._sign_extrinsic_submit_check_response(substrate, call, who)

    def cancel_contract(self, contract_id=1, port=9945, who=DEFAULT_SIGNER):
        assert isinstance(contract_id, int), "name should be an integer"
        assert isinstance(port, int), "port should be an integer"

        substrate = self._connect_to_server(f"ws://127.0.0.1:{port}")

        call = substrate.compose_call("SmartContractModule", "cancel_contract",
                                      {
                                          "contract_id": contract_id
                                      })

        self._sign_extrinsic_submit_check_response(substrate, call, who)

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

        self._sign_extrinsic_submit_check_response(substrate, call, who)
