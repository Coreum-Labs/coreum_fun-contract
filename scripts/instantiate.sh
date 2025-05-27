#!/bin/bash

CONTRACT_CODE_ID=2463  # 
CONTRACT_CODE_ID_MAINNET=254
LABEL="Coreum Fun Contract"
FROM_ADDRESS="coreum_fun"  # Your address that will pay for transaction fees

# Testnet
cat > coreum_fun_contract_instantiate.json << 'EOF'
{
  "validator_address": "testcorevaloper1xjehmty2z5j7mfmpzxe8dgrf506c70n37lggga",
  "total_tickets": "500",
  "ticket_price": "200000000",
  "max_tickets_per_user": "5",
  "ticket_token_symbol": "ticket_test",
  "core_denom": "utestcore"
}
EOF

# Run the instantiate command
cored tx wasm instantiate $CONTRACT_CODE_ID \
    "$(cat coreum_fun_contract_instantiate.json)" \
    --label "$LABEL" \
    --admin "$FROM_ADDRESS" \
    --from=dex \
    --chain-id coreum-testnet-1 \
    --gas auto --gas-adjustment 1.3 \
    --fees 30000utestcore \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --amount 10000000utestcore \
    -y

# Mainnet

# cat > coreum_fun_contract_instantiate.json << 'EOF'
# {
#   "validator_address": "corevaloper14e0slqpzhgsakm6fwnh5sk6mu2dmdc9ghxhuw5",
#   "total_tickets": "500",
#   "ticket_price": "200000000", 
#   "max_tickets_per_user": "5",
#   "ticket_token_symbol": "ticket",
#   "core_denom": "ucore"
# }
# EOF

# Run the instantiate command
# cored tx wasm instantiate $CONTRACT_CODE_ID_MAINNET \
#     "$(cat coreum_fun_contract_instantiate.json)" \
#     --label "$LABEL" \
#     --admin "$FROM_ADDRESS" \
#     --from=coreum_fun \
#     --chain-id coreum-mainnet-1 \
#     --gas auto --gas-adjustment 1.3 \
#     --fees 30000ucore \
#     --node https://coreum-rpc.ibs.team/ \
#     --amount 10000000ucore \
#     -y
