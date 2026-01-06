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

export interface Client {
  /**
   * Construct and simulate a percent_long_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
   *      * to per collateralPerPair.
   *      * @param oracle_price price from the optimistic oracle for the LSP price identifier.
   *      * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
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
      new ContractSpec([ "AAAAAAAAAVoqIEBub3RpY2UgUmV0dXJucyBhIG51bWJlciBiZXR3ZWVuIDAgYW5kIDEgdG8gaW5kaWNhdGUgaG93IG11Y2ggY29sbGF0ZXJhbCBlYWNoIGxvbmcgYW5kIHNob3J0IHRva2VuIGlzIGVudGl0bGVkCiAgICAgKiB0byBwZXIgY29sbGF0ZXJhbFBlclBhaXIuCiAgICAgKiBAcGFyYW0gb3JhY2xlX3ByaWNlIHByaWNlIGZyb20gdGhlIG9wdGltaXN0aWMgb3JhY2xlIGZvciB0aGUgTFNQIHByaWNlIGlkZW50aWZpZXIuCiAgICAgKiBAcmV0dXJuIGV4cGlyeVBlcmNlbnRMb25nIHRvIGluZGljYXRlIGhvdyBtdWNoIGNvbGxhdGVyYWwgc2hvdWxkIGJlIHNlbnQgYmV0d2VlbiBsb25nIGFuZCBzaG9ydCB0b2tlbnMuAAAAAAAXcGVyY2VudF9sb25nX2NvbGxhdGVyYWwAAAAAAwAAAAAAAAAMb3JhY2xlX3ByaWNlAAAACgAAAAAAAAALbG93ZXJfYm91bmQAAAAACgAAAAAAAAALdXBwZXJfYm91bmQAAAAACgAAAAEAAAAG",
        "AAAABAAAAAAAAAAAAAAAD0NhbGN1bGF0b3JFcnJvcgAAAAAEAAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAMkAAAAAAAAADUludmFsaWRCb3VuZHMAAAAAAADKAAAAAAAAABBQYXJhbXNBbHJlYWR5U2V0AAAAywAAAAAAAAAZUGFyYW1zTm90U2V0Rm9yQ2FsbGluZ0xTUAAAAAAAAMw=" ]),
      options
    )
  }
  public readonly fromJSON = {
    percent_long_collateral: this.txFromJSON<u64>
  }
}