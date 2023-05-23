/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryMsg =
  | {
      get_config: {};
    }
  | {
      get_pairs: {};
    }
  | {
      get_time_trigger_ids: {
        limit?: number | null;
      };
    }
  | {
      get_trigger_id_by_fin_limit_order_idx: {
        order_idx: Uint128;
      };
    }
  | {
      get_vault: {
        vault_id: Uint128;
      };
    }
  | {
      get_vaults_by_address: {
        address: Addr;
        limit?: number | null;
        start_after?: Uint128 | null;
        status?: VaultStatus | null;
      };
    }
  | {
      get_vaults: {
        limit?: number | null;
        reverse?: boolean | null;
        start_after?: Uint128 | null;
      };
    }
  | {
      get_events_by_resource_id: {
        limit?: number | null;
        resource_id: Uint128;
        reverse?: boolean | null;
        start_after?: number | null;
      };
    }
  | {
      get_events: {
        limit?: number | null;
        reverse?: boolean | null;
        start_after?: number | null;
      };
    }
  | {
      get_custom_swap_fees: {};
    }
  | {
      get_vault_performance: {
        vault_id: Uint128;
      };
    }
  | {
      get_disburse_escrow_tasks: {
        limit?: number | null;
      };
    };
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint128 = string;
/**
 * A human readable address.
 *
 * In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.
 *
 * This type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.
 *
 * This type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.
 */
export type Addr = string;
export type VaultStatus = "scheduled" | "active" | "inactive" | "cancelled";
