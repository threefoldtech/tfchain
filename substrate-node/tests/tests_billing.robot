*** Settings ***
Documentation    Suite for integration tests for testing billing
Library          substrate_network.py



*** Test Cases ***
Test Setup Then Tear Down
    ${nodes} =  Setup Multi Node Network

    Sleep  5s

    Tear Down Multi Node Network  ${nodes}