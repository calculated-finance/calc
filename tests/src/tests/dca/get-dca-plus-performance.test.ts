import { coin } from '@cosmjs/proto-signing';
import dayjs from 'dayjs';
import { Context } from 'mocha';
import { execute } from '../../shared/cosmwasm';
import { createVault } from '../helpers';
import { expect } from '../shared.test';

describe('when fetching dca plus performance', () => {
  describe('for a vault with no executions', () => {
    let deposit = coin(1000000, 'ukuji');
    let performance: any;

    before(async function (this: Context) {
      const vault_id = await createVault(
        this,
        {
          target_start_time_utc_seconds: `${dayjs().add(1, 'hour').unix()}`,
          use_dca_plus: true,
        },
        [deposit],
      );

      performance = await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
        get_dca_plus_performance: { vault_id },
      });
    });

    it('has an empty performance fee', async function (this: Context) {
      expect(performance.fee).to.deep.equal(coin(0, 'ukuji'));
    });

    it('has an even performance factor', async function (this: Context) {
      expect(performance.factor).to.equal('1');
    });
  });

  describe('for a vault that performed worse', () => {
    let deposit = coin(1000000, 'ukuji');
    let performance: any;

    before(async function (this: Context) {
      const vault_id = await createVault(
        this,
        {
          use_dca_plus: true,
        },
        [deposit],
      );

      performance = await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
        get_dca_plus_performance: { vault_id },
      });
    });

    it('has an empty performance fee', async function (this: Context) {
      expect(performance.fee).to.deep.equal(coin(0, 'ukuji'));
    });

    it('has an even performance factor', async function (this: Context) {
      expect(performance.factor).to.equal('0.999459027873087939');
    });
  });

  describe('for a vault that performed better', () => {
    let deposit = coin(1000000, 'ukuji');
    let performance: any;

    before(async function (this: Context) {
      await execute(this.cosmWasmClient, this.adminContractAddress, this.dcaContractAddress, {
        update_swap_adjustments: {
          position_type: 'exit',
          adjustments: [
            [30, '0.8'],
            [35, '0.8'],
            [40, '0.8'],
            [45, '0.8'],
            [50, '0.8'],
            [55, '0.8'],
            [60, '0.8'],
            [70, '0.8'],
            [80, '0.8'],
            [90, '0.8'],
          ],
        },
      });

      const vault_id = await createVault(
        this,
        {
          use_dca_plus: true,
        },
        [deposit],
      );

      performance = await this.cosmWasmClient.queryContractSmart(this.dcaContractAddress, {
        get_dca_plus_performance: { vault_id },
      });
    });

    it('has an empty performance fee', async function (this: Context) {
      expect(performance.fee).to.deep.equal(coin(71, 'ukuji'));
    });

    it('has an even performance factor', async function (this: Context) {
      expect(performance.factor).to.equal('1.000359645924079647');
    });
  });
});
