*** Settings ***
Documentation    Suite for integration tests for testing billing
Library          SubstrateNetwork.py
Library          TfChainClient.py


*** Test Cases ***
Test Start And Stop Network
    Setup Multi Node Network    log_name=test_start_and_stop_network  amt=${4}

    Tear Down Multi Node Network

Test Create Twin
    Setup Multi Node Network  log_name=test_create_twin  amt=${2}

    Setup Alice
    User Accept Tc

    Create Twin  ip=::1
    ${twin} =  Get Twin  ${1}
    Should Be Equal  ${twin}[ip]  ::1

    Update Twin  ip=0000:0000:0000:0000:0000:0000:0000:0001
    ${twin} =  Get Twin  ${1}
    Should Be Equal  ${twin}[ip]  0000:0000:0000:0000:0000:0000:0000:0001

    Delete Twin  ${1}

    ${twin} =  Get Twin  ${1}
    Should Be Equal  ${twin}  ${None}

    Tear Down Multi Node Network

Test Create Farm
    Setup Multi Node Network  log_name=test_create_farm

    Setup Alice
    Setup Bob
    User Accept Tc
    Create Twin

    Create Farm  name=this_is_the_name_of_the_farm
    ${farm_before} =  Get Farm  ${1}
    Should Be Equal  ${farm_before}[name]  this_is_the_name_of_the_farm

    Update Farm  id=${1}  name=name_change  pricing_policy_id=1
    ${farm_after} =  Get Farm  ${1}
    Should Be Equal  ${farm_after}[name]  name_change

    Tear Down Multi Node Network

Test Create Node
    Setup Multi Node Network  log_name=test_billing  amt=${3}

    # Setup for billing
    Setup Alice
    Setup Bob
    Setup Charlie
    User Accept Tc
    Create Twin
    Create Farm  name=alice_farm
    Create Node  farm_id=${1}  hru=${1024}  sru=${512}  cru=${8}  mru=${16}  longitude=2.17403  latitude=41.40338  country=Belgium  city=Ghent
    ${node} =  Get Node  ${1}
    Should Be Equal  ${node}[city]  Ghent

    Update Node  node_id=${1}  farm_id=${1}  hru=${1024}  sru=${512}  cru=${8}  mru=${16}  longitude=2.17403  latitude=41.40338  country=Belgium  city=Celles
    ${node} =  Get Node  ${1}
    Should Be Equal  ${node}[city]  Celles

    Delete Node  ${1}
    ${node} =  Get Node  ${1}
    Should Be Equal  ${node}  ${None}

    Tear Down Multi Node Network


Test Billing
    Setup Multi Node Network  log_name=test_billing  amt=${3}

    # Setup for billing
    Setup Alice
    Setup Bob
    Setup Charlie
    User Accept Tc
    Create Twin
    Create Farm  name=alice_farm
    Create Node  farm_id=${1}  hru=${1024}  sru=${512}  cru=${8}  mru=${16}  longitude=2.17403  latitude=41.40338  country=Belgium  city=Ghent
    Create Node Contract  node_id=${1}
    Report Contract Resources  contract_id=${1}  hru=${0}  sru=${20}  cru=${2}  mru=${4}

    # let it run for some time
    Sleep  10s

    Tear Down Multi Node Network