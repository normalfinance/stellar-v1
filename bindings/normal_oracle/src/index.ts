import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const NormalOracleError = {
  201: {message:"AlreadyInitialized"},
  202: {message:"AssetSupported"},
  203: {message:"AssetNotSupported"},
  204: {message:"FailedToGetOraclePrice"},
  205: {message:"InvalidInput"},
  206: {message:"InvalidOracleSource"}
}


export interface OracleConfig {
  /**
 * Symbol representing the underlying asset being priced (e.g. "BTC", "ETH").
 * Used for metadata and sanity checks by consumers.
 */
asset: string;
  /**
 * Address of the upstream oracle contract providing raw price data.
 * This contract acts as a *proxy / sanitizer* in front of this oracle.
 */
oracle: string;
  /**
 * Declares the type of upstream oracle being used (e.g. Pyth, Chainlink, etc.).
 * This is informational and can also be used by clients to apply
 * source-specific logic if needed.
 */
source: OracleSource;
}


/**
 * Guard-rail parameters applied to raw oracle updates before they are exposed
 * to downstream consumers (Treasury, Pair, etc.).
 * 
 * These values define *when* a price is considered stale or unsafe, and *how*
 * aggressively new oracle prices are clamped relative to historical values.
 */
export interface OracleGuardRails {
  /**
 * Controls how tightly new oracle prices are clamped to historical prices.
 * The allowed band is:
 * ```text
 * last_price ± (last_price / sanitize_clamp_denominator)
 * ```
 * Example:
 * - `sanitize_clamp_denominator = 10` → ±10% per update
 */
sanitize_clamp_denominator: u128;
  /**
 * Maximum age (in seconds) before an oracle price is considered stale.
 * If exceeded, consumers may reject the price or treat the oracle as unhealthy.
 */
seconds_before_stale: u64;
  /**
 * Maximum allowed relative price change between updates, expressed in
 * `PERCENTAGE_PRECISION_U64` units.
 * Used to detect abnormally volatile price jumps that may indicate oracle failure
 * or manipulation.
 */
too_volatile_ratio: u64;
}

/**
 * Persistent data keys for historical oracle state.
 * 
 * Historical data is kept in persistent storage so it survives across
 * ledger boundaries and can be used for TWAPs, volatility checks, and clamping.
 */
export type DataKey = {tag: "Config", values: readonly [string]} | {tag: "HistoricalData", values: readonly [string]} | {tag: "GuardRails", values: readonly [string]};

