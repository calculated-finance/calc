import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Coin, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { Attribute, Event } from '@cosmjs/stargate/build/logs';
import dayjs from 'dayjs';
import { reduce, assoc } from 'ramda';
import { Config } from './config';
import RelativeTime from 'dayjs/plugin/relativeTime';
dayjs.extend(RelativeTime);

export const createAdminCosmWasmClient = async (config: Config) => {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(config.adminContractMnemonic, {
    prefix: config.bech32AddressPrefix,
  });
  return await SigningCosmWasmClient.connectWithSigner(config.netUrl, wallet, {
    prefix: config.bech32AddressPrefix,
    gasPrice: GasPrice.fromString(`${config.gasPrice}${config.feeDenom}`),
  });
};

export const execute = async (
  config: Config,
  cosmWasmClient: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  message: Record<string, unknown>,
  funds: Coin[] = [],
): Promise<Record<string, unknown>> => {
  const response = await cosmWasmClient.execute(senderAddress, contractAddress, message, 'auto', 'memo', funds);
  return parseEventAttributes(response.logs[0].events);
};

export const parseEventAttributes = (events: readonly Event[]): Record<string, Record<string, string>> =>
  reduce(
    (obj: object, event: Event) => ({
      [event.type]: reduce((obj: any, attr: Attribute) => assoc(attr.key, attr.value, obj), {}, event.attributes),
      ...obj,
    }),
    {},
    events,
  );

export const dayFromCosmWasmUnix = (unix: number) => dayjs(unix / 1000000);
