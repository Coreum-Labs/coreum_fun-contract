
CONTRACT_ADDRESS="core1pkdpdj05g5xjvq98qxlyt6faz2p7d7vhughrnrqelu6ue3eakeaseux75g"

cored tx wasm execute $CONTRACT_ADDRESS '{"send_funds_to_winner":{}}' \
    --from coreum_fun \
    --chain-id coreum-mainnet-1 \
    --node https://coreum-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 300000ucore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"

