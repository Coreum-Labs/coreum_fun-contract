# Coreum fun contract:

## How to build

```bash
cargo build
```

## How to Optimize

```bash
./scripts/optimize.sh
```

## How to generate types

```bash
ts-codegen generate \
          --plugin client \
          --schema ./schema \
          --out ./ts \
          --name Whitelist \
          --no-bundle
```

## For storing, executing, instantiating, querying the contract you can use <https://devtools.coreumlabs.org/>

## How to store

```bash
cored tx wasm store ./artifacts/coreum_fun_contract.wasm --chain-id=coreum-testnet-1 --from=dex --node=https://coreum-testnet-rpc.ibs.team/  --gas auto --gas-adjustment 1.3

```
## How to Instantiate

```bash
See scripts/instantiate.sh
```

## How to Query

```bash
cargo run
```

//TODO:

## How to buy tickets (JSON)

```bash
{
  "buy_ticket": {
    "number_of_tickets": "1"
  }
}
```

## How to select winner (JSON)

```bash
{
  "select_winner": {
    "winner_address": "testcore1zgdprlr3hz5hhke9ght8mq723a8wlnzqcepjcd"
  }
}
```


## How to send funds to winner (JSON)

```bash
{
  "send_funds_to_winner": {}
}
```
## How to burn tickets (JSON)

```bash
{
  "burn_tickets": {
    "number_of_tickets": "1"
  }
}
```
## How to add bonus rewards (JSON)

```bash
{
  "add_bonus_rewards": {
    "amount": "1"
  }
}
```
## How to update draw state (JSON)

```bash
{
  "update_draw_state": {}
}
```
## How to set undelegation timestamp (JSON)

```bash
{
  "set_undelegation_timestamp": {}
}
```
