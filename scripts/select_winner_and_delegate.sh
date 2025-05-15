#!/bin/bash

# Set the contract address and the winner address
CONTRACT_ADDRESS="testcore190mwus5wkung2wvku60hnvu0p2h4nvwutms67n5tyrysu35cejwq3cjwad"
WINNER_ADDRESS="testcore1lf3r3cx8xdmvae3qnaaxdnjtzuqwu4rjl3gy5z"

# Send the select winner and start undelegation message
cored tx wasm execute $CONTRACT_ADDRESS '{"select_winner_and_undelegate":{"winner_address":"'$WINNER_ADDRESS'"}}' \
    --from dex \
    --chain-id coreum-testnet-1 \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 3000000utestcore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"

