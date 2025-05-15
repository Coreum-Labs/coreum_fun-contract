
CONTRACT_ADDRESS="testcore190mwus5wkung2wvku60hnvu0p2h4nvwutms67n5tyrysu35cejwq3cjwad"

cored tx wasm execute $CONTRACT_ADDRESS '{"send_funds_to_winner":{}}' \
    --from dex \
    --chain-id coreum-testnet-1 \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --gas auto --gas-adjustment 1.3 \
    --fees 3000000utestcore \
    -y

# Print the transaction hash
echo "Transaction hash: $tx_hash"

