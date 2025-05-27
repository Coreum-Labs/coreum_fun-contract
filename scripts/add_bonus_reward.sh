#!/bin/bash

# Set the contract address and the amount to add as bonus
CONTRACT_ADDRESS="core1pkdpdj05g5xjvq98qxlyt6faz2p7d7vhughrnrqelu6ue3eakeaseux75g"
BONUS_AMOUNT="1000000000" # Amount in utestcore

# Send the add bonus reward message
cored tx wasm execute $CONTRACT_ADDRESS '{"add_bonus_reward_to_the_pool":{"amount":"'$BONUS_AMOUNT'"}}' \
    --from coreum_fun \
    --chain-id coreum-mainnet-1 \
    --node https://coreum-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 30000ucore \
    --amount "1000000000ucore" \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash" 
