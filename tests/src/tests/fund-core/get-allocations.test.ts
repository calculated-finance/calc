import { coin } from '@cosmjs/proto-signing';
import { Context } from 'mocha';
import { execute } from '../../shared/cosmwasm';
import { instantiateFinPairContract, instantiateSwapContract, instatiateFundCoreContract } from '../hooks';
import { expect } from '../shared.test';

describe.only('when fetching allocations', () => {
  let fundContractAddress: string;

  before(async function (this: Context) {
    // set up the required fin pairs
    const swapperContractAddress = await instantiateSwapContract(this.cosmWasmClient, this.adminContractAddress);

    const pairs = [
      { baseDenom: 'ukuji', quoteDenom: 'uusk' },
      { baseDenom: 'uatom', quoteDenom: 'uusk' },
      { baseDenom: 'uosmo', quoteDenom: 'uusk' },
      { baseDenom: 'uaxlusdc', quoteDenom: 'uusk' },
    ];

    for (const pair of pairs) {
      const pairAddress = await instantiateFinPairContract(
        this.cosmWasmClient,
        this.adminContractAddress,
        pair.baseDenom,
        pair.quoteDenom,
      );

      await execute(this.cosmWasmClient, this.adminContractAddress, swapperContractAddress, {
        add_path: {
          pair: {
            fin: { address: pairAddress, base_denom: pair.baseDenom, quote_denom: pair.quoteDenom },
          },
        },
      });
    }

    fundContractAddress = await instatiateFundCoreContract(
      this.cosmWasmClient,
      this.adminContractAddress,
      swapperContractAddress,
    );

    await this.cosmWasmClient.sendTokens(
      this.adminContractAddress,
      fundContractAddress,
      [coin(100000, 'uatom'), coin(100000, 'uosmo'), coin(100000, 'uaxlusdc')],
      'auto',
    );
  });

  it('returns the expected allocations', async function (this: Context) {
    const allocations = await this.cosmWasmClient.queryContractSmart(fundContractAddress, {
      get_allocations: {},
    });

    expect(allocations).to.deep.equal({
      uatom: '0.333333333333333333',
      uosmo: '0.333333333333333333',
      axlusdc: '0.333333333333333334',
    });
  });
});
