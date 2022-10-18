# ibc
## overview

in order to communicate between contracts using ibc a channel must be created (4 way handshake) and port.

port (receiver on blockchain): created when a contract is instantiated and is used to send communications to a specific contract. to find the port for your contract use ```kujirad query wasm contract <address>``` and look at the ibc_port_field

relayers are used to send packets from chain A to chain B. in order to do this in test, you can configure the ```hermes``` relayer to work between two chain (kujira & juno)

1. openInit - message to chain B containing contents to verify the identity of chain A
2. openTry - message to chain A containing contents to verify the identity of chain B
3. openAck - message to chain B that chain A is ready to receive communications
4. openConfirm - message to chain A that chain B is ready to receive communications

## entry point 
1. ibc_channel_open - handles openInit and OpenTry
2. ibc_channel_connect - handles openAck and openConfirm
3. ibc_channel_close - handles closing of ibc channel by counter party
4. ibc_handle_receive - handles receiving ibc packets from counter party
5. ibc_packet_ack - handles ack messages from the counter party
6. ibc_packet_timeout - handles packet timeouts

## instantiation
once a contract is instantiated it will be assigned a port


## hermes
1. config.toml
    - chain-id for kujira local = test
    - chain-id for juno local = testing
2. ports
    - export gRpc ports on all docker images (9090)
    - map ports in docker compose
    - use these ports as values for the config.toml gRpc
3. hermes keys add
    - create keys on kujir and output to json
    - use output json in hermes keys add command to add the key to the chain
    - same for juno