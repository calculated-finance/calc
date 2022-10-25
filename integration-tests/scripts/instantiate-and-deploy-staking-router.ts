import fs from 'fs';
import dotenv from 'dotenv';
import { fetchConfig } from '../shared/config';
import { createAdminCosmWasmClient } from '../shared/cosmwasm';

dotenv.config();

const instantiateAndDeploy = async () => {
  const config = await fetchConfig();
  const client = await createAdminCosmWasmClient(config);

  const uploadResponse = await client.upload(config.adminContractAddress, fs.readFileSync(process.argv[2]), 'auto');

  console.log(`Uploaded staking router contract to local chain with codeId: ${uploadResponse.codeId}`);

  const instantiateResponse = await client.instantiate(
    config.adminContractAddress,
    uploadResponse.codeId,
    {
      admin: config.adminContractAddress,
      allowed_z_callers: ['kujira14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sl4e867'],
    },
    'calc-staking-router',
    'auto',
    {
      admin: config.adminContractAddress,
    },
  );

  console.log(`Instantiated staking router contract on local chain at address: ${instantiateResponse.contractAddress}`);
  process.exit(0);
};

instantiateAndDeploy();
