#!/bin/bash

# Set the contract address and the winner address
CONTRACT_ADDRESS="testcore1a9cpwl0jtl8npqymekphw4ynfxs377nweuh922rts2gs7gva7nvsrlpgdv"
WINNER_ADDRESS="testcore1zgdprlr3hz5hhke9ght8mq723a8wlnzqcepjcd"

# Send the select winner and start undelegation message
cored tx wasm execute $CONTRACT_ADDRESS '{"select_winner_and_undelegate":{"winner_address":"'$WINNER_ADDRESS'"}}' \
    --from dex \
    --chain-id coreum-testnet-1 \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 30000utestcore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"

# before winning 89.80433
# accumulated rewards 356.950328
# bonus rewards 1000
#total rewards 1356.950328

# expected after winning 1446.754658


