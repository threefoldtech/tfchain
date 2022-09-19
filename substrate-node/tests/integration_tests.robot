*** Settings ***
Documentation       Suite for integration tests on tfchain
Library             Collections
Library             SubstrateNetwork.py
Library             TfChainClient.py


*** Keywords ***
Public Ips Should Contain Ip
    [Arguments]    ${list}    ${ip}

    FOR    ${pub_ip_config}    IN    @{list}
        IF    "${pub_ip_config}[ip]" == "${ip}"
            Return From Keyword    
        END
    END
    
    Fail    msg=The list of public ips ${list} does not contain ip ${ip}

Public Ips Should Not Contain Ip
    [Arguments]    ${list }    ${ip}
    ${status} =    Run Keyword And Return Status    Public Ips Should Contain Ip    ${list}    ${ip}

    Run Keyword If    ${status}    Fail    The list of public ips ${list} contains the ip ${ip}, it shouldn't!


Setup Network And Create Farm
    [Documentation]    Helper function to quickly create a network with 2 nodes and creating a farm using Alice's key
    Setup Alice    create_twin=${True}
    Setup Bob    create_twin=${True}
    Create Farm    name=alice_farm

Setup Network And Create Node
    [Documentation]    Helper function to quickly create a network with 2 nodes and creating a farm and a node using Alice's key
    Setup Network And Create Farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent

Create Interface
    [Arguments]    ${name}    ${mac}    ${ips}
    ${dict} =     Create Dictionary    name    ${name}    mac    ${mac}    ips    ${ips}

    [Return]    ${dict}


*** Test Cases ***
Test Start And Stop Network
    [Documentation]     Starts and immediately stops the network once correctly started
    Setup Multi Node Network    log_name=test_start_stop_network    amt=${4}

    Tear Down Multi Node Network

Test Create Update Delete Twin
    [Documentation]    Testing api calls for managing twins
    Setup Multi Node Network    log_name=test_create_update_delete_twin    amt=${2}

    Setup Alice
    User Accept Tc

    Create Twin    ip=::1
    ${twin} =    Get Twin    ${1}
    Should Not Be Equal    ${twin}    ${None}
    Should Be Equal    ${twin}[ip]    ::1

    Update Twin    ip=0000:0000:0000:0000:0000:0000:0000:0001
    ${twin} =    Get Twin    ${1}
    Should Not Be Equal    ${twin}    ${None}
    Should Be Equal    ${twin}[ip]    0000:0000:0000:0000:0000:0000:0000:0001

    Delete Twin    ${1}

    ${twin} =    Get Twin    ${1}
    Should Be Equal    ${twin}    ${None}

    Tear Down Multi Node Network

Test Create Update Farm
    [Documentation]    Testing api calls for managing farms
    Setup Multi Node Network    log_name=test_create_update_farm

    Setup Alice    create_twin=${True}
    Setup Bob    create_twin=${True}
 
    Create Farm    name=this_is_the_name_of_the_farm
    ${farm_before} =    Get Farm    ${1}
    Should Not Be Equal    ${farm_before}    ${None}
    Should Be Equal    ${farm_before}[name]    this_is_the_name_of_the_farm

    Update Farm    id=${1}    name=name_change    pricing_policy_id=1
    ${farm_after} =    Get Farm    ${1}
    Should Not Be Equal    ${farm_after}    ${None}
    Should Be Equal    ${farm_after}[name]    name_change

    Tear Down Multi Node Network

Test Add Remove Public Ips
    [Documentation]    Testing api calls for managing public ips
    Setup Multi Node Network    log_name=test_add_remove_pub_ips

    Setup Network And Create Farm

    # Add an ip to the farm
    Add Farm Ip    id=${1}    ip=185.206.122.125/16    gateway=185.206.122.1
    ${farm} =    Get Farm    ${1}
    Should Not Be Equal    ${farm}    ${None}
    Public Ips Should Contain Ip    ${farm}[public_ips]    185.206.122.125/16

    # Remove the ip that we added
    Remove Farm Ip    id=${1}    ip=185.206.122.125/16
    ${farm} =    Get Farm    ${1}
    Should Not Be Equal    ${farm}    ${None}
    Public Ips Should Not Contain Ip    ${farm}[public_ips]    185.206.122.125/16

Test Add Public Ips: Failure InvalidPublicIP
    [Documentation]    Testing api calls for managing public ips
    Setup Multi Node Network    log_name=test_add_pub_ips_failure_invalidpubip

    Setup Network And Create Farm
    # Add an ip in an invalid format
    Run Keyword And Expect Error    *'InvalidPublicIP'*
    ...    Add Farm Ip    id=${1}    ip="185.206.122.125"    gateway=185.206.122.1

    Tear Down Multi Node Network    
    

Test Create Update Delete Node
    [Documentation]    Testing api calls for managing nodes
    Setup Multi Node Network    log_name=test_create_update_delet_node    amt=${3}

    Setup Network And Create Farm

    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent
    ${node} =    Get Node    ${1}
    Should Not Be Equal    ${node}    ${None}
    Should Be Equal    ${node}[city]    Ghent

    Update Node    node_id=${1}    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Celles
    ${node} =    Get Node    ${1}
    Should Not Be Equal    ${node}    ${None}
    Should Be Equal    ${node}[city]    Celles

    Delete Node    ${1}
    ${node} =    Get Node    ${1}
    Should Be Equal    ${node}    ${None}

    Tear Down Multi Node Network

