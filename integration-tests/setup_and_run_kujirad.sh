#!/bin/sh
#set -o errexit -o nounset -o pipefail
# from: https://github.com/CosmosContracts/juno/blob/main/docker/setup_junod.sh

KEYRING_PASSWORD="12345678"
STAKE="ukuji"
CHAIN_ID="test"
NODE_NAME="node001"
KEY_NAME="validator"
GENESIS_FILE="$HOME"/.kujira/config/genesis.json

# init genesis.json
kujirad init --chain-id "$CHAIN_ID" "$NODE_NAME"

# create keys
(echo "$KEYRING_PASSWORD"; echo "$KEYRING_PASSWORD") | kujirad keys add "$KEY_NAME"

# add to genesis
echo "$KEYRING_PASSWORD" | kujirad add-genesis-account "$KEY_NAME" "10000000000$STAKE" --keyring-backend os

# add additional account to genesis if passed in
if [[ $# -eq 1 ]] ; then
    kujirad add-genesis-account "$1" "999999999$STAKE"
fi

# create genesis transaction and bond ukuji
echo "$KEYRING_PASSWORD" | kujirad gentx "$KEY_NAME" "1000000$STAKE" --chain-id="$CHAIN_ID" --amount="1$STAKE"

kujirad collect-gentxs

# replace some chain specifics
# from: https://docs.kujira.app/validators/run-a-node
sed -i "s/\"stake\"/\"$STAKE\"/" "$GENESIS_FILE"

sed -i "s/^minimum-gas-prices *=.*/minimum-gas-prices = \"0.00125ukuji\"/;" $HOME/.kujira/config/app.toml

sed -i "s/^timeout_commit *=.*/timeout_commit = \"1500ms\"/;" $HOME/.kujira/config/config.toml

kujirad start --rpc.laddr tcp://0.0.0.0:26657 --trace