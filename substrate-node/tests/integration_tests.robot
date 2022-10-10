*** Settings ***
Documentation      Suite for integration tests on tfchain
Library            Collections
Library            SubstrateNetwork.py
Library            TfChainClient.py
Library            OperatingSystem


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
    Setup Predefined Account    who=Alice
    Setup Predefined Account    who=Bob
    Create Farm    name=alice_farm

Setup Network And Create Node
    [Documentation]    Helper function to quickly create a network with 2 nodes and creating a farm and a node using Alice's key
    Setup Network And Create Farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent

Create Interface
    [Arguments]    ${name}    ${mac}    ${ips}
    ${dict} =     Create Dictionary    name    ${name}    mac    ${mac}    ips    ${ips}
    [Return]    ${dict}

Ensure Account Balance Increased
    [Arguments]    ${balance_before}    ${balance_after}
    IF    ${balance_before}[free] >= ${balance_after}[free]-${balance_after}[fee_frozen]
        Fail    msg=It looks like the billing did not take place.
    END

Ensure Account Balance Decreased
    [Arguments]    ${balance_before}    ${balance_after}
    IF    ${balance_before}[free] <= ${balance_after}[free]-${balance_after}[fee_frozen]
        Fail    msg=It looks like the billing did not take place.
    END



*** Test Cases ***
Test Start And Stop Network
    [Documentation]     Starts and immediately stops the network (4 nodes) once correctly started
    Setup Multi Node Network    log_name=test_start_stop_network    amt=${4}

    Tear Down Multi Node Network

Test Create Update Delete Twin
    [Documentation]    Testing api calls (create, update, delete) for managing twins
    Setup Multi Node Network    log_name=test_create_update_delete_twin

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
    [Documentation]    Testing api calls (create, update) for managing farms
    Setup Multi Node Network    log_name=test_create_update_farm

    Setup Predefined Account    who=Alice
    
    Create Farm    name=this_is_the_name_of_the_farm
    ${farm_before} =    Get Farm    ${1}
    Should Not Be Equal    ${farm_before}    ${None}
    Should Be Equal    ${farm_before}[name]    this_is_the_name_of_the_farm

    Update Farm    id=${1}    name=name_change    pricing_policy_id=1
    ${farm_after} =    Get Farm    ${1}
    Should Not Be Equal    ${farm_after}    ${None}
    Should Be Equal    ${farm_after}[name]    name_change

    Tear Down Multi Node Network

Test Add Stellar Payout V2ADDRESS 
    [Documentation]    Testing adding a stellar payout address
    Setup Multi Node Network    log_name=test_add_stellar_address

    Setup Network And Create Farm

    Add Stellar Payout V2address    farm_id=${1}    stellar_address=address
    ${payout_address} =    Get Farm Payout V2address    farm_id=${1}
    Should Be Equal    ${payout_address}    address

    Add Stellar Payout V2address    farm_id=${1}    stellar_address=changed address
    ${payout_address} =    Get Farm Payout V2address    farm_id=${1}
    Should Be Equal    ${payout_address}    changed address

    Run Keyword And Expect Error    *'CannotUpdateFarmWrongTwin'*
    ...    Add Stellar Payout V2address    farm_id=${1}    who=Bob

    Tear Down Multi Node Network

Test Set Farm Certification
    [Documentation]    Testing setting a farm certification
    Setup Multi Node Network    log_name=test_farm_certification

    Setup Network And Create Farm

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Set Farm Certification    farm_id=${1}    certification=Gold

    Set Farm Certification    farm_id=${1}    certification=Gold    who=sudo

    Tear Down Multi Node Network

Test Set Node Certification
    [Documentation]    Testing setting a node certification
    Setup Multi Node Network    log_name=test_node_certification

    Setup Network And Create Node

    # Make Alice a node certifier
    Add Node Certifier    account_name=Alice    who=Sudo

    Set Node Certification    node_id=${1}    certification=Certified

    Remove Node Certifier    account_name=Alice    who=Sudo

    # Alice is no longer able to set node certification
    Run Keyword And Expect Error    *'NotAllowedToCertifyNode'*
    ...    Set Node Certification    node_id=${1}    certification=Certified

    Tear Down Multi Node Network

Test Add Remove Public Ips
    [Documentation]    Testing api calls (adding, removing) for managing public ips
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
    [Documentation]    Testing adding an invalid public IP
    Setup Multi Node Network    log_name=test_add_pub_ips_failure_invalidpubip

    Setup Network And Create Farm
    # Add an ip in an invalid format
    Run Keyword And Expect Error    *'InvalidPublicIP'*
    ...    Add Farm Ip    id=${1}    ip="185.206.122.125"    gateway=185.206.122.1

    Tear Down Multi Node Network

Test Create Update Delete Node
    [Documentation]    Testing api calls (create, update, delete) for managing nodes
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

Test Reporting Uptime
    [Documentation]    Testing reporting uptime including a failed attempt to report uptime to a non existing node
    Setup Multi Node Network    log_name=test_reporting_uptime
    
    Run Keyword And Expect Error    *'TwinNotExists'*
    ...    Report Uptime    ${500}

    Setup Predefined Account    who=Alice
    
    Run Keyword And Expect Error    *'NodeNotExists'*
    ...    Report Uptime    ${500}

    Create Farm    name=alice_farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent
    
    Report Uptime    ${500}

    Tear Down Multi Node Network

Test Add Public Config On Node: Success
    [Documentation]    Testing adding a public config on a node
    Setup Multi Node Network    log_name=test_add_pub_config_node

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
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_ipv4

    Setup Network And Create Node

    Run Keyword And Expect Error    *'InvalidIP4'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33    gw4=185.206.122.1    domain=some-domain

    Tear Down Multi Node Network

Test Add Public Config On Node: Failure InvalidIP6
    [Documentation]    Testing adding a public config on a node with an invalid ipv6
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_ipv6

    Setup Network And Create Node  

    Run Keyword And Expect Error    *'InvalidIP6'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33/24    gw4=185.206.122.1    ipv6=2a10:b600:1::0cc4:7a30:65b5    gw6=2a10:b600:1::1    domain=some-domain
    
    Tear Down Multi Node Network

Test Add Public Config On Node: Failure InvalidDomain
    [Documentation]    Testing adding a public config on a node with an invalid domain
    Setup Multi Node Network    log_name=test_add_pub_config_node_failure_invaliddomain

    Setup Network And Create Node
    Run Keyword And Expect Error    *'InvalidDomain'*
    ...    Add Node Public Config    farm_id=${1}    node_id=${1}    ipv4=185.206.122.33/24    gw4=185.206.122.1    ipv6=2a10:b600:1::0cc4:7a30:65b5/64    gw6=2a10:b600:1::1    domain=some_invalid_domain

    Tear Down Multi Node Network

Test Create Update Cancel Node Contract: Success
    [Documentation]    Testing api calls (create, update, cancel) for managing a node contract
    Setup Multi Node Network    log_name=test_create_node_contract

    Setup Predefined Account    who=Alice
    Setup Predefined Account    who=Bob
    
    ${ip_1} =     Create Dictionary    ip    185.206.122.33/24    gw    185.206.122.1
    ${public_ips} =    Create List    ${ip_1}
    Create Farm    name=alice_farm    public_ips=${public_ips}
    
    ${interface_ips} =     Create List    10.2.3.3
    ${interface_1} =     Create Interface    name=zos    mac=00:00:5e:00:53:af    ips=${interface_ips}
    ${interfaces} =    Create List    ${interface_1}
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}   longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent    interfaces=${interfaces}
    
    # Bob is the one creating the contract and thus being billed
    Create Deployment Contract    farm_id=${1}    public_ips=${1}    who=Bob    port=9946

    ${farm} =     Get Farm    ${1}
    Should Not Be Equal    ${farm}    ${None}    msg=Farm with id 1 doesn't exist
    Dictionary Should Contain Key    ${farm}    public_ips    msg=The farm doesn't have a key public_ips
    Length Should Be    ${farm}[public_ips]    1    msg=There should only be one public ip in public_ips
    Should Be Equal    ${farm}[public_ips][0][ip]    185.206.122.33/24    msg=The public ip address should be 185.206.122.33/24
    Should Be Equal    ${farm}[public_ips][0][gateway]    185.206.122.1    msg=The gateway should be 185.206.122.1
    Should Be Equal    ${farm}[public_ips][0][contract_id]    ${1}    msg=The public ip was claimed in contract with id 1 while the farm contains a different contract id for it

    Update Deployment Contract    contract_id=${1}    who=Bob    port=9946

    Cancel Node Contract    contract_id=${1}    who=Bob    port=9946

    Tear Down Multi Node Network

Test Create Node Contract: Failure Not Enough Public Ips
    [Documentation]    Testing creating a node contract and requesting too much pub ips
    Setup Multi Node Network    log_name=test_create_node_contract_failure_notenoughpubips

    # the function below creates a farm containing 0 public ips and a node with 0 configured interfaces
    Setup Network And Create Node
    # let's request 2 public ips which should result in an error
    Run Keyword And Expect Error    *'FarmHasNotEnoughPublicIPs'*
    ...    Create Deployment Contract    farm_id=${1}    public_ips=${2}

    Tear Down Multi Node Network

Test Create Rent Contract: Success
    [Documentation]    Testing api calls (create, cancel) for managing a rent contract
    Setup Multi Node Network    log_name=test_create_rent_contract

    Setup Network And Create Node
    
    Create Rent Contract    node_id=${1}

    Cancel Rent Contract    contract_id=${1}

    Tear Down Multi Node Network

