*** Settings ***
Documentation    Suite for integration tests for testing billing
Library          SubstrateNetwork.py
Library          TfChainClient.py


*** Test Cases ***
Test Setup Then Tear Down
    Setup Multi Node Network

    # Setup for billing
    Setup Alice
    Setup Bob
    User Accept Tc
    Create Twin
    Create Farm
    Create Node
    Create Node Contract
    Report Contract Resources

    # let it run for some time
    Sleep  10s

    Tear Down Multi Node Network