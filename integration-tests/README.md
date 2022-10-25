# **Calculated Finance LocalNet Automation**

Set up and run a Kujira Localnet for development

### **Local Development**

1. Follow the instructions in the `calc-dca` repository to build a calc.wasm binary, and copy it into this directory.

2. Copy .env.example and insert relevant values

```bash
cp .env.example .env.dev
```

```bash
BECH32_ADDRESS_PREFIX="kujira"
FEE_DENOM="ukuji"
GAS_PRICE="0.00125"
NET_URL="ws://localhost:26657"
ADMIN_CONTRACT_MNEMONIC="{provide a mnemonic of an address you own on testnet}"
ADMIN_CONTRACT_ADDRESS="{provide the bech32 of the same address}"
```

3. Add your ADMIN_CONTRACT_ADDRESS to the docker-compose file

```yml
...
    command: ./setup_and_run_kujirad.sh {replace with ADMIN_CONTRACT_ADDRESS value}
    ...
```

4. Run `npm run start` - this will:

- start a docker container with the Kujira blockchain running inside, and instantiate a wallet at ADMIN_CONTRACT_ADDRESS with some funds
- Upload and instantiate the Calc contract and print out the contract address for you to copy into the relevant `.env` files elsewhere for local testing
