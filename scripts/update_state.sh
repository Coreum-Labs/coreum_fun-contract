#!/bin/bash

# Contract address - replace with your contract address
# testcore1a9cpwl0jtl8npqymekphw4ynfxs377nweuh922rts2gs7gva7nvsrlpgdv
# testcore1x66zxsh75r0ydpz7kqcwxe989ms4fvfprpwdxzd8c509z4m6d0wqsawm5w
CONTRACT_ADDRESS="testcore1a9cpwl0jtl8npqymekphw4ynfxs377nweuh922rts2gs7gva7nvsrlpgdv"  # Replace with your contract address
FROM_ADDRESS="dex"  # Your address that will pay for transaction fees

# Function to update draw state
update_draw_state() {
    local new_state=$1
    
    # Create the update draw state message
    cat > update_draw_state.json << EOF
{
    "update_draw_state": {
        "new_state": "$new_state"
    }
}
EOF

    # Execute the update draw state command
    cored tx wasm execute $CONTRACT_ADDRESS \
        "$(cat update_draw_state.json)" \
        --from=$FROM_ADDRESS \
        --chain-id coreum-testnet-1 \
        --gas auto --gas-adjustment 1.3 \
        --fees 30000utestcore \
        --node https://coreum-testnet-rpc.ibs.team/ \
        -y
}

# Function to set undelegation timestamp
set_undelegation_timestamp() {
    local timestamp=$1
    
    # Create the set undelegation timestamp message
    cat > set_undelegation_timestamp.json << EOF
{
    "set_undelegation_timestamp": {
        "timestamp": $timestamp
    }
}
EOF

    # Execute the set undelegation timestamp command
    cored tx wasm execute $CONTRACT_ADDRESS \
        "$(cat set_undelegation_timestamp.json)" \
        --from=$FROM_ADDRESS \
        --chain-id coreum-testnet-1 \
        --gas auto --gas-adjustment 1.3 \
        --fees 30000utestcore \
        --node https://coreum-testnet-rpc.ibs.team/ \
        -y
}

# Function to get current state
get_current_state() {
    cored query wasm contract-state smart $CONTRACT_ADDRESS \
        '{"get_current_state":{}}' \
        --node https://coreum-testnet-rpc.ibs.team/
}

# Main script
case "$1" in
    "update_state")
        if [ -z "$2" ]; then
            echo "Error: Please provide a new state"
            echo "Available states:"
            echo "  - TicketSalesOpen"
            echo "  - TicketsSoldOutAccumulationInProgress"
            echo "  - WinnerSelectedUndelegationInProcess"
            echo "  - UndelegationCompletedTokensCanBeBurned"
            echo "  - DrawFinished"
            exit 1
        fi
        update_draw_state "$2"
        ;;
    "set_timestamp")
        if [ -z "$2" ]; then
            echo "Error: Please provide a timestamp"
            exit 1
        fi
        set_undelegation_timestamp "$2"
        ;;
    "get_state")
        get_current_state
        ;;
    *)
        echo "Usage: $0 {update_state|set_timestamp|get_state}"
        echo "  update_state <new_state>  - Update the draw state"
        echo "  set_timestamp <timestamp> - Set the undelegation timestamp"
        echo "  get_state                - Get the current state"
        exit 1
        ;;
esac 
