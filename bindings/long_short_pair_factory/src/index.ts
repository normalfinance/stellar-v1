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





export interface FactoryConfig {
  lsp_contract_wasm: Buffer;
  token_factory: string;
}


export interface CreatorParams {
  admin: string;
  calculator: string;
  collateral_per_pair: u128;
  collateral_token: string;
  oracle: string;
  pair_name: string;
  pool: string;
  serialized_long_asset: Buffer;
  serialized_short_asset: Buffer;
}

export type DataKey = {tag: "TokenFactory", values: void} | {tag: "LongShortPairContractWASM", values: void} | {tag: "ContractSequence", values: readonly [string]} | {tag: "DeployedLSPs", values: readonly [string]} | {tag: "AllDeployedLSPs", values: void} | {tag: "IsKilledCreate", values: void};

export const AccessControlError = {
  101: {message:"RoleNotFound"},
  102: {message:"Unauthorized"},
  103: {message:"AdminAlreadySet"},
  104: {message:"BadRoleUsage"},
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
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
   * Construct and simulate a deploy_lsp_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Creates a longShortPair contract and associated long and short tokens.
   *      * @param params Constructor params used to initialize the LSP. Key-valued object with the following structure:
   *      *     - `pairName`: Name of the long short pair contract.
   *      *     - `collateralPerPair`: How many units of collateral are required to mint one pair of synthetic tokens.
   *      *     - `collateralToken`: ERC20 token used as collateral in the LSP.
   *      *     - `calculator`: Contract providing settlement payout logic.
   *      * @return lspAddress the deployed address of the new long short pair contract.
   *      * @notice Created LSP is not registered within the registry as the LSP uses the Optimistic Oracle for settlement.
   *      * @notice The LSP constructor does a number of validations on input params. These are not repeated here.
   */
  deploy_lsp_contract: ({params}: {params: CreatorParams}, options?: {
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
   * Construct and simulate a get_factory_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_factory_config: (options?: {
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
  }) => Promise<AssembledTransaction<FactoryConfig>>

  /**
   * Construct and simulate a get_token_factory transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_token_factory: (options?: {
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
   * Construct and simulate a get_lsp_contract_wasm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_lsp_contract_wasm: (options?: {
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
   * Construct and simulate a get_deployed_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_deployed_pairs: ({operator}: {operator: string}, options?: {
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
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_all_deployed_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_deployed_pairs: (options?: {
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
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_pair_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pair_count: ({operator}: {operator: string}, options?: {
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
   * Construct and simulate a get_total_pair_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_pair_count: (options?: {
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
   * Construct and simulate a set_lsp_contract_wasm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_lsp_contract_wasm: ({admin, lsp_contract_wasm}: {admin: string, lsp_contract_wasm: Buffer}, options?: {
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
   * Construct and simulate a kill_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  kill_create: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a unkill_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unkill_create: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a get_is_killed_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_is_killed_create: (options?: {
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
        {admin, emergency_admin, token_factory, lsp_contract_wasm}: {admin: string, emergency_admin: string, token_factory: string, lsp_contract_wasm: Buffer},
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
    return ContractClient.deploy({admin, emergency_admin, token_factory, lsp_contract_wasm}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAADUZhY3RvcnlDb25maWcAAAAAAAACAAAAAAAAABFsc3BfY29udHJhY3Rfd2FzbQAAAAAAA+4AAAAgAAAAAAAAAA10b2tlbl9mYWN0b3J5AAAAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAADUNyZWF0b3JQYXJhbXMAAAAAAAAJAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAACmNhbGN1bGF0b3IAAAAAABMAAAAAAAAAE2NvbGxhdGVyYWxfcGVyX3BhaXIAAAAACgAAAAAAAAAQY29sbGF0ZXJhbF90b2tlbgAAABMAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAJcGFpcl9uYW1lAAAAAAAAEQAAAAAAAAAEcG9vbAAAABMAAAAAAAAAFXNlcmlhbGl6ZWRfbG9uZ19hc3NldAAAAAAAAA4AAAAAAAAAFnNlcmlhbGl6ZWRfc2hvcnRfYXNzZXQAAAAAAA4=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAQAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAADXRva2VuX2ZhY3RvcnkAAAAAAAATAAAAAAAAABFsc3BfY29udHJhY3Rfd2FzbQAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAA0AqIEBub3RpY2UgQ3JlYXRlcyBhIGxvbmdTaG9ydFBhaXIgY29udHJhY3QgYW5kIGFzc29jaWF0ZWQgbG9uZyBhbmQgc2hvcnQgdG9rZW5zLgogICAgICogQHBhcmFtIHBhcmFtcyBDb25zdHJ1Y3RvciBwYXJhbXMgdXNlZCB0byBpbml0aWFsaXplIHRoZSBMU1AuIEtleS12YWx1ZWQgb2JqZWN0IHdpdGggdGhlIGZvbGxvd2luZyBzdHJ1Y3R1cmU6CiAgICAgKiAgICAgLSBgcGFpck5hbWVgOiBOYW1lIG9mIHRoZSBsb25nIHNob3J0IHBhaXIgY29udHJhY3QuCiAgICAgKiAgICAgLSBgY29sbGF0ZXJhbFBlclBhaXJgOiBIb3cgbWFueSB1bml0cyBvZiBjb2xsYXRlcmFsIGFyZSByZXF1aXJlZCB0byBtaW50IG9uZSBwYWlyIG9mIHN5bnRoZXRpYyB0b2tlbnMuCiAgICAgKiAgICAgLSBgY29sbGF0ZXJhbFRva2VuYDogRVJDMjAgdG9rZW4gdXNlZCBhcyBjb2xsYXRlcmFsIGluIHRoZSBMU1AuCiAgICAgKiAgICAgLSBgY2FsY3VsYXRvcmA6IENvbnRyYWN0IHByb3ZpZGluZyBzZXR0bGVtZW50IHBheW91dCBsb2dpYy4KICAgICAqIEByZXR1cm4gbHNwQWRkcmVzcyB0aGUgZGVwbG95ZWQgYWRkcmVzcyBvZiB0aGUgbmV3IGxvbmcgc2hvcnQgcGFpciBjb250cmFjdC4KICAgICAqIEBub3RpY2UgQ3JlYXRlZCBMU1AgaXMgbm90IHJlZ2lzdGVyZWQgd2l0aGluIHRoZSByZWdpc3RyeSBhcyB0aGUgTFNQIHVzZXMgdGhlIE9wdGltaXN0aWMgT3JhY2xlIGZvciBzZXR0bGVtZW50LgogICAgICogQG5vdGljZSBUaGUgTFNQIGNvbnN0cnVjdG9yIGRvZXMgYSBudW1iZXIgb2YgdmFsaWRhdGlvbnMgb24gaW5wdXQgcGFyYW1zLiBUaGVzZSBhcmUgbm90IHJlcGVhdGVkIGhlcmUuAAAAE2RlcGxveV9sc3BfY29udHJhY3QAAAAAAQAAAAAAAAAGcGFyYW1zAAAAAAfQAAAADUNyZWF0b3JQYXJhbXMAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAASZ2V0X2ZhY3RvcnlfY29uZmlnAAAAAAAAAAAAAQAAB9AAAAANRmFjdG9yeUNvbmZpZwAAAA==",
        "AAAAAAAAAAAAAAARZ2V0X3Rva2VuX2ZhY3RvcnkAAAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAVZ2V0X2xzcF9jb250cmFjdF93YXNtAAAAAAAAAAAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAASZ2V0X2RlcGxveWVkX3BhaXJzAAAAAAABAAAAAAAAAAhvcGVyYXRvcgAAABMAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAWZ2V0X2FsbF9kZXBsb3llZF9wYWlycwAAAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAOZ2V0X3BhaXJfY291bnQAAAAAAAEAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAEAAAAE",
        "AAAAAAAAAAAAAAAUZ2V0X3RvdGFsX3BhaXJfY291bnQAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAVc2V0X2xzcF9jb250cmFjdF93YXNtAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABFsc3BfY29udHJhY3Rfd2FzbQAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAALa2lsbF9jcmVhdGUAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAANdW5raWxsX2NyZWF0ZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAUZ2V0X2lzX2tpbGxlZF9jcmVhdGUAAAAAAAAAAQAAAAE=",
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
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABgAAAAAAAAAAAAAADFRva2VuRmFjdG9yeQAAAAAAAAAAAAAAGUxvbmdTaG9ydFBhaXJDb250cmFjdFdBU00AAAAAAAABAAAAAAAAABBDb250cmFjdFNlcXVlbmNlAAAAAQAAABMAAAABAAAAAAAAAAxEZXBsb3llZExTUHMAAAABAAAAEwAAAAAAAAAAAAAAD0FsbERlcGxveWVkTFNQcwAAAAAAAAAAAAAAAA5Jc0tpbGxlZENyZWF0ZQAA",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAAAAAAAAMUm9sZU5vdEZvdW5kAAAAZQAAAAAAAAAMVW5hdXRob3JpemVkAAAAZgAAAAAAAAAPQWRtaW5BbHJlYWR5U2V0AAAAAGcAAAAAAAAADEJhZFJvbGVVc2FnZQAAAGgAAAAAAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAAwAAAAAAAAATQW5vdGhlckFjdGlvbkFjdGl2ZQAAAAtaAAAAAAAAAA5Ob0FjdGlvbkFjdGl2ZQAAAAALWwAAAAAAAAARQWN0aW9uTm90UmVhZHlZZXQAAAAAAAtc",
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
    deploy_lsp_contract: this.txFromJSON<string>,
        get_factory_config: this.txFromJSON<FactoryConfig>,
        get_token_factory: this.txFromJSON<string>,
        get_lsp_contract_wasm: this.txFromJSON<Buffer>,
        get_deployed_pairs: this.txFromJSON<Array<string>>,
        get_all_deployed_pairs: this.txFromJSON<Array<string>>,
        get_pair_count: this.txFromJSON<u32>,
        get_total_pair_count: this.txFromJSON<u32>,
        set_lsp_contract_wasm: this.txFromJSON<null>,
        kill_create: this.txFromJSON<null>,
        unkill_create: this.txFromJSON<null>,
        get_is_killed_create: this.txFromJSON<boolean>,
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