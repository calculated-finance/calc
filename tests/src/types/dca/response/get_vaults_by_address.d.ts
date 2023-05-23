/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

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
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp = Uint64;
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;
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
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
export type PerformanceAssessmentStrategy = {
  compare_to_standard_dca: {
    received_amount: Coin;
    swapped_amount: Coin;
  };
};
export type VaultStatus = "scheduled" | "active" | "inactive" | "cancelled";
export type SwapAdjustmentStrategy =
  | {
      risk_weighted_average: {
        base_denom: BaseDenom;
        model_id: number;
        position_type: PositionType;
      };
    }
  | {
      weighted_scale: {
        base_receive_amount: Uint128;
        increase_only: boolean;
        multiplier: Decimal;
      };
    };
export type BaseDenom = "bitcoin";
export type PositionType = "enter" | "exit";
export type TimeInterval =
  | (
      | "every_second"
      | "every_minute"
      | "half_hourly"
      | "hourly"
      | "half_daily"
      | "daily"
      | "weekly"
      | "fortnightly"
      | "monthly"
    )
  | {
      custom: {
        seconds: number;
      };
    };
export type TriggerConfiguration =
  | {
      time: {
        target_time: Timestamp;
      };
    }
  | {
      price: {
        order_idx?: Uint128 | null;
        target_price: Decimal;
      };
    };

export interface VaultsResponse {
  vaults: Vault[];
}
export interface Vault {
  balance: Coin;
  created_at: Timestamp;
  deposited_amount: Coin;
  destinations: Destination[];
  escrow_level: Decimal;
  escrowed_amount: Coin;
  id: Uint128;
  label?: string | null;
  minimum_receive_amount?: Uint128 | null;
  owner: Addr;
  performance_assessment_strategy?: PerformanceAssessmentStrategy | null;
  received_amount: Coin;
  slippage_tolerance: Decimal;
  started_at?: Timestamp | null;
  status: VaultStatus;
  swap_adjustment_strategy?: SwapAdjustmentStrategy | null;
  swap_amount: Uint128;
  swapped_amount: Coin;
  target_denom: string;
  time_interval: TimeInterval;
  trigger?: TriggerConfiguration | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface Destination {
  address: Addr;
  allocation: Decimal;
  msg?: Binary | null;
}
