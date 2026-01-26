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
  202: {message:"InvalidBounds"}
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
   * Construct and simulate a percent_long_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
   * to per collateral_per_pair.
   * @param oracle_price price from the oracle for the target asset.
   * @param lower_bound lower price boundary from the Long Short Pair.
   * @param upper_bound upper price boundary from the Long Short Pair.
   * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
   */
  percent_long_collateral: ({oracle_price, lower_bound, upper_bound}: {oracle_price: u128, lower_bound: u128, upper_bound: u128}, options?: {
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
  }) => Promise<AssembledTransaction<u128>>

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
      new ContractSpec([ "AAAAAAAAAa5SZXR1cm5zIGEgbnVtYmVyIGJldHdlZW4gMCBhbmQgMSB0byBpbmRpY2F0ZSBob3cgbXVjaCBjb2xsYXRlcmFsIGVhY2ggbG9uZyBhbmQgc2hvcnQgdG9rZW4gaXMgZW50aXRsZWQKdG8gcGVyIGNvbGxhdGVyYWxfcGVyX3BhaXIuCkBwYXJhbSBvcmFjbGVfcHJpY2UgcHJpY2UgZnJvbSB0aGUgb3JhY2xlIGZvciB0aGUgdGFyZ2V0IGFzc2V0LgpAcGFyYW0gbG93ZXJfYm91bmQgbG93ZXIgcHJpY2UgYm91bmRhcnkgZnJvbSB0aGUgTG9uZyBTaG9ydCBQYWlyLgpAcGFyYW0gdXBwZXJfYm91bmQgdXBwZXIgcHJpY2UgYm91bmRhcnkgZnJvbSB0aGUgTG9uZyBTaG9ydCBQYWlyLgpAcmV0dXJuIGV4cGlyeVBlcmNlbnRMb25nIHRvIGluZGljYXRlIGhvdyBtdWNoIGNvbGxhdGVyYWwgc2hvdWxkIGJlIHNlbnQgYmV0d2VlbiBsb25nIGFuZCBzaG9ydCB0b2tlbnMuAAAAAAAXcGVyY2VudF9sb25nX2NvbGxhdGVyYWwAAAAAAwAAAAAAAAAMb3JhY2xlX3ByaWNlAAAACgAAAAAAAAALbG93ZXJfYm91bmQAAAAACgAAAAAAAAALdXBwZXJfYm91bmQAAAAACgAAAAEAAAAK",
        "AAAABAAAAAAAAAAAAAAAD0NhbGN1bGF0b3JFcnJvcgAAAAACAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAMkAAAAAAAAADUludmFsaWRCb3VuZHMAAAAAAADK",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAkAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAB1NYXRoRXJyb3I6IEdlbmVyaWMgbWF0aCBlcnJvcgAAAAAAAAlNYXRoRXJyb3IAAAAAAAH/AAAALU1hdGhFcnJvcjogQWRkaXRpb24gb3BlcmF0aW9uIGNhdXNlZCBvdmVyZmxvdwAAAAAAABBBZGRpdGlvbk92ZXJmbG93AAACAAAAADFNYXRoRXJyb3I6IFN1YnRyYWN0aW9uIG9wZXJhdGlvbiBjYXVzZWQgdW5kZXJmbG93AAAAAAAAFFN1YnRyYWN0aW9uVW5kZXJmbG93AAACAQAAADNNYXRoRXJyb3I6IE11bHRpcGxpY2F0aW9uIG9wZXJhdGlvbiBjYXVzZWQgb3ZlcmZsb3cAAAAAFk11bHRpcGxpY2F0aW9uT3ZlcmZsb3cAAAAAAgIAAAAbTWF0aEVycm9yOiBEaXZpc2lvbiBieSB6ZXJvAAAAAA5EaXZpc2lvbkJ5WmVybwAAAAACAwAAACNNYXRoRXJyb3I6IFR5cGUgY29udmVyc2lvbiBvdmVyZmxvdwAAAAASQ29udmVyc2lvbk92ZXJmbG93AAAAAAIEAAAAP01hdGhFcnJvcjogQXR0ZW1wdGVkIHRvIGNvbnZlcnQgbmVnYXRpdmUgdmFsdWUgdG8gdW5zaWduZWQgdHlwZQAAAAASTmVnYXRpdmVUb1Vuc2lnbmVkAAAAAAIFAAAAKk1hdGhFcnJvcjogRml4ZWQtcG9pbnQgYXJpdGhtZXRpYyBvdmVyZmxvdwAAAAAAEkZpeGVkUG9pbnRPdmVyZmxvdwAAAAACBg==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAQAAAAMU3RvcmFnZUVycm9yAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAyQAAAAAAAAATVmFsdWVOb3RJbml0aWFsaXplZAAAAAH1AAAAAAAAAAxWYWx1ZU1pc3NpbmcAAAH2AAAAAAAAABRWYWx1ZUNvbnZlcnNpb25FcnJvcgAAAfc=",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAEAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQAAAAAAAAARSW52YWxpZFBlcmNlbnRhZ2UAAAAAAAMiAAAAAAAAAApaZXJvQW1vdW50AAAAAAMkAAAAAAAAABZJbnZhbGlkT3JhY2xlVGltZXN0YW1wAAAAAAMl",
        "AAAAAQAAAAAAAAAAAAAABURlbGF5AAAAAAAAAQAAAAAAAAABMAAAAAAAAAY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    percent_long_collateral: this.txFromJSON<u128>
  }
}