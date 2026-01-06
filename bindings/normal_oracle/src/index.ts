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
  202: {message:"InvalidToken"},
  203: {message:"InsufficientBalance"}
}

export type DataKey = {tag: "Asset", values: void} | {tag: "OracleSource", values: void} | {tag: "Oracle", values: void} | {tag: "HistoricalData", values: void} | {tag: "SecondsBeforeStale", values: void} | {tag: "TooVolatileRatio", values: void};


export interface GuardRails {
  seconds_before_stale: u64;
  too_volatile_ratio: u64;
}

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
  603: {message:"OracleStaleForPool"},
  604: {message:"OracleInvalid"},
  605: {message:"FailedToGetOraclePrice"}
}

export type OracleValidity = {tag: "NonPositive", values: void} | {tag: "TooVolatile", values: void} | {tag: "StaleForPool", values: void} | {tag: "Frozen", values: void} | {tag: "Valid", values: void};


export interface HistoricalOracleData {
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
  collateral_per_pair: u128;
  collateral_token: string;
  long_token_serialized_info: Buffer;
  lower_bound: u128;
  oracle: string;
  pair_calculator: string;
  pair_token_wasm_hash: Buffer;
  short_token_serialized_info: Buffer;
  upper_bound: u128;
}

export type Direction = {tag: "Long", values: void} | {tag: "Short", values: void};


export interface LinearLongShortPairParameters {
  lower_bound: u128;
  upper_bound: u128;
}

export type PairStatus = {tag: "Active", values: void} | {tag: "Settlement", values: void} | {tag: "Inactive", values: void};


export interface CollateralInfo {
  collateral_per_pair: u128;
  collateral_percent_long: u64;
}


export interface PairSummary {
  calculator: string;
  collateral: CollateralInfo;
  collateral_token: string;
  long_token: string;
  oracle: string;
  price_bounds: readonly [u128, u128];
  status: PairStatus;
}


export interface PoolParams {
  admin: string;
  assets_config: readonly [string, string];
  direction: Direction;
  fees_config: readonly [u32, u32];
  long_short_pair: string;
  lp_token_wasm_hash: Buffer;
  privileged_addrs: readonly [string, string, string, string, Array<string>, string];
  router: string;
  tokens: Array<string>;
}


export interface RateTableEntry {
  deviation: u128;
  rate: u32;
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
  804: {message:"ZeroAmount"}
}

export type Delay = readonly [u64];

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, asset, oracle_source, oracle_addr}: {admin: string, asset: string, oracle_source: OracleSource, oracle_addr: string}, options?: {
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
   * Construct and simulate a get_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_price: (options?: {
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
   * Construct and simulate a set_seconds_before_stale transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_seconds_before_stale: ({admin, stale_limit}: {admin: string, stale_limit: u64}, options?: {
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
   * Construct and simulate a set_too_volatile_ratio transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_too_volatile_ratio: ({admin, too_volatile_ratio}: {admin: string, too_volatile_ratio: u64}, options?: {
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
   * Construct and simulate a get_guard_rails transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_guard_rails: (options?: {
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
  }) => Promise<AssembledTransaction<GuardRails>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
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
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAABAAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAVhc3NldAAAAAAAABEAAAAAAAAADW9yYWNsZV9zb3VyY2UAAAAAAAfQAAAADE9yYWNsZVNvdXJjZQAAAAAAAAALb3JhY2xlX2FkZHIAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAJZ2V0X3ByaWNlAAAAAAAAAAAAAAEAAAfQAAAAD09yYWNsZVByaWNlRGF0YQA=",
        "AAAAAAAAAAAAAAAYc2V0X3NlY29uZHNfYmVmb3JlX3N0YWxlAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAtzdGFsZV9saW1pdAAAAAAGAAAAAA==",
        "AAAAAAAAAAAAAAAWc2V0X3Rvb192b2xhdGlsZV9yYXRpbwAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABJ0b29fdm9sYXRpbGVfcmF0aW8AAAAAAAYAAAAA",
        "AAAAAAAAAAAAAAAPZ2V0X2d1YXJkX3JhaWxzAAAAAAAAAAABAAAH0AAAAApHdWFyZFJhaWxzAAA=",
        "AAAABAAAAAAAAAAAAAAAEU5vcm1hbE9yYWNsZUVycm9yAAAAAAAAAwAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAADJAAAAAAAAAAxJbnZhbGlkVG9rZW4AAADKAAAAAAAAABNJbnN1ZmZpY2llbnRCYWxhbmNlAAAAAMs=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABgAAAAAAAAAAAAAABUFzc2V0AAAAAAAAAAAAAAAAAAAMT3JhY2xlU291cmNlAAAAAAAAAAAAAAAGT3JhY2xlAAAAAAAAAAAAAAAAAA5IaXN0b3JpY2FsRGF0YQAAAAAAAAAAAAAAAAASU2Vjb25kc0JlZm9yZVN0YWxlAAAAAAAAAAAAAAAAABBUb29Wb2xhdGlsZVJhdGlv",
        "AAAAAQAAAAAAAAAAAAAACkd1YXJkUmFpbHMAAAAAAAIAAAAAAAAAFHNlY29uZHNfYmVmb3JlX3N0YWxlAAAABgAAAAAAAAASdG9vX3ZvbGF0aWxlX3JhdGlvAAAAAAAG",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAAAAAAAAMUm9sZU5vdEZvdW5kAAAAZQAAAAAAAAAMVW5hdXRob3JpemVkAAAAZgAAAAAAAAAPQWRtaW5BbHJlYWR5U2V0AAAAAGcAAAAAAAAADEJhZFJvbGVVc2FnZQAAAGgAAAAAAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAABAAAAAAAAAAAAAAAC09yYWNsZUVycm9yAAAAAAUAAAAeT3JhY2xlRXJyb3I6IE9yYWNsZU5vblBvc2l0aXZlAAAAAAART3JhY2xlTm9uUG9zaXRpdmUAAAAAAAJZAAAAAAAAABFPcmFjbGVUb29Wb2xhdGlsZQAAAAAAAloAAAAAAAAAEk9yYWNsZVN0YWxlRm9yUG9vbAAAAAACWwAAAAAAAAANT3JhY2xlSW52YWxpZAAAAAAAAlwAAAAAAAAAFkZhaWxlZFRvR2V0T3JhY2xlUHJpY2UAAAAAAl0=",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQb29sAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAAAwAAAAAAAAAKbGFzdF9wcmljZQAAAAAACgAAAAAAAAAPbGFzdF9wcmljZV90d2FwAAAAAAoAAAAAAAAADmxhc3RfdXBkYXRlX3RzAAAAAAAG",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR",
        "AAAAAQAAAAAAAAAAAAAAD09yYWNsZVByaWNlRGF0YQAAAAACAAAAAAAAAAVkZWxheQAAAAAAB9AAAAAFRGVsYXkAAAAAAAAAAAAABXByaWNlAAAAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAADE9yYWNsZVNvdXJjZQAAAAEAAAAAAAAAAAAAAAlSZWZsZWN0b3IAAAA=",
        "AAAAAQAAAAAAAAAAAAAAClBhaXJQYXJhbXMAAAAAAAsAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAAAAABNjb2xsYXRlcmFsX3Blcl9wYWlyAAAAAAoAAAAAAAAAEGNvbGxhdGVyYWxfdG9rZW4AAAATAAAAAAAAABpsb25nX3Rva2VuX3NlcmlhbGl6ZWRfaW5mbwAAAAAADgAAAAAAAAALbG93ZXJfYm91bmQAAAAACgAAAAAAAAAGb3JhY2xlAAAAAAATAAAAAAAAAA9wYWlyX2NhbGN1bGF0b3IAAAAAEwAAAAAAAAAUcGFpcl90b2tlbl93YXNtX2hhc2gAAAPuAAAAIAAAAAAAAAAbc2hvcnRfdG9rZW5fc2VyaWFsaXplZF9pbmZvAAAAAA4AAAAAAAAAC3VwcGVyX2JvdW5kAAAAAAo=",
        "AAAAAgAAAAAAAAAAAAAACURpcmVjdGlvbgAAAAAAAAIAAAAAAAAAAAAAAARMb25nAAAAAAAAAAAAAAAFU2hvcnQAAAA=",
        "AAAAAQAAAAAAAAAAAAAAHUxpbmVhckxvbmdTaG9ydFBhaXJQYXJhbWV0ZXJzAAAAAAAAAgAAAAAAAAALbG93ZXJfYm91bmQAAAAACgAAAAAAAAALdXBwZXJfYm91bmQAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAAClBhaXJTdGF0dXMAAAAAAAMAAAAAAAAAAAAAAAZBY3RpdmUAAAAAAAAAAAAAAAAAClNldHRsZW1lbnQAAAAAAAAAAAAAAAAACEluYWN0aXZl",
        "AAAAAQAAAAAAAAAAAAAADkNvbGxhdGVyYWxJbmZvAAAAAAACAAAAAAAAABNjb2xsYXRlcmFsX3Blcl9wYWlyAAAAAAoAAAAAAAAAF2NvbGxhdGVyYWxfcGVyY2VudF9sb25nAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAAC1BhaXJTdW1tYXJ5AAAAAAcAAAAAAAAACmNhbGN1bGF0b3IAAAAAABMAAAAAAAAACmNvbGxhdGVyYWwAAAAAB9AAAAAOQ29sbGF0ZXJhbEluZm8AAAAAAAAAAAAQY29sbGF0ZXJhbF90b2tlbgAAABMAAAAAAAAACmxvbmdfdG9rZW4AAAAAABMAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAMcHJpY2VfYm91bmRzAAAD7QAAAAIAAAAKAAAACgAAAAAAAAAGc3RhdHVzAAAAAAfQAAAAClBhaXJTdGF0dXMAAA==",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xQYXJhbXMAAAAAAAkAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANYXNzZXRzX2NvbmZpZwAAAAAAA+0AAAACAAAAEQAAABEAAAAAAAAACWRpcmVjdGlvbgAAAAAAB9AAAAAJRGlyZWN0aW9uAAAAAAAAAAAAAAtmZWVzX2NvbmZpZwAAAAPtAAAAAgAAAAQAAAAEAAAAAAAAAA9sb25nX3Nob3J0X3BhaXIAAAAAEwAAAAAAAAASbHBfdG9rZW5fd2FzbV9oYXNoAAAAAAPuAAAAIAAAAAAAAAAQcHJpdmlsZWdlZF9hZGRycwAAA+0AAAAGAAAAEwAAABMAAAATAAAAEwAAA+oAAAATAAAAEwAAAAAAAAAGcm91dGVyAAAAAAATAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAAT",
        "AAAAAQAAAAAAAAAAAAAADlJhdGVUYWJsZUVudHJ5AAAAAAACAAAAAAAAAAlkZXZpYXRpb24AAAAAAAAKAAAAAAAAAARyYXRlAAAABA==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAkAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAB1NYXRoRXJyb3I6IEdlbmVyaWMgbWF0aCBlcnJvcgAAAAAAAAlNYXRoRXJyb3IAAAAAAAH/AAAALU1hdGhFcnJvcjogQWRkaXRpb24gb3BlcmF0aW9uIGNhdXNlZCBvdmVyZmxvdwAAAAAAABBBZGRpdGlvbk92ZXJmbG93AAACAAAAADFNYXRoRXJyb3I6IFN1YnRyYWN0aW9uIG9wZXJhdGlvbiBjYXVzZWQgdW5kZXJmbG93AAAAAAAAFFN1YnRyYWN0aW9uVW5kZXJmbG93AAACAQAAADNNYXRoRXJyb3I6IE11bHRpcGxpY2F0aW9uIG9wZXJhdGlvbiBjYXVzZWQgb3ZlcmZsb3cAAAAAFk11bHRpcGxpY2F0aW9uT3ZlcmZsb3cAAAAAAgIAAAAbTWF0aEVycm9yOiBEaXZpc2lvbiBieSB6ZXJvAAAAAA5EaXZpc2lvbkJ5WmVybwAAAAACAwAAACNNYXRoRXJyb3I6IFR5cGUgY29udmVyc2lvbiBvdmVyZmxvdwAAAAASQ29udmVyc2lvbk92ZXJmbG93AAAAAAIEAAAAP01hdGhFcnJvcjogQXR0ZW1wdGVkIHRvIGNvbnZlcnQgbmVnYXRpdmUgdmFsdWUgdG8gdW5zaWduZWQgdHlwZQAAAAASTmVnYXRpdmVUb1Vuc2lnbmVkAAAAAAIFAAAAKk1hdGhFcnJvcjogRml4ZWQtcG9pbnQgYXJpdGhtZXRpYyBvdmVyZmxvdwAAAAAAEkZpeGVkUG9pbnRPdmVyZmxvdwAAAAACBg==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAQAAAAMU3RvcmFnZUVycm9yAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAyQAAAAAAAAATVmFsdWVOb3RJbml0aWFsaXplZAAAAAH1AAAAAAAAAAxWYWx1ZU1pc3NpbmcAAAH2AAAAAAAAABRWYWx1ZUNvbnZlcnNpb25FcnJvcgAAAfc=",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAADAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQAAAAAAAAARSW52YWxpZFBlcmNlbnRhZ2UAAAAAAAMiAAAAAAAAAApaZXJvQW1vdW50AAAAAAMk",
        "AAAAAQAAAAAAAAAAAAAABURlbGF5AAAAAAAAAQAAAAAAAAABMAAAAAAAAAY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<null>,
        get_price: this.txFromJSON<OraclePriceData>,
        set_seconds_before_stale: this.txFromJSON<null>,
        set_too_volatile_ratio: this.txFromJSON<null>,
        get_guard_rails: this.txFromJSON<GuardRails>
  }
}