#!/bin/bash

# Set the contract address and the recipient address
CONTRACT_ADDRESS="testcore190mwus5wkung2wvku60hnvu0p2h4nvwutms67n5tyrysu35cejwq3cjwad"
RECIPIENT_ADDRESS="testcore1zgdprlr3hz5hhke9ght8mq723a8wlnzqcepjcd"
AMOUNT="10000000"

# Send the send funds message
cored tx wasm execute $CONTRACT_ADDRESS '{"send_funds":{"recipient":"'$RECIPIENT_ADDRESS'","amount":"'$AMOUNT'"}}' \
    --from dex \
    --chain-id coreum-testnet-1 \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 3000000utestcore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"

