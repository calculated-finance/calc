import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import path from 'path';
import { Config, fetchConfig } from '../shared/config';
import { createAdminCosmWasmClient, execute } from '../shared/cosmwasm';

(async () => {
  const config: Config = {
    bech32AddressPrefix: 'kujira',
    netUrl: 'http://0.0.0.0:26657',
    gasPrice: 0.00125,
    feeDenom: 'ukuji',
    adminContractMnemonic:
      'solution select match tell survey picnic trouble off bread fork gold dragon taxi mad artefact truck hair avocado success scene heavy alarm stay lazy',
  };

  const client = await createAdminCosmWasmClient(config);

  // GET BALANCE

  const balance = await client.getBalance('kujira127u07nu657pav5dlhxh0pshzpanjvqykwnehxe', 'udemo');

  console.log(balance);

  // UPDATE CONFIG

  // const update_config_response = await execute(config, client, config.adminContractAddress, config.dcaContractAddress, {
  //   update_config: { fee_percent: '0.015' },
  // });

  // console.log(update_config_response);

  // GET VAULT

  // const response = await client.queryContractSmart(config.dcaContractAddress, {
  //   get_vault: {
  //     address: 'kujira1y4k0re9q905nvvcvvmxug3sqtd9e7du4709d0t',
  //     vault_id: '12',
  //   },
  // });

  // console.log(response.vault);
  // console.log(response.trigger);
  // console.log(`Trigger to be executed ${dayjs(response.trigger.configuration.time.target_time / 1000000).fromNow()}`);

  // GET EVENTS

  // const events_response = await client.queryContractSmart(config.dcaContractAddress, {
  //   get_events_by_resource_id: {
  //     resource_id: '1',
  //   },
  // });

  // console.log(events_response);

  // forEach((event: any) => {
  //   console.log(dayjs(event.timestamp / 1000000).fromNow());
  //   console.log(event.data);
  // }, events_response.events);

  // GET TIME TRIGGERS

  // const triggers_response = await client.queryContractSmart(config.dcaContractAddress, {
  //   get_time_trigger_ids: {},
  // });

  // console.log(triggers_response);

  // EXECUTE TRIGGER

  // const execute_trigger_response = await execute(
  //   config,
  //   client,
  //   config.adminContractAddress,
  //   config.dcaContractAddress,
  //   {
  //     execute_trigger: { trigger_id: '12' },
  //   },
  // );

  // console.log(execute_trigger_response);

  // CANCEL VAULT

  // const cancel_vault_response = await execute(config, client, config.adminContractAddress, config.dcaContractAddress, {
  //   cancel_vault: { address: 'kujira16q6jpx7ns0ugwghqay73uxd5aq30du3uqgxf0d', vault_id: '14' },
  // });

  // console.log(cancel_vault_response);

  // FIN Swap

  // const swapPairAddress = 'kujira14sa4u42n2a8kmlvj3qcergjhy6g9ps06rzeth94f2y6grlat6u6ssqzgtg';

  // const finSwapResponse = await execute(
  //   config,
  //   client,
  //   config.adminContractAddress,
  //   swapPairAddress,
  //   {
  //     swap: {
  //       // max_spread: '0.0',
  //       // belief_price: '3.0',
  //     },
  //   },
  //   [{ denom: 'factory/kujira1ltvwg69sw3c5z99c6rr08hal7v0kdzfxz07yj5/demo', amount: '2' }],
  // );

  // console.log(finSwapResponse);

  // FIN Price Response

  // const pricePairAddress = 'kujira1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsqq4jjh';

  // const finPriceResponse = await client.queryContractSmart(pricePairAddress, {
  //   price: { price: '1.0' },
  // });

  // console.log(JSON.stringify(finPriceResponse));

  // FIN Book Response

  // const bookPairAddress = 'kujira14sa4u42n2a8kmlvj3qcergjhy6g9ps06rzeth94f2y6grlat6u6ssqzgtg';

  // const finBookResponse = await client.queryContractSmart(bookPairAddress, {
  //   book: { limit: 2 },
  // });

  // console.log(finBookResponse);
})();
