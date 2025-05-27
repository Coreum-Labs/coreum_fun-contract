CONTRACT_ADDRESS="testcore1x66zxsh75r0ydpz7kqcwxe989ms4fvfprpwdxzd8c509z4m6d0wqsawm5w"

# Calculate the ticket denom (assuming ticket symbol is "TICKET")
TICKET_DENOM="uticket_test-${CONTRACT_ADDRESS}"

# Amount of tickets to burn (1 ticket)
TICKETS_TO_BURN="1000000" # 1 ticket with 6 decimals precision

cored tx wasm execute $CONTRACT_ADDRESS '{"burn_tickets": { "number_of_tickets": "1" }}' \
    --node https://coreum-testnet-rpc.ibs.team/ \
    --from dex \
    --chain-id coreum-testnet-1  \
    --gas auto --gas-adjustment 1.3 \
    --fees 3000000utestcore \
    --amount "${TICKETS_TO_BURN}${TICKET_DENOM}" \
    -y





