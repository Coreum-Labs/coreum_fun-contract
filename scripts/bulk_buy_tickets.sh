#!/bin/bash

# Configuration
FROM_ADDRESS="dex"
TOTAL_ADDRESSES=200
TICKETS_PER_ADDRESS=5
FUND_AMOUNT="1001000000utestcore"
TICKET_PRICE="200000000"
CONTRACT_ADDRESS="testcore1x66zxsh75r0ydpz7kqcwxe989ms4fvfprpwdxzd8c509z4m6d0wqsawm5w" # Replace with your contract address after instantiation

# Create a directory for our keys
mkdir -p keys

# Main execution
echo "Starting bulk ticket purchase process..."
echo "Using FROM_ADDRESS: $FROM_ADDRESS"

# Combined function to generate address, fund it, and buy tickets
for i in $(seq 101 $TOTAL_ADDRESSES); do
    echo "Processing address $i of $TOTAL_ADDRESSES..."
    
    # Generate new address with --no-backup flag to avoid prompts
    cored keys add "buyer_$i" --chain-id=coreum-testnet-1
    
    # Get the address using the correct command
    ADDRESS=$(cored keys show "buyer_$i" --bech acc -a --chain-id=coreum-testnet-1)
    
    # Send funds to the new address
    cored tx bank send $FROM_ADDRESS $ADDRESS $FUND_AMOUNT \
        --chain-id coreum-testnet-1 \
        --gas auto --gas-adjustment 1.3 \
        --fees 30000utestcore \
        --node https://coreum-testnet-rpc.ibs.team/ \
        --from $FROM_ADDRESS \
        -y
        
    echo "Created and funded address $i: $ADDRESS"
    sleep 5  # Wait for funding transaction to be processed
    
    # Buy tickets for the address
    cored tx wasm execute $CONTRACT_ADDRESS \
        '{"buy_ticket": {"number_of_tickets": "'$TICKETS_PER_ADDRESS'"}}' \
        --from "buyer_$i" \
        --chain-id coreum-testnet-1 \
        --gas auto --gas-adjustment 1.3 \
        --fees 30000utestcore \
        --node https://coreum-testnet-rpc.ibs.team/ \
        --amount "$(($TICKET_PRICE * $TICKETS_PER_ADDRESS))utestcore" \
        -y
        
    echo "Bought $TICKETS_PER_ADDRESS tickets for buyer_$i"
    sleep 2  # Wait for ticket purchase transaction to be processed
    
   
done

echo "Bulk ticket purchase process completed!" 