Test Create Name Contract: Success
    [Documentation]    Testing api calls (create, cancel) for managing a name contract
    Setup Multi Node Network    log_name=test_create_name_contract

    Setup Network And Create Node

    Create Name Contract    name=my_name_contract

    Cancel Name Contract    contract_id=${1}

    Tear Down Multi Node Network

Test Create Update Pricing Policy
    [Documentation]    Testing api calls (create, update) for managing pricing policies including failed attempts
    Setup Multi Node Network    log_name=test_create_update_pricing_policy

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Create Pricing Policy    name=mypricingpolicy    unit=Gigabytes    su=${55000}    cu=${90000}    nu=${20000}    ipu=${35000}    unique_name=${3000}    domain_name=${6000}    foundation_account=Bob    certified_sales_account=Bob    discount_for_dedication_nodes=45

    Create Pricing Policy    name=mypricingpolicy    unit=Gigabytes    su=${55000}    cu=${90000}    nu=${20000}    ipu=${35000}    unique_name=${3000}    domain_name=${6000}    foundation_account=Bob    certified_sales_account=Bob    discount_for_dedication_nodes=45    who=Sudo
    ${pricing_policy} =     Get Pricing Policy    id=${2}
    Should Not Be Equal    ${pricing_policy}    ${None}
    Should Be Equal    ${pricing_policy}[name]    mypricingpolicy

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Update Pricing Policy    id=${2}    name=mypricingpolicyupdated    unit=Gigabytes    su=${55000}    cu=${90000}    nu=${20000}    ipu=${35000}    unique_name=${3000}    domain_name=${6000}    foundation_account=Bob    certified_sales_account=Bob    discount_for_dedication_nodes=45

    Update Pricing Policy    id=${2}    name=mypricingpolicyupdated    unit=Gigabytes    su=${55000}    cu=${90000}    nu=${20000}    ipu=${35000}    unique_name=${3000}    domain_name=${6000}    foundation_account=Bob    certified_sales_account=Bob    discount_for_dedication_nodes=45    who=Sudo
    ${pricing_policy} =     Get Pricing Policy    id=${2}
    Should Not Be Equal    ${pricing_policy}    ${None}
    Should Be Equal    ${pricing_policy}[name]    mypricingpolicyupdated

    Tear Down Multi Node Network

Test Create Update Farming Policy
    [Documentation]    Testing api calls (create, update) for managing farming policies including failed attempts
    Setup Multi Node Network    log_name=test_create_update_farming_policy

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Create Farming Policy    name=myfarmingpolicy    su=${12}    cu=${15}    nu=${10}    ipv4=${8}    minimal_uptime=${9999}    policy_end=${10}    immutable=${True}    default=${True}    node_certification=Diy    farm_certification=Gold

    Create Farming Policy    name=myfarmingpolicy    su=${12}    cu=${15}    nu=${10}    ipv4=${8}    minimal_uptime=${9999}    policy_end=${15}    immutable=${True}    default=${True}    node_certification=Diy    farm_certification=Gold    who=Sudo
    ${farming_policy} =    Get Farming Policy    id=${3}
    Should Not Be Equal    ${farming_policy}    ${None}
    Should Be Equal    ${farming_policy}[name]    myfarmingpolicy

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Update Farming Policy    id=${3}    name=myfarmingpolicyupdated    su=${12}    cu=${15}    nu=${10}    ipv4=${8}    minimal_uptime=${9999}    policy_end=${10}    immutable=${True}    default=${True}    node_certification=Diy    farm_certification=Gold

    Update Farming Policy    id=${3}    name=myfarmingpolicyupdated    su=${12}    cu=${15}    nu=${10}    ipv4=${8}    minimal_uptime=${9999}    policy_end=${10}    immutable=${True}    default=${True}    node_certification=Diy    farm_certification=Gold    who=Sudo
    ${farming_policy} =    Get Farming Policy    id=${3}
    Should Not Be Equal    ${farming_policy}    ${None}
    Should Be Equal    ${farming_policy}[name]    myfarmingpolicyupdated
    
    Tear Down Multi Node Network

