import { coin } from '@cosmjs/proto-signing';
import { Context } from 'mocha';
import { keys, map, omit, reduce, toPairs, values, forEach } from 'ramda';
import { execute } from '../../shared/cosmwasm';
import { getBalances, isWithinPercent, sendTokens } from '../helpers';
import { instantiateFinPairContract, instantiateSwapContract, instantiateFundCoreContract } from '../hooks';
import { expect } from '../shared.test';

describe.only('when rebalancing a fund with all the same assets', () => {
  let fundContractAddress: string;
  let swapperContractAddress: string;
  let balancesAfterExecution: Record<string, Record<string, number>>;

  const baseAsset = 'uusk';

  const originalAllocations = {
    ukuji: 0.1,
    udemo: 0.1,
    utest: 0.1,
    uatom: 0.1,
    uosmo: 0.1,
    uaxlusdc: 0.1,
    uusk: 0.1,
    umars: 0.1,
    uweth: 0.1,
    uwbtc: 0.1,
  };

  const newAllocations = {
    ukuji: 0.003,
    udemo: 0.197,
    utest: 0.148,
    uatom: 0.052,
    uosmo: 0.35,
    uaxlusdc: 0.05,
    uusk: 0.0999,
    umars: 0.0001,
    uweth: 0.1,
    uwbtc: 0.0,
  };

  const originalFundTokens = 100000000;

  before(async function (this: Context) {
    swapperContractAddress = await instantiateSwapContract(this.cosmWasmClient, this.adminContractAddress);

    const pairs = map((baseDenom) => ({ baseDenom, quoteDenom: baseAsset }), keys(originalAllocations));

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

    fundContractAddress = await instantiateFundCoreContract(
      this.cosmWasmClient,
      this.adminContractAddress,
      swapperContractAddress,
      baseAsset,
    );

    await sendTokens(
      this.cosmWasmClient,
      this.adminContractAddress,
      fundContractAddress,
      map(([denom, allocation]) => coin(originalFundTokens * allocation, denom), toPairs(originalAllocations)),
    );

    await execute(this.cosmWasmClient, this.adminContractAddress, fundContractAddress, {
      rebalance: {
        allocations: map(([denom, allocation]) => [denom, `${allocation}`], toPairs(newAllocations)),
        slippage_tolerance: null,
        failure_behaviour: null,
      },
    });

    balancesAfterExecution = await getBalances(this.cosmWasmClient, [fundContractAddress], keys(newAllocations));
  });

  it('rebalances the fund correctly', async function (this: Context) {
    const newFundBalances = omit(['address'], balancesAfterExecution[fundContractAddress]);
    const totalFundBalance = reduce((acc, amount) => acc + amount, 0, values(newFundBalances));
    forEach(([denom, allocation]) => {
      expect(isWithinPercent(totalFundBalance, newFundBalances[denom], totalFundBalance * allocation, 2)).to.be.true;
    }, toPairs(newAllocations));
  });
});