Test Add Public Config On Node: Success
    [Documentation]    Testing adding a public config on a node
    Setup Multi Node Network    log_name=test_add_pub_config_node    amt=${2}

    Setup Network And Create Node

    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33/24    gw4=185.206.122.1    ipv6=2a10:b600:1::0cc4:7a30:65b5/64    gw6=2a10:b600:1::1    domain=some-domain
    ${node} =     Get Node    ${1}
    Should Not Be Equal    ${node}    ${None}
    Should Not Be Equal    ${node}[public_config]    ${None}
    Should Not Be Equal    ${node}[public_config][ip4]    ${None}
    Should Be Equal    ${node}[public_config][ip4][ip]  185.206.122.33/24
    Should Be Equal    ${node}[public_config][ip4][gw]    185.206.122.1
    Should Not Be Equal    ${node}[public_config][ip6]    ${None}
    Should Be Equal    ${node}[public_config][ip6][ip]    2a10:b600:1::0cc4:7a30:65b5/64
    Should Be Equal    ${node}[public_config][ip6][gw]    2a10:b600:1::1
    Should Be Equal    ${node}[public_config][domain]    some-domain

    Tear Down Multi Node Network

Test Add Public Config On Node: Failure InvalidIP4
    [Documentation]    Testing adding a public config on a node with an invalid ipv4
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_ipv4    amt=${2}

    Setup Network And Create Node

    Run Keyword And Expect Error    *'InvalidIP4'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33    gw4=185.206.122.1    domain=some-domain

    Tear Down Multi Node Network

Test Add Public Config On Node: Failure InvalidIP6
    [Documentation]    Testing adding a public config on a node with an invalid ipv6
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_ipv6    amt=${2}

    Setup Network And Create Node  

    Run Keyword And Expect Error    *'InvalidIP6'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33/24    gw4=185.206.122.1    ipv6=2a10:b600:1::0cc4:7a30:65b5    gw6=2a10:b600:1::1    domain=some-domain
    
    Tear Down Multi Node Network


Test Add Public Config On Node: Failure InvalidDomain
    [Documentation]    Testing adding a public config on a node with an invalid domain
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_invaliddomain    amt=${2}

    Setup Network And Create Node
    Run Keyword And Expect Error    *'InvalidDomain'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33/24    gw4=185.206.122.1    ipv6=2a10:b600:1::0cc4:7a30:65b5/64    gw6=2a10:b600:1::1    domain=some_invalid_domain

    Tear Down Multi Node Network

Test Create Update Cancel Node Contract: Success
    [Documentation]    Testing creating a node contract
    Setup Multi Node Network    log_name=test_create_node_contract    amt=${2}

    Setup Alice    create_twin=${True}
    Setup Bob    create_twin=${True}
    ${ip_1} =     Create Dictionary    ip    185.206.122.33/24    gw    185.206.122.1
    ${public_ips} =    Create List    ${ip_1}
    Create Farm    name=alice_farm    public_ips=${public_ips}
    ${interface_ips} =     Create List    10.2.3.3
    ${interface_1} =     Create Interface    name=zos    mac=00:00:5e:00:53:af    ips=${interface_ips}
    ${interfaces} =    Create List    ${interface_1}
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}   longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent    interfaces=${interfaces}
    
    # Bob is the one creating the contract and thus being billed
    Create Node Contract    node_id=${1}    public_ips=${1}    who=Bob    port=9946

    Update Node Contract    contract_id=${1}    who=Bob    port=9946

    Cancel Contract    contract_id=${1}    who=Bob    port=9946

    Tear Down Multi Node Network

Test Create Node Contract: Failure Not Enough Public Ips
    [Documentation]    Testing creating a node contract and requesting too much pub ips
    Setup Multi Node Network    log_name=test_create_node_contract_failure_notenoughpubips    amt=${2}

    # the function below creates a farm containing 0 public ips and a node with 0 configured interfaces
    Setup Network And Create Node
    # let's request 2 public ips which should result in an error
    Run Keyword And Expect Error    *'FarmHasNotEnoughPublicIPs'*
    ...    Create Node Contract    node_id=${1}    public_ips=${2}

    Tear Down Multi Node Network

Test Create Rent Contract: Success
    [Documentation]    Testing creating a rent contract
    Setup Multi Node Network    log_name=test_create_rent_contract    amt=${2}

    Setup Network And Create Node
    
    Create Rent Contract    node_id=${1}

    Cancel Contract    contract_id=${1}

    Tear Down Multi Node Network

Test Create Name Contract: Success
    [Documentation]    Testing creating a name contract
    Setup Multi Node Network    log_name=test_create_name_contract    amt=${2}

    Setup Network And Create Node

    Create Name Contract    name=my_name_contract

    Cancel Contract    contract_id=${1}

    Tear Down Multi Node Network

Test Billing
    [Documentation]    Sets up a network of 3 nodes. Alice creates a twin, a farm, a node, a node contract and reports contract resources. The test will run for some time with the goal to go through the billing process
    Setup Multi Node Network    log_name=test_billing    amt=${3}

    Setup Alice    create_twin=${True}
    Setup Bob    create_twin=${True}
    Setup Charlie    create_twin=${True}
    Create Farm    name=alice_farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent
    Create Node Contract    node_id=${1}
    Report Contract Resources    contract_id=${1}    hru=${0}    sru=${20}    cru=${2}    mru=${4}

    # let it run for some time
    Sleep    10s

    Tear Down Multi Node Network
