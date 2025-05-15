#!/bin/bash

# Replace these variables with your actual values
CONTRACT_CODE_ID=2413  # 
LABEL="Coreum Fun Contract"
FROM_ADDRESS="dex"  # Your address that will pay for transaction fees and be the owner of the contract

# Save the instantiate message to a file
cat > coreum_fun_contract_instantiate.json << 'EOF'
{
  "validator_address": "testcorevaloper1xjehmty2z5j7mfmpzxe8dgrf506c70n37lggga",
  "total_tickets": "10",
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
    --fees 3000000utestcore \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --amount 10000000utestcore \
    -y