Test Attach Policy To Farm
    [Documentation]    Testing attaching a policy to a farm including a failed attempt to attach an expired policy
    Setup Multi Node Network    log_name=test_attach_policy_to_farm

    Setup Network And Create Farm
    Create Farming Policy    name=myfarmingpolicy    su=${12}    cu=${15}    nu=${10}    ipv4=${8}    minimal_uptime=${9999}    policy_end=${5}    immutable=${True}    default=${True}    node_certification=Diy    farm_certification=Gold    who=Sudo
    ${policy} =     Get Farming Policy    id=${3}
    Should Not Be Equal    ${policy}    ${None}
    Should Be Equal    ${policy}[name]    myfarmingpolicy

    # only possible with sudo
    Run Keyword And Expect Error    *'BadOrigin'*
    ...    Attach Policy To Farm    farm_id=${1}    farming_policy_id=${3}    cu=${20}    su=${2}    end=${1654058949}    node_certification=${False}    node_count=${10}

    Attach Policy To Farm    farm_id=${1}    farming_policy_id=${3}    cu=${20}    su=${2}    end= ${1654058949}    node_certification=${False}    node_count=${10}    who=Sudo

    # farming policy expires after 5 blocks
    Wait X Blocks    x=${5}
    Run Keyword And Expect Error    {'Err': {'Module': {'index': 11, 'error': '0x52000000'}}}
    ...    Attach_policy_to_farm    farm_id=${1}    farming_policy_id=${3}    cu=${20}    su=${2}    end=${1654058949}    node_certification=${False}    node_count=${10}    who=Sudo

    Tear Down Multi Node Network

Test Billing
    [Documentation]    Testing billing. Alice creates a twin and Bob too. Alice creates a farm and a node in that farm while Bob creates a node contract requesting Alice to use her node. Alice will report contract resources. We will wait 6 blocks so that Bob will be billed a single time.
    Setup Multi Node Network    log_name=test_billing

    # Setup
    Setup Predefined Account    who=Alice
    Setup Predefined Account    who=Bob  
    Create Farm    name=alice_farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent

    ${balance_alice} =    Balance Data    who=Alice
    ${balance_bob} =    Balance Data    who=Bob    port=${9946}
    # Bob will be using the node: let's create a node contract in his name
    Create Deployment Contract    farm_id=${1}    hru=${20}    sru=${20}    cru=${2}    mru=${4}   port=${9946}    who=Bob
    Add Nru Reports    contract_id=${1}    nru=${3}

    # Let it run 6 blocks so that the user will be billed 1 time
    Wait X Blocks    ${6}
    Cancel Node Contract    contract_id=${1}    who=Bob

    # Balance should have decreased
    ${balance_alice_after} =    Balance Data    who=Alice
    ${balance_bob_after} =    Balance Data    who=Bob    port=${9946}
    Ensure Account Balance Decreased    ${balance_bob}    ${balance_bob_after}

    Tear Down Multi Node Network

Test Solution Provider
    [Documentation]    Testing creating and validating a solution provider
    Setup Multi Node Network    log_name=test_create_approve_solution_provider    amt=${2}

    # Setup
    Setup Predefined Account    who=Alice
    Setup Predefined Account    who=Bob
    Create Farm    name=alice_farm
    Create Node    farm_id=${1}    hru=${1024}    sru=${512}    cru=${8}    mru=${16}    longitude=2.17403    latitude=41.40338    country=Belgium    city=Ghent
    
    # lets add two providers: charlie gets 30% and Dave 10%
    ${providers} =    Create Dictionary    Charlie    ${30}    Dave    ${10}
    Create Solution Provider    description=mysolutionprovider    providers=${providers}
    ${solution_provider} =    Get Solution Provider    id=${1}
    Should Not Be Equal    ${solution_provider}    ${None}
    Should Be Equal    ${solution_provider}[description]    mysolutionprovider
    Should Be Equal    ${solution_provider}[approved]    ${False}
    Length Should Be    ${solution_provider}[providers]    ${2}
    
    # The solution provider has to be approved
    Approve Solution Provider    solution_provider_id=${1}    who=Sudo
    ${solution_provider} =    Get Solution Provider    id=${1}
    Should Not Be Equal    ${solution_provider}    ${None}
    Should Be Equal    ${solution_provider}[approved]    ${True}

    ${balance_charlie_before} =     Balance Data    who=Charlie
    ${balance_dave_before} =    Balance Data    who=Dave
    # Bob will be using the node: let's create a node contract in his name
    Create Deployment Contract    farm_id=${1}    hru=${20}    sru=${20}    cru=${2}    mru=${4}    port=9946    who=Bob    solution_provider_id=${1}
    Add Nru Reports    contract_id=${1}    nru=${3}
    # Wait 6 blocks: after 5 blocks Bob should be billed
    Wait X Blocks    ${6}
    # Cancel the contract so that the bill is distributed and so that the providers get their part
    Cancel Node Contract    contract_id=${1}    who=Bob

    # Verification: both providers should have received their part
    ${balance_charlie_after} =     Balance Data    who=Charlie
    ${balance_dave_after} =    Balance Data    who=Dave
    Ensure Account Balance Increased    ${balance_charlie_before}    ${balance_charlie_after}
    Ensure Account Balance Increased    ${balance_dave_before}    ${balance_dave_after}

    Tear Down Multi Node Network