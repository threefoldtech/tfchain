*** Settings ***
Documentation    Suite for integration tests for testing billing
Library          substrate_network.py
Library          node_setup.py



*** Test Cases ***
Test Setup Then Tear Down
    ${nodes} =  Setup Multi Node Network

    Setup First Node
    Setup Second Node

    Sleep  20s

    Tear Down Multi Node Network  ${nodes}