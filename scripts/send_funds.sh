#!/bin/bash

# Set the contract address and the recipient address
CONTRACT_ADDRESS="testcore19zsn4cmtqzxlegkdg6vn62ptsk40q9ssnynetemjxec2q3mllhgq9h2ehn"
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

