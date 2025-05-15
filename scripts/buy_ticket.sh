#!/bin/bash

# Set the contract address and the number of tickets to buy
CONTRACT_ADDRESS="testcore190mwus5wkung2wvku60hnvu0p2h4nvwutms67n5tyrysu35cejwq3cjwad"
NUMBER_OF_TICKETS="4"
PRICE="200000000"
AMOUNT=$((PRICE * NUMBER_OF_TICKETS))

# Send the buy ticket message
cored tx wasm execute $CONTRACT_ADDRESS '{"buy_ticket":{"number_of_tickets":"'$NUMBER_OF_TICKETS'"}}' \
    --from ebc-poa-mainnet \
    --chain-id coreum-testnet-1 \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 3000000utestcore \
    --amount 800000000utestcore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"
# ebc-poa-mainnet
# RelayerTestnet

