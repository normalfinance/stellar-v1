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





export interface Client {
  /**
   * Construct and simulate a create_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Create a new token and return it to the caller.
   *      * @dev The caller will become the only minter and burner and the new owner capable of assigning the roles.
   *      * @param serialized_asset used to describe the new token.
   *      * @return sac_address an instance of the newly created token interface.
   */
  create_token: ({serialized_asset}: {serialized_asset: Buffer}, options?: {
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
      new ContractSpec([ "AAAAAAAAATUqIEBub3RpY2UgQ3JlYXRlIGEgbmV3IHRva2VuIGFuZCByZXR1cm4gaXQgdG8gdGhlIGNhbGxlci4KICAgICAqIEBkZXYgVGhlIGNhbGxlciB3aWxsIGJlY29tZSB0aGUgb25seSBtaW50ZXIgYW5kIGJ1cm5lciBhbmQgdGhlIG5ldyBvd25lciBjYXBhYmxlIG9mIGFzc2lnbmluZyB0aGUgcm9sZXMuCiAgICAgKiBAcGFyYW0gc2VyaWFsaXplZF9hc3NldCB1c2VkIHRvIGRlc2NyaWJlIHRoZSBuZXcgdG9rZW4uCiAgICAgKiBAcmV0dXJuIHNhY19hZGRyZXNzIGFuIGluc3RhbmNlIG9mIHRoZSBuZXdseSBjcmVhdGVkIHRva2VuIGludGVyZmFjZS4AAAAAAAAMY3JlYXRlX3Rva2VuAAAAAQAAAAAAAAAQc2VyaWFsaXplZF9hc3NldAAAAA4AAAABAAAAEw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    create_token: this.txFromJSON<string>
  }
}