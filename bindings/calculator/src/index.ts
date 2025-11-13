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




export const CalculatorError = {
  201: {message:"AlreadyInitialized"},
  202: {message:"InvalidBounds"},
  203: {message:"ParamsAlreadySet"},
  204: {message:"ParamsNotSetForCallingLSP"}
}

export type DataKey = {tag: "LongShortPairParams", values: readonly [string]};


export interface LinearLongShortPairParameters {
  lower_bound: u128;
  upper_bound: u128;
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

export const OracleError = {
  /**
   * OracleError: OracleNonPositive
   */
  601: {message:"OracleNonPositive"},
  602: {message:"OracleTooVolatile"},
  603: {message:"OracleStaleForPool"}
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


export interface OraclePriceData {
  delay: Delay;
  price: u128;
}


export interface PriceDivergenceGuardRails {
  ratio_percent_divergence: u64;
}


export interface ValidityGuardRails {
  seconds_before_stale: u64;
  too_volatile_ratio: u64;
}


export interface OracleGuardRails {
  price_divergence: PriceDivergenceGuardRails;
  validity: ValidityGuardRails;
}

export type OracleValidity = {tag: "NonPositive", values: void} | {tag: "TooVolatile", values: void} | {tag: "StaleForPool", values: void} | {tag: "Frozen", values: void} | {tag: "Valid", values: void};


export interface HistoricalOracleData {
  last_price: u128;
  last_price_twap: u128;
  last_update_ts: u64;
}

export type Delay = readonly [u64];

export interface Client {
  /**
   * Construct and simulate a set_parameters transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_parameters: ({long_short_pair, lower_bound, upper_bound}: {long_short_pair: string, lower_bound: u128, upper_bound: u128}, options?: {
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
   * Construct and simulate a percent_long_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
   *      * to per collateralPerPair.
   *      * @param oracle_price price from the optimistic oracle for the LSP price identifier.
   *      * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
   */
  percent_long_collateral: ({caller, oracle_price}: {caller: string, oracle_price: u128}, options?: {
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
  }) => Promise<AssembledTransaction<u64>>

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
      new ContractSpec([ "AAAAAAAAAAAAAAAOc2V0X3BhcmFtZXRlcnMAAAAAAAMAAAAAAAAAD2xvbmdfc2hvcnRfcGFpcgAAAAATAAAAAAAAAAtsb3dlcl9ib3VuZAAAAAAKAAAAAAAAAAt1cHBlcl9ib3VuZAAAAAAKAAAAAA==",
        "AAAAAAAAAVoqIEBub3RpY2UgUmV0dXJucyBhIG51bWJlciBiZXR3ZWVuIDAgYW5kIDEgdG8gaW5kaWNhdGUgaG93IG11Y2ggY29sbGF0ZXJhbCBlYWNoIGxvbmcgYW5kIHNob3J0IHRva2VuIGlzIGVudGl0bGVkCiAgICAgKiB0byBwZXIgY29sbGF0ZXJhbFBlclBhaXIuCiAgICAgKiBAcGFyYW0gb3JhY2xlX3ByaWNlIHByaWNlIGZyb20gdGhlIG9wdGltaXN0aWMgb3JhY2xlIGZvciB0aGUgTFNQIHByaWNlIGlkZW50aWZpZXIuCiAgICAgKiBAcmV0dXJuIGV4cGlyeVBlcmNlbnRMb25nIHRvIGluZGljYXRlIGhvdyBtdWNoIGNvbGxhdGVyYWwgc2hvdWxkIGJlIHNlbnQgYmV0d2VlbiBsb25nIGFuZCBzaG9ydCB0b2tlbnMuAAAAAAAXcGVyY2VudF9sb25nX2NvbGxhdGVyYWwAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAxvcmFjbGVfcHJpY2UAAAAKAAAAAQAAAAY=",
        "AAAABAAAAAAAAAAAAAAAD0NhbGN1bGF0b3JFcnJvcgAAAAAEAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAMkAAAAAAAAADUludmFsaWRCb3VuZHMAAAAAAADKAAAAAAAAABBQYXJhbXNBbHJlYWR5U2V0AAAAywAAAAAAAAAZUGFyYW1zTm90U2V0Rm9yQ2FsbGluZ0xTUAAAAAAAAMw=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAQAAAAEAAAAAAAAAE0xvbmdTaG9ydFBhaXJQYXJhbXMAAAAAAQAAABM=",
        "AAAAAQAAAAAAAAAAAAAAHUxpbmVhckxvbmdTaG9ydFBhaXJQYXJhbWV0ZXJzAAAAAAAAAgAAAAAAAAALbG93ZXJfYm91bmQAAAAACgAAAAAAAAALdXBwZXJfYm91bmQAAAAACg==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAkAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAB1NYXRoRXJyb3I6IEdlbmVyaWMgbWF0aCBlcnJvcgAAAAAAAAlNYXRoRXJyb3IAAAAAAAH/AAAALU1hdGhFcnJvcjogQWRkaXRpb24gb3BlcmF0aW9uIGNhdXNlZCBvdmVyZmxvdwAAAAAAABBBZGRpdGlvbk92ZXJmbG93AAACAAAAADFNYXRoRXJyb3I6IFN1YnRyYWN0aW9uIG9wZXJhdGlvbiBjYXVzZWQgdW5kZXJmbG93AAAAAAAAFFN1YnRyYWN0aW9uVW5kZXJmbG93AAACAQAAADNNYXRoRXJyb3I6IE11bHRpcGxpY2F0aW9uIG9wZXJhdGlvbiBjYXVzZWQgb3ZlcmZsb3cAAAAAFk11bHRpcGxpY2F0aW9uT3ZlcmZsb3cAAAAAAgIAAAAbTWF0aEVycm9yOiBEaXZpc2lvbiBieSB6ZXJvAAAAAA5EaXZpc2lvbkJ5WmVybwAAAAACAwAAACNNYXRoRXJyb3I6IFR5cGUgY29udmVyc2lvbiBvdmVyZmxvdwAAAAASQ29udmVyc2lvbk92ZXJmbG93AAAAAAIEAAAAP01hdGhFcnJvcjogQXR0ZW1wdGVkIHRvIGNvbnZlcnQgbmVnYXRpdmUgdmFsdWUgdG8gdW5zaWduZWQgdHlwZQAAAAASTmVnYXRpdmVUb1Vuc2lnbmVkAAAAAAIFAAAAKk1hdGhFcnJvcjogRml4ZWQtcG9pbnQgYXJpdGhtZXRpYyBvdmVyZmxvdwAAAAAAEkZpeGVkUG9pbnRPdmVyZmxvdwAAAAACBg==",
        "AAAABAAAAAAAAAAAAAAAC09yYWNsZUVycm9yAAAAAAMAAAAeT3JhY2xlRXJyb3I6IE9yYWNsZU5vblBvc2l0aXZlAAAAAAART3JhY2xlTm9uUG9zaXRpdmUAAAAAAAJZAAAAAAAAABFPcmFjbGVUb29Wb2xhdGlsZQAAAAAAAloAAAAAAAAAEk9yYWNsZVN0YWxlRm9yUG9vbAAAAAACWw==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAQAAAAMU3RvcmFnZUVycm9yAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAyQAAAAAAAAATVmFsdWVOb3RJbml0aWFsaXplZAAAAAH1AAAAAAAAAAxWYWx1ZU1pc3NpbmcAAAH2AAAAAAAAABRWYWx1ZUNvbnZlcnNpb25FcnJvcgAAAfc=",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAADAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQAAAAAAAAARSW52YWxpZFBlcmNlbnRhZ2UAAAAAAAMiAAAAAAAAAApaZXJvQW1vdW50AAAAAAMk",
        "AAAAAQAAAAAAAAAAAAAAD09yYWNsZVByaWNlRGF0YQAAAAACAAAAAAAAAAVkZWxheQAAAAAAB9AAAAAFRGVsYXkAAAAAAAAAAAAABXByaWNlAAAAAAAACg==",
        "AAAAAQAAAAAAAAAAAAAAGVByaWNlRGl2ZXJnZW5jZUd1YXJkUmFpbHMAAAAAAAABAAAAAAAAABhyYXRpb19wZXJjZW50X2RpdmVyZ2VuY2UAAAAG",
        "AAAAAQAAAAAAAAAAAAAAElZhbGlkaXR5R3VhcmRSYWlscwAAAAAAAgAAAAAAAAAUc2Vjb25kc19iZWZvcmVfc3RhbGUAAAAGAAAAAAAAABJ0b29fdm9sYXRpbGVfcmF0aW8AAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAAEE9yYWNsZUd1YXJkUmFpbHMAAAACAAAAAAAAABBwcmljZV9kaXZlcmdlbmNlAAAH0AAAABlQcmljZURpdmVyZ2VuY2VHdWFyZFJhaWxzAAAAAAAAAAAAAAh2YWxpZGl0eQAAB9AAAAASVmFsaWRpdHlHdWFyZFJhaWxzAAA=",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQb29sAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAAAwAAAAAAAAAKbGFzdF9wcmljZQAAAAAACgAAAAAAAAAPbGFzdF9wcmljZV90d2FwAAAAAAoAAAAAAAAADmxhc3RfdXBkYXRlX3RzAAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAABURlbGF5AAAAAAAAAQAAAAAAAAABMAAAAAAAAAY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    set_parameters: this.txFromJSON<null>,
        percent_long_collateral: this.txFromJSON<u64>
  }
}