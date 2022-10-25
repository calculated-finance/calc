import fs from 'fs';
import dotenv from 'dotenv';
import { fetchConfig } from '../shared/config';
import { createAdminCosmWasmClient } from '../shared/cosmwasm';

dotenv.config();

const instantiateAndDeploy = async () => {
  const config = await fetchConfig();
  const client = await createAdminCosmWasmClient(config);

  const uploadResponse = await client.upload(config.adminContractAddress, fs.readFileSync(process.argv[2]), 'auto');

  console.log(`Uploaded dca contract to local chain with codeId: ${uploadResponse.codeId}`);

  const instantiateResponse = await client.instantiate(
    config.adminContractAddress,
    uploadResponse.codeId,
    {
      admin: config.adminContractAddress,
      fee_collector: config.adminContractAddress,
      fee_percent: '2000000',
      staking_router_address: 'kujira1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqnqu9cc',
    },
    'calc-dca',
    'auto',
    {
      admin: config.adminContractAddress,
    },
  );

  console.log(`Instantiated dca contract on local chain at address: ${instantiateResponse.contractAddress}`);
  process.exit(0);
};

instantiateAndDeploy();