export const AccessControlError = {
  101: {message:"RoleNotFound"},
  102: {message:"Unauthorized"},
  103: {message:"AdminAlreadySet"},
  104: {message:"BadRoleUsage"},
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export const OracleError = {
  /**
   * OracleError: OracleNonPositive
   */
  601: {message:"OracleNonPositive"},
  602: {message:"OracleTooVolatile"},
  603: {message:"OracleStaleForPair"},
  604: {message:"OracleInvalid"},
  605: {message:"FailedToGetOraclePrice"},
  606: {message:"InvalidInput"}
}

export type OracleValidity = {tag: "NonPositive", values: void} | {tag: "TooVolatile", values: void} | {tag: "StaleForPair", values: void} | {tag: "Frozen", values: void} | {tag: "Valid", values: void};


export interface HistoricalOracleData {
  last_delay_ts: u64;
  last_price: u128;
  last_price_twap: u128;
  last_update_ts: u64;
}


/**
 * Price data for an asset at a specific timestamp
 */
export interface PriceData {
  price: i128;
  timestamp: u64;
}

/**
 * Asset type
 */
export type Asset = {tag: "Stellar", values: readonly [string]} | {tag: "Other", values: readonly [string]};


export interface OraclePriceData {
  delay: Delay;
  price: u128;
}

export type OracleSource = {tag: "Reflector", values: void};


export interface PairParams {
  admin: string;
  asset: string;
  calculator: string;
  collateral_per_pair: u128;
  collateral_token: string;
  long_token: string;
  lower_bound: u128;
  oracle: string;
  short_token: string;
  upper_bound: u128;
}

export type Side = {tag: "Long", values: void} | {tag: "Short", values: void};

export type Direction = {tag: "Buy", values: void} | {tag: "Sell", values: void};


export interface PairPriceBounds {
  lower: u128;
  upper: u128;
}


export interface PairAmounts {
  long: u128;
  short: u128;
}


export interface PairAmountsWithUSDC {
  long: u128;
  short: u128;
  usdc: u128;
}


export interface PairTokens {
  collateral: string;
  long: string;
  short: string;
}

export type PairStatus = {tag: "Inactive", values: void} | {tag: "Active", values: void} | {tag: "Expired", values: void};


export interface CollateralInfo {
  collateral_per_pair: u128;
  collateral_percent_long: u128;
  collateral_token: string;
  total_collateral: u128;
}


export interface PairSummary {
  asset: string;
  calculator: string;
  collateral: CollateralInfo;
  oracle: string;
  price_bounds: PairPriceBounds;
  status: PairStatus;
  tokens: PairTokens;
}

export const Errors = {
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export const MathError = {
  /**
   * MathError: NumberOverflow
   */
  510: {message:"NumberOverflow"},
  /**
   * MathError: Generic math error
   */
  511: {message:"MathError"},
  /**
   * MathError: Addition operation caused overflow
   */
  512: {message:"AdditionOverflow"},
  /**
   * MathError: Subtraction operation caused underflow
   */
  513: {message:"SubtractionUnderflow"},
  /**
   * MathError: Multiplication operation caused overflow
   */
  514: {message:"MultiplicationOverflow"},
  /**
   * MathError: Division by zero
   */
  515: {message:"DivisionByZero"},
  /**
   * MathError: Type conversion overflow
   */
  516: {message:"ConversionOverflow"},
  /**
   * MathError: Attempted to convert negative value to unsigned type
   */
  517: {message:"NegativeToUnsigned"},
  /**
   * MathError: Fixed-point arithmetic overflow
   */
  518: {message:"FixedPointOverflow"}
}

export const StorageError = {
  /**
   * StorageError
   */
  201: {message:"AlreadyInitialized"},
  501: {message:"ValueNotInitialized"},
  502: {message:"ValueMissing"},
  503: {message:"ValueConversionError"}
}

export const ValidationError = {
  /**
   * ValidationError
   */
  801: {message:"InvalidToken"},
  802: {message:"InvalidPercentage"},
  804: {message:"ZeroAmount"},
  805: {message:"InvalidOracleTimestamp"}
}

export type Delay = readonly [u64];

export interface Client {
  /**
   * Construct and simulate a get_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_oracle_price: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<OraclePriceData>>

  /**
   * Construct and simulate a get_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_price: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<HistoricalOracleData>>

  /**
   * Construct and simulate a get_price_and_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_price_and_update: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<HistoricalOracleData>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<OracleConfig>>

  /**
   * Construct and simulate a get_guard_rails transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_guard_rails: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<OracleGuardRails>>

  /**
   * Construct and simulate a add_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_asset: ({admin, config, guard_rails}: {admin: string, config: OracleConfig, guard_rails: OracleGuardRails}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a remove_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_asset: ({admin, asset}: {admin: string, asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_guard_rails transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_guard_rails: ({admin, asset, guard_rails}: {admin: string, asset: string, guard_rails: OracleGuardRails}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  version: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a contract_name transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  contract_name: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a commit_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_upgrade: ({admin, new_wasm_hash}: {admin: string, new_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a apply_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_upgrade: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a revert_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_upgrade: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_emergency_mode: ({emergency_admin, value}: {emergency_admin: string, value: boolean}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_emergency_mode: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a commit_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_transfer_ownership: ({admin, role_name, new_address}: {admin: string, role_name: string, new_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a apply_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a revert_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_future_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_future_address: ({role_name}: {role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin}: {admin: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAQZ2V0X29yYWNsZV9wcmljZQAAAAEAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAEAAAfQAAAAD09yYWNsZVByaWNlRGF0YQA=",
        "AAAAAAAAAAAAAAAJZ2V0X3ByaWNlAAAAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAQAAB9AAAAAUSGlzdG9yaWNhbE9yYWNsZURhdGE=",
        "AAAAAAAAAAAAAAAUZ2V0X3ByaWNlX2FuZF91cGRhdGUAAAABAAAAAAAAAAVhc3NldAAAAAAAABEAAAABAAAH0AAAABRIaXN0b3JpY2FsT3JhY2xlRGF0YQ==",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAQAAB9AAAAAMT3JhY2xlQ29uZmln",
        "AAAAAAAAAAAAAAAPZ2V0X2d1YXJkX3JhaWxzAAAAAAEAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAEAAAfQAAAAEE9yYWNsZUd1YXJkUmFpbHM=",
        "AAAAAAAAAAAAAAAJYWRkX2Fzc2V0AAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZjb25maWcAAAAAB9AAAAAMT3JhY2xlQ29uZmlnAAAAAAAAAAtndWFyZF9yYWlscwAAAAfQAAAAEE9yYWNsZUd1YXJkUmFpbHMAAAAA",
        "AAAAAAAAAAAAAAAMcmVtb3ZlX2Fzc2V0AAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAVhc3NldAAAAAAAABEAAAAA",
        "AAAAAAAAAAAAAAAPc2V0X2d1YXJkX3JhaWxzAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAAAAAAtndWFyZF9yYWlscwAAAAfQAAAAEE9yYWNsZUd1YXJkUmFpbHMAAAAA",
        "AAAAAAAAAAAAAAAHdmVyc2lvbgAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAANY29udHJhY3RfbmFtZQAAAAAAAAAAAAABAAAAEQ==",
        "AAAAAAAAAAAAAAAOY29tbWl0X3VwZ3JhZGUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAANYXBwbHlfdXBncmFkZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAOcmV2ZXJ0X3VwZ3JhZGUAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASc2V0X2VtZXJnZW5jeV9tb2RlAAAAAAACAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAFdmFsdWUAAAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X2VtZXJnZW5jeV9tb2RlAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAZY29tbWl0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAAAAAALbmV3X2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYYXBwbHlfdHJhbnNmZXJfb3duZXJzaGlwAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAA==",
        "AAAAAAAAAAAAAAAZcmV2ZXJ0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X2Z1dHVyZV9hZGRyZXNzAAAAAAABAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAQAAABM=",
        "AAAABAAAAAAAAAAAAAAAEU5vcm1hbE9yYWNsZUVycm9yAAAAAAAABgAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAADJAAAAAAAAAA5Bc3NldFN1cHBvcnRlZAAAAAAAygAAAAAAAAARQXNzZXROb3RTdXBwb3J0ZWQAAAAAAADLAAAAAAAAABZGYWlsZWRUb0dldE9yYWNsZVByaWNlAAAAAADMAAAAAAAAAAxJbnZhbGlkSW5wdXQAAADNAAAAAAAAABNJbnZhbGlkT3JhY2xlU291cmNlAAAAAM4=",
        "AAAAAQAAAAAAAAAAAAAADE9yYWNsZUNvbmZpZwAAAAMAAAB8U3ltYm9sIHJlcHJlc2VudGluZyB0aGUgdW5kZXJseWluZyBhc3NldCBiZWluZyBwcmljZWQgKGUuZy4gIkJUQyIsICJFVEgiKS4KVXNlZCBmb3IgbWV0YWRhdGEgYW5kIHNhbml0eSBjaGVja3MgYnkgY29uc3VtZXJzLgAAAAVhc3NldAAAAAAAABEAAACGQWRkcmVzcyBvZiB0aGUgdXBzdHJlYW0gb3JhY2xlIGNvbnRyYWN0IHByb3ZpZGluZyByYXcgcHJpY2UgZGF0YS4KVGhpcyBjb250cmFjdCBhY3RzIGFzIGEgKnByb3h5IC8gc2FuaXRpemVyKiBpbiBmcm9udCBvZiB0aGlzIG9yYWNsZS4AAAAAAAZvcmFjbGUAAAAAABMAAACtRGVjbGFyZXMgdGhlIHR5cGUgb2YgdXBzdHJlYW0gb3JhY2xlIGJlaW5nIHVzZWQgKGUuZy4gUHl0aCwgQ2hhaW5saW5rLCBldGMuKS4KVGhpcyBpcyBpbmZvcm1hdGlvbmFsIGFuZCBjYW4gYWxzbyBiZSB1c2VkIGJ5IGNsaWVudHMgdG8gYXBwbHkKc291cmNlLXNwZWNpZmljIGxvZ2ljIGlmIG5lZWRlZC4AAAAAAAAGc291cmNlAAAAAAfQAAAADE9yYWNsZVNvdXJjZQ==",
        "AAAAAQAAARJHdWFyZC1yYWlsIHBhcmFtZXRlcnMgYXBwbGllZCB0byByYXcgb3JhY2xlIHVwZGF0ZXMgYmVmb3JlIHRoZXkgYXJlIGV4cG9zZWQKdG8gZG93bnN0cmVhbSBjb25zdW1lcnMgKFRyZWFzdXJ5LCBQYWlyLCBldGMuKS4KClRoZXNlIHZhbHVlcyBkZWZpbmUgKndoZW4qIGEgcHJpY2UgaXMgY29uc2lkZXJlZCBzdGFsZSBvciB1bnNhZmUsIGFuZCAqaG93KgphZ2dyZXNzaXZlbHkgbmV3IG9yYWNsZSBwcmljZXMgYXJlIGNsYW1wZWQgcmVsYXRpdmUgdG8gaGlzdG9yaWNhbCB2YWx1ZXMuAAAAAAAAAAAAEE9yYWNsZUd1YXJkUmFpbHMAAAADAAAA40NvbnRyb2xzIGhvdyB0aWdodGx5IG5ldyBvcmFjbGUgcHJpY2VzIGFyZSBjbGFtcGVkIHRvIGhpc3RvcmljYWwgcHJpY2VzLgpUaGUgYWxsb3dlZCBiYW5kIGlzOgpgYGB0ZXh0Cmxhc3RfcHJpY2UgwrEgKGxhc3RfcHJpY2UgLyBzYW5pdGl6ZV9jbGFtcF9kZW5vbWluYXRvcikKYGBgCkV4YW1wbGU6Ci0gYHNhbml0aXplX2NsYW1wX2Rlbm9taW5hdG9yID0gMTBgIOKGkiDCsTEwJSBwZXIgdXBkYXRlAAAAABpzYW5pdGl6ZV9jbGFtcF9kZW5vbWluYXRvcgAAAAAACgAAAJJNYXhpbXVtIGFnZSAoaW4gc2Vjb25kcykgYmVmb3JlIGFuIG9yYWNsZSBwcmljZSBpcyBjb25zaWRlcmVkIHN0YWxlLgpJZiBleGNlZWRlZCwgY29uc3VtZXJzIG1heSByZWplY3QgdGhlIHByaWNlIG9yIHRyZWF0IHRoZSBvcmFjbGUgYXMgdW5oZWFsdGh5LgAAAAAAFHNlY29uZHNfYmVmb3JlX3N0YWxlAAAABgAAAMZNYXhpbXVtIGFsbG93ZWQgcmVsYXRpdmUgcHJpY2UgY2hhbmdlIGJldHdlZW4gdXBkYXRlcywgZXhwcmVzc2VkIGluCmBQRVJDRU5UQUdFX1BSRUNJU0lPTl9VNjRgIHVuaXRzLgpVc2VkIHRvIGRldGVjdCBhYm5vcm1hbGx5IHZvbGF0aWxlIHByaWNlIGp1bXBzIHRoYXQgbWF5IGluZGljYXRlIG9yYWNsZSBmYWlsdXJlCm9yIG1hbmlwdWxhdGlvbi4AAAAAABJ0b29fdm9sYXRpbGVfcmF0aW8AAAAAAAY=",
        "AAAAAgAAAMRQZXJzaXN0ZW50IGRhdGEga2V5cyBmb3IgaGlzdG9yaWNhbCBvcmFjbGUgc3RhdGUuCgpIaXN0b3JpY2FsIGRhdGEgaXMga2VwdCBpbiBwZXJzaXN0ZW50IHN0b3JhZ2Ugc28gaXQgc3Vydml2ZXMgYWNyb3NzCmxlZGdlciBib3VuZGFyaWVzIGFuZCBjYW4gYmUgdXNlZCBmb3IgVFdBUHMsIHZvbGF0aWxpdHkgY2hlY2tzLCBhbmQgY2xhbXBpbmcuAAAAAAAAAAdEYXRhS2V5AAAAAAMAAAABAAAAAAAAAAZDb25maWcAAAAAAAEAAAARAAAAAQAAAEdTdG9yZXMgdGhlIHJvbGxpbmcgb3JhY2xlIGhpc3RvcnkgKFRXQVAsIGxhc3QgcHJpY2UsIHRpbWVzdGFtcHMsIGV0Yy4pLgAAAAAOSGlzdG9yaWNhbERhdGEAAAAAAAEAAAARAAAAAQAAAAAAAAAKR3VhcmRSYWlscwAAAAAAAQAAABE=",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAAAAAAAAMUm9sZU5vdEZvdW5kAAAAZQAAAAAAAAAMVW5hdXRob3JpemVkAAAAZgAAAAAAAAAPQWRtaW5BbHJlYWR5U2V0AAAAAGcAAAAAAAAADEJhZFJvbGVVc2FnZQAAAGgAAAAAAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAABAAAAAAAAAAAAAAAC09yYWNsZUVycm9yAAAAAAYAAAAeT3JhY2xlRXJyb3I6IE9yYWNsZU5vblBvc2l0aXZlAAAAAAART3JhY2xlTm9uUG9zaXRpdmUAAAAAAAJZAAAAAAAAABFPcmFjbGVUb29Wb2xhdGlsZQAAAAAAAloAAAAAAAAAEk9yYWNsZVN0YWxlRm9yUGFpcgAAAAACWwAAAAAAAAANT3JhY2xlSW52YWxpZAAAAAAAAlwAAAAAAAAAFkZhaWxlZFRvR2V0T3JhY2xlUHJpY2UAAAAAAl0AAAAAAAAADEludmFsaWRJbnB1dAAAAl4=",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQYWlyAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAABAAAAAAAAAANbGFzdF9kZWxheV90cwAAAAAAAAYAAAAAAAAACmxhc3RfcHJpY2UAAAAAAAoAAAAAAAAAD2xhc3RfcHJpY2VfdHdhcAAAAAAKAAAAAAAAAA5sYXN0X3VwZGF0ZV90cwAAAAAABg==",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR",
        "AAAAAQAAAAAAAAAAAAAAD09yYWNsZVByaWNlRGF0YQAAAAACAAAAAAAAAAVkZWxheQAAAAAAB9AAAAAFRGVsYXkAAAAAAAAAAAAABXByaWNlAAAAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAADE9yYWNsZVNvdXJjZQAAAAEAAAAAAAAAAAAAAAlSZWZsZWN0b3IAAAA=",
        "AAAAAQAAAAAAAAAAAAAAClBhaXJQYXJhbXMAAAAAAAoAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAAAAAApjYWxjdWxhdG9yAAAAAAATAAAAAAAAABNjb2xsYXRlcmFsX3Blcl9wYWlyAAAAAAoAAAAAAAAAEGNvbGxhdGVyYWxfdG9rZW4AAAATAAAAAAAAAApsb25nX3Rva2VuAAAAAAATAAAAAAAAAAtsb3dlcl9ib3VuZAAAAAAKAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAAC3Nob3J0X3Rva2VuAAAAABMAAAAAAAAAC3VwcGVyX2JvdW5kAAAAAAo=",
        "AAAAAgAAAAAAAAAAAAAABFNpZGUAAAACAAAAAAAAAAAAAAAETG9uZwAAAAAAAAAAAAAABVNob3J0AAAA",
        "AAAAAgAAAAAAAAAAAAAACURpcmVjdGlvbgAAAAAAAAIAAAAAAAAAAAAAAANCdXkAAAAAAAAAAAAAAAAEU2VsbA==",
        "AAAAAQAAAAAAAAAAAAAAD1BhaXJQcmljZUJvdW5kcwAAAAACAAAAAAAAAAVsb3dlcgAAAAAAAAoAAAAAAAAABXVwcGVyAAAAAAAACg==",
        "AAAAAQAAAAAAAAAAAAAAC1BhaXJBbW91bnRzAAAAAAIAAAAAAAAABGxvbmcAAAAKAAAAAAAAAAVzaG9ydAAAAAAAAAo=",
        "AAAAAQAAAAAAAAAAAAAAE1BhaXJBbW91bnRzV2l0aFVTREMAAAAAAwAAAAAAAAAEbG9uZwAAAAoAAAAAAAAABXNob3J0AAAAAAAACgAAAAAAAAAEdXNkYwAAAAo=",
        "AAAAAQAAAAAAAAAAAAAAClBhaXJUb2tlbnMAAAAAAAMAAAAAAAAACmNvbGxhdGVyYWwAAAAAABMAAAAAAAAABGxvbmcAAAATAAAAAAAAAAVzaG9ydAAAAAAAABM=",
        "AAAAAgAAAAAAAAAAAAAAClBhaXJTdGF0dXMAAAAAAAMAAAAAAAAAAAAAAAhJbmFjdGl2ZQAAAAAAAAAAAAAABkFjdGl2ZQAAAAAAAAAAAAAAAAAHRXhwaXJlZAA=",
        "AAAAAQAAAAAAAAAAAAAADkNvbGxhdGVyYWxJbmZvAAAAAAAEAAAAAAAAABNjb2xsYXRlcmFsX3Blcl9wYWlyAAAAAAoAAAAAAAAAF2NvbGxhdGVyYWxfcGVyY2VudF9sb25nAAAAAAoAAAAAAAAAEGNvbGxhdGVyYWxfdG9rZW4AAAATAAAAAAAAABB0b3RhbF9jb2xsYXRlcmFsAAAACg==",
        "AAAAAQAAAAAAAAAAAAAAC1BhaXJTdW1tYXJ5AAAAAAcAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAAAAAAKY2FsY3VsYXRvcgAAAAAAEwAAAAAAAAAKY29sbGF0ZXJhbAAAAAAH0AAAAA5Db2xsYXRlcmFsSW5mbwAAAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAADHByaWNlX2JvdW5kcwAAB9AAAAAPUGFpclByaWNlQm91bmRzAAAAAAAAAAAGc3RhdHVzAAAAAAfQAAAAClBhaXJTdGF0dXMAAAAAAAAAAAAGdG9rZW5zAAAAAAfQAAAAClBhaXJUb2tlbnMAAA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAAwAAAAAAAAATQW5vdGhlckFjdGlvbkFjdGl2ZQAAAAtaAAAAAAAAAA5Ob0FjdGlvbkFjdGl2ZQAAAAALWwAAAAAAAAARQWN0aW9uTm90UmVhZHlZZXQAAAAAAAtc",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAkAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAB1NYXRoRXJyb3I6IEdlbmVyaWMgbWF0aCBlcnJvcgAAAAAAAAlNYXRoRXJyb3IAAAAAAAH/AAAALU1hdGhFcnJvcjogQWRkaXRpb24gb3BlcmF0aW9uIGNhdXNlZCBvdmVyZmxvdwAAAAAAABBBZGRpdGlvbk92ZXJmbG93AAACAAAAADFNYXRoRXJyb3I6IFN1YnRyYWN0aW9uIG9wZXJhdGlvbiBjYXVzZWQgdW5kZXJmbG93AAAAAAAAFFN1YnRyYWN0aW9uVW5kZXJmbG93AAACAQAAADNNYXRoRXJyb3I6IE11bHRpcGxpY2F0aW9uIG9wZXJhdGlvbiBjYXVzZWQgb3ZlcmZsb3cAAAAAFk11bHRpcGxpY2F0aW9uT3ZlcmZsb3cAAAAAAgIAAAAbTWF0aEVycm9yOiBEaXZpc2lvbiBieSB6ZXJvAAAAAA5EaXZpc2lvbkJ5WmVybwAAAAACAwAAACNNYXRoRXJyb3I6IFR5cGUgY29udmVyc2lvbiBvdmVyZmxvdwAAAAASQ29udmVyc2lvbk92ZXJmbG93AAAAAAIEAAAAP01hdGhFcnJvcjogQXR0ZW1wdGVkIHRvIGNvbnZlcnQgbmVnYXRpdmUgdmFsdWUgdG8gdW5zaWduZWQgdHlwZQAAAAASTmVnYXRpdmVUb1Vuc2lnbmVkAAAAAAIFAAAAKk1hdGhFcnJvcjogRml4ZWQtcG9pbnQgYXJpdGhtZXRpYyBvdmVyZmxvdwAAAAAAEkZpeGVkUG9pbnRPdmVyZmxvdwAAAAACBg==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAQAAAAMU3RvcmFnZUVycm9yAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAyQAAAAAAAAATVmFsdWVOb3RJbml0aWFsaXplZAAAAAH1AAAAAAAAAAxWYWx1ZU1pc3NpbmcAAAH2AAAAAAAAABRWYWx1ZUNvbnZlcnNpb25FcnJvcgAAAfc=",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAEAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQAAAAAAAAARSW52YWxpZFBlcmNlbnRhZ2UAAAAAAAMiAAAAAAAAAApaZXJvQW1vdW50AAAAAAMkAAAAAAAAABZJbnZhbGlkT3JhY2xlVGltZXN0YW1wAAAAAAMl",
        "AAAAAQAAAAAAAAAAAAAABURlbGF5AAAAAAAAAQAAAAAAAAABMAAAAAAAAAY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_oracle_price: this.txFromJSON<OraclePriceData>,
        get_price: this.txFromJSON<HistoricalOracleData>,
        get_price_and_update: this.txFromJSON<HistoricalOracleData>,
        get_config: this.txFromJSON<OracleConfig>,
        get_guard_rails: this.txFromJSON<OracleGuardRails>,
        add_asset: this.txFromJSON<null>,
        remove_asset: this.txFromJSON<null>,
        set_guard_rails: this.txFromJSON<null>,
        version: this.txFromJSON<u32>,
        contract_name: this.txFromJSON<string>,
        commit_upgrade: this.txFromJSON<null>,
        apply_upgrade: this.txFromJSON<Buffer>,
        revert_upgrade: this.txFromJSON<null>,
        set_emergency_mode: this.txFromJSON<null>,
        get_emergency_mode: this.txFromJSON<boolean>,
        commit_transfer_ownership: this.txFromJSON<null>,
        apply_transfer_ownership: this.txFromJSON<null>,
        revert_transfer_ownership: this.txFromJSON<null>,
        get_future_address: this.txFromJSON<string>
  }
}