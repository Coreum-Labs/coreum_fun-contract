#! /bin/bash

# Get the contract address
CONTRACT_ADDRESS="core1pkdpdj05g5xjvq98qxlyt6faz2p7d7vhughrnrqelu6ue3eakeaseux75g"

# Get the new validator address
NEW_VALIDATOR_ADDRESS="corevaloper14e0slqpzhgsakm6fwnh5sk6mu2dmdc9ghxhuw5"

NEW_CODE_ID="254"

#  cored tx wasm migrate [contract_addr_bech32] [new_code_id_int64] [json_encoded_migration_args] [flags]

# Migrate the contract
cored tx wasm migrate $CONTRACT_ADDRESS \
    $NEW_CODE_ID \
    "{\"new_validator_address\": \"$NEW_VALIDATOR_ADDRESS\"}" \
    --from coreum_fun \
    --chain-id coreum-mainnet-1 \
    --gas auto --gas-adjustment 1.3 \
    --fees 300000ucore \
    --node https://coreum-rpc.ibs.team/ \
    -y
