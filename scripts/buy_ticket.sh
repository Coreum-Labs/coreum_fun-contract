#!/bin/bash

# Set the contract address and the number of tickets to buy
CONTRACT_ADDRESS="core1evlgj0xvqtnpw5ejzvucgsmuy6a6sfd9f94a0tw89pc5f3hwkkksan5c9t"
NUMBER_OF_TICKETS="1"
PRICE="5000000ucore"
AMOUNT=$((PRICE * NUMBER_OF_TICKETS))

# Send the buy ticket message
cored tx wasm execute $CONTRACT_ADDRESS '{"buy_ticket":{"number_of_tickets":"'$NUMBER_OF_TICKETS'"}}' \
    --from dex \
    --chain-id coreum-mainnet-1 \
    --node https://coreum-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 30000ucore \
    --amount $PRICE \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"
# ebc-poa-mainnet
# RelayerTestnet

