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




export const LongShortPairError = {
  201: {message:"AlreadyInitialized"},
  202: {message:"WrongInputVecSize"},
  203: {message:"InvalidOracle"},
  204: {message:"InvalidInput"},
  205: {message:"FundingWasNotUpdated"},
  206: {message:"FailedToGetPoolReserves"},
  207: {message:"FailedToGetCalculatorPercent"},
  208: {message:"FailedToUpdateTokenScalingFactor"},
  209: {message:"FailedToGetOraclePrice"}
}


export interface FundingCheckpoint {
  account: string;
  long_balance: u128;
  long_index: i64;
  short_balance: u128;
  short_index: i64;
}


export interface CollateralInfo {
  collateral_per_pair: u128;
  collateral_percent_long: u64;
}


export interface FundingInfo {
  cumulative_funding_index_long: i64;
  cumulative_funding_index_short: i64;
  funding_period: u64;
  last24h_avg_funding_rate: i64;
  last_funding_rate: i64;
  last_funding_rate_ts: u64;
  sanitize_clamp_denominator: i64;
}

export type DataKey = {tag: "TokenLong", values: void} | {tag: "TokenShort", values: void} | {tag: "TokenCollateral", values: void} | {tag: "CollateralPerPair", values: void} | {tag: "CollateralPercentLong", values: void} | {tag: "Pool", values: void} | {tag: "Calculator", values: void} | {tag: "NormalOracle", values: void} | {tag: "GuardRails", values: void} | {tag: "OracleData", values: void} | {tag: "SanitizeClampDenominator", values: void} | {tag: "FundingCheckpoint", values: readonly [string]} | {tag: "CumulativeFundingIndexLong", values: void} | {tag: "CumulativeFundingIndexShort", values: void} | {tag: "LastFundingRate", values: void} | {tag: "Last24hAvgFundingRate", values: void} | {tag: "LastFundingRateTs", values: void} | {tag: "FundingPeriod", values: void} | {tag: "LastUpdateTs", values: void} | {tag: "IsKilledCreate", values: void} | {tag: "IsKilledRedeem", values: void} | {tag: "IsKilledUpdateFunding", values: void};

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
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, privileged_addrs, tokens, oracle, calculator, pool}: {admin: string, privileged_addrs: readonly [string, string, string, string, Array<string>, string], tokens: Array<string>, oracle: string, calculator: string, pool: string}, options?: {
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
   * Construct and simulate a create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Creates a pair of long and short tokens equal in number to tokensToCreate. Pulls the required collateral
   *      * amount into this contract, defined by the collateralPerPair value.
   *      * @dev The caller must approve this contract to transfer `tokensToCreate * collateralPerPair` amount of collateral.
   *      * @param tokensToCreate number of long and short synthetic tokens to create.
   *      * @return collateralUsed total collateral used to mint the synthetics.
   */
  create: ({user, tokens_to_create}: {user: string, tokens_to_create: u128}, options?: {
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

  /**
   * Construct and simulate a redeem transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * * @notice Redeems a pair of long and short tokens equal in number to tokensToRedeem. Returns the commensurate
   *      * amount of collateral to the caller for the pair of tokens, defined by the collateralPerPair value.
   *      * @dev This contract must have the `Burner` role for the `longToken` and `shortToken` in order to call `burnFrom`.
   *      * @dev The caller does not need to approve this contract to transfer any amount of `tokensToRedeem` since long
   *      * and short tokens are burned, rather than transferred, from the caller.
   *      * @dev This method can be called either pre or post expiration.
   *      * @param tokensToRedeem number of long and short synthetic tokens to redeem.
   *      * @return collateralReturned total collateral returned in exchange for the pair of synthetics.
   */
  redeem: ({user, tokens_to_redeem}: {user: string, tokens_to_redeem: u128}, options?: {
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

  /**
   * Construct and simulate a checkpoint_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  checkpoint_funding: ({token_contract, from, to, transfer_amount}: {token_contract: string, from: string, to: string, transfer_amount: u128}, options?: {
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
   * Construct and simulate a update_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_oracle_price: (options?: {
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
   * Construct and simulate a get_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_tokens: (options?: {
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
   * Construct and simulate a get_position_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_position_tokens: ({user}: {user: string}, options?: {
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
  }) => Promise<AssembledTransaction<Array<u128>>>

  /**
   * Construct and simulate a get_collateral_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_collateral_info: (options?: {
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
  }) => Promise<AssembledTransaction<CollateralInfo>>

  /**
   * Construct and simulate a get_funding_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_funding_info: (options?: {
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
  }) => Promise<AssembledTransaction<FundingInfo>>

  /**
   * Construct and simulate a update_funding_period transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_funding_period: ({admin, funding_period}: {admin: string, funding_period: u64}, options?: {
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
   * Construct and simulate a update_funding_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_funding_rate: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a set_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_privileged_addrs: ({admin, rewards_admin, operations_admin, pause_admin, emergency_pause_admins, system_fee_admin}: {admin: string, rewards_admin: string, operations_admin: string, pause_admin: string, emergency_pause_admins: Array<string>, system_fee_admin: string}, options?: {
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
   * Construct and simulate a get_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_privileged_addrs: (options?: {
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
  }) => Promise<AssembledTransaction<Map<string, Array<string>>>>

  /**
   * Construct and simulate a set_oracle transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_oracle: ({admin, oracle}: {admin: string, oracle: string}, options?: {
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
   * Construct and simulate a kill_redeem transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  kill_redeem: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a kill_update_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  kill_update_funding: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a unkill_redeem transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unkill_redeem: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a unkill_update_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unkill_update_funding: ({admin}: {admin: string}, options?: {
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
   * Construct and simulate a get_is_killed_redeem transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_is_killed_redeem: (options?: {
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
   * Construct and simulate a get_is_killed_update_funding transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_is_killed_update_funding: (options?: {
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
      new ContractSpec([ "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAABgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABBwcml2aWxlZ2VkX2FkZHJzAAAD7QAAAAYAAAATAAAAEwAAABMAAAATAAAD6gAAABMAAAATAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAACmNhbGN1bGF0b3IAAAAAABMAAAAAAAAABHBvb2wAAAATAAAAAA==",
        "AAAAAAAAAdMqIEBub3RpY2UgQ3JlYXRlcyBhIHBhaXIgb2YgbG9uZyBhbmQgc2hvcnQgdG9rZW5zIGVxdWFsIGluIG51bWJlciB0byB0b2tlbnNUb0NyZWF0ZS4gUHVsbHMgdGhlIHJlcXVpcmVkIGNvbGxhdGVyYWwKICAgICAqIGFtb3VudCBpbnRvIHRoaXMgY29udHJhY3QsIGRlZmluZWQgYnkgdGhlIGNvbGxhdGVyYWxQZXJQYWlyIHZhbHVlLgogICAgICogQGRldiBUaGUgY2FsbGVyIG11c3QgYXBwcm92ZSB0aGlzIGNvbnRyYWN0IHRvIHRyYW5zZmVyIGB0b2tlbnNUb0NyZWF0ZSAqIGNvbGxhdGVyYWxQZXJQYWlyYCBhbW91bnQgb2YgY29sbGF0ZXJhbC4KICAgICAqIEBwYXJhbSB0b2tlbnNUb0NyZWF0ZSBudW1iZXIgb2YgbG9uZyBhbmQgc2hvcnQgc3ludGhldGljIHRva2VucyB0byBjcmVhdGUuCiAgICAgKiBAcmV0dXJuIGNvbGxhdGVyYWxVc2VkIHRvdGFsIGNvbGxhdGVyYWwgdXNlZCB0byBtaW50IHRoZSBzeW50aGV0aWNzLgAAAAAGY3JlYXRlAAAAAAACAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAQdG9rZW5zX3RvX2NyZWF0ZQAAAAoAAAABAAAACg==",
        "AAAAAAAAAwwqIEBub3RpY2UgUmVkZWVtcyBhIHBhaXIgb2YgbG9uZyBhbmQgc2hvcnQgdG9rZW5zIGVxdWFsIGluIG51bWJlciB0byB0b2tlbnNUb1JlZGVlbS4gUmV0dXJucyB0aGUgY29tbWVuc3VyYXRlCiAgICAgKiBhbW91bnQgb2YgY29sbGF0ZXJhbCB0byB0aGUgY2FsbGVyIGZvciB0aGUgcGFpciBvZiB0b2tlbnMsIGRlZmluZWQgYnkgdGhlIGNvbGxhdGVyYWxQZXJQYWlyIHZhbHVlLgogICAgICogQGRldiBUaGlzIGNvbnRyYWN0IG11c3QgaGF2ZSB0aGUgYEJ1cm5lcmAgcm9sZSBmb3IgdGhlIGBsb25nVG9rZW5gIGFuZCBgc2hvcnRUb2tlbmAgaW4gb3JkZXIgdG8gY2FsbCBgYnVybkZyb21gLgogICAgICogQGRldiBUaGUgY2FsbGVyIGRvZXMgbm90IG5lZWQgdG8gYXBwcm92ZSB0aGlzIGNvbnRyYWN0IHRvIHRyYW5zZmVyIGFueSBhbW91bnQgb2YgYHRva2Vuc1RvUmVkZWVtYCBzaW5jZSBsb25nCiAgICAgKiBhbmQgc2hvcnQgdG9rZW5zIGFyZSBidXJuZWQsIHJhdGhlciB0aGFuIHRyYW5zZmVycmVkLCBmcm9tIHRoZSBjYWxsZXIuCiAgICAgKiBAZGV2IFRoaXMgbWV0aG9kIGNhbiBiZSBjYWxsZWQgZWl0aGVyIHByZSBvciBwb3N0IGV4cGlyYXRpb24uCiAgICAgKiBAcGFyYW0gdG9rZW5zVG9SZWRlZW0gbnVtYmVyIG9mIGxvbmcgYW5kIHNob3J0IHN5bnRoZXRpYyB0b2tlbnMgdG8gcmVkZWVtLgogICAgICogQHJldHVybiBjb2xsYXRlcmFsUmV0dXJuZWQgdG90YWwgY29sbGF0ZXJhbCByZXR1cm5lZCBpbiBleGNoYW5nZSBmb3IgdGhlIHBhaXIgb2Ygc3ludGhldGljcy4AAAAGcmVkZWVtAAAAAAACAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAQdG9rZW5zX3RvX3JlZGVlbQAAAAoAAAABAAAACg==",
        "AAAAAAAAAAAAAAASY2hlY2twb2ludF9mdW5kaW5nAAAAAAAEAAAAAAAAAA50b2tlbl9jb250cmFjdAAAAAAAEwAAAAAAAAAEZnJvbQAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAA90cmFuc2Zlcl9hbW91bnQAAAAACgAAAAA=",
        "AAAAAAAAAAAAAAATdXBkYXRlX29yYWNsZV9wcmljZQAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAKZ2V0X3Rva2VucwAAAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAATZ2V0X3Bvc2l0aW9uX3Rva2VucwAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPqAAAACg==",
        "AAAAAAAAAAAAAAATZ2V0X2NvbGxhdGVyYWxfaW5mbwAAAAAAAAAAAQAAB9AAAAAOQ29sbGF0ZXJhbEluZm8AAA==",
        "AAAAAAAAAAAAAAAQZ2V0X2Z1bmRpbmdfaW5mbwAAAAAAAAABAAAH0AAAAAtGdW5kaW5nSW5mbwA=",
        "AAAAAAAAAAAAAAAVdXBkYXRlX2Z1bmRpbmdfcGVyaW9kAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAA5mdW5kaW5nX3BlcmlvZAAAAAAABgAAAAA=",
        "AAAAAAAAAAAAAAATdXBkYXRlX2Z1bmRpbmdfcmF0ZQAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAUc2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAGAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADXJld2FyZHNfYWRtaW4AAAAAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAAWZW1lcmdlbmN5X3BhdXNlX2FkbWlucwAAAAAD6gAAABMAAAAAAAAAEHN5c3RlbV9mZWVfYWRtaW4AAAATAAAAAA==",
        "AAAAAAAAAAAAAAAUZ2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAAAAAAAQAAA+wAAAARAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAKc2V0X29yYWNsZQAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAALa2lsbF9jcmVhdGUAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAALa2lsbF9yZWRlZW0AAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAATa2lsbF91cGRhdGVfZnVuZGluZwAAAAABAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAANdW5raWxsX2NyZWF0ZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANdW5raWxsX3JlZGVlbQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAVdW5raWxsX3VwZGF0ZV9mdW5kaW5nAAAAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAUZ2V0X2lzX2tpbGxlZF9jcmVhdGUAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAUZ2V0X2lzX2tpbGxlZF9yZWRlZW0AAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAcZ2V0X2lzX2tpbGxlZF91cGRhdGVfZnVuZGluZwAAAAAAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAZY29tbWl0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAAAAAALbmV3X2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYYXBwbHlfdHJhbnNmZXJfb3duZXJzaGlwAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAA==",
        "AAAAAAAAAAAAAAAZcmV2ZXJ0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X2Z1dHVyZV9hZGRyZXNzAAAAAAABAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAHdmVyc2lvbgAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAANY29udHJhY3RfbmFtZQAAAAAAAAAAAAABAAAAEQ==",
        "AAAAAAAAAAAAAAAOY29tbWl0X3VwZ3JhZGUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAANYXBwbHlfdXBncmFkZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAOcmV2ZXJ0X3VwZ3JhZGUAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASc2V0X2VtZXJnZW5jeV9tb2RlAAAAAAACAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAFdmFsdWUAAAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X2VtZXJnZW5jeV9tb2RlAAAAAAAAAAAAAQAAAAE=",
        "AAAABAAAAAAAAAAAAAAAEkxvbmdTaG9ydFBhaXJFcnJvcgAAAAAACQAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAADJAAAAAAAAABFXcm9uZ0lucHV0VmVjU2l6ZQAAAAAAAMoAAAAAAAAADUludmFsaWRPcmFjbGUAAAAAAADLAAAAAAAAAAxJbnZhbGlkSW5wdXQAAADMAAAAAAAAABRGdW5kaW5nV2FzTm90VXBkYXRlZAAAAM0AAAAAAAAAF0ZhaWxlZFRvR2V0UG9vbFJlc2VydmVzAAAAAM4AAAAAAAAAHEZhaWxlZFRvR2V0Q2FsY3VsYXRvclBlcmNlbnQAAADPAAAAAAAAACBGYWlsZWRUb1VwZGF0ZVRva2VuU2NhbGluZ0ZhY3RvcgAAANAAAAAAAAAAFkZhaWxlZFRvR2V0T3JhY2xlUHJpY2UAAAAAANE=",
        "AAAAAQAAAAAAAAAAAAAAEUZ1bmRpbmdDaGVja3BvaW50AAAAAAAABQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAAAAAAxsb25nX2JhbGFuY2UAAAAKAAAAAAAAAApsb25nX2luZGV4AAAAAAAHAAAAAAAAAA1zaG9ydF9iYWxhbmNlAAAAAAAACgAAAAAAAAALc2hvcnRfaW5kZXgAAAAABw==",
        "AAAAAQAAAAAAAAAAAAAADkNvbGxhdGVyYWxJbmZvAAAAAAACAAAAAAAAABNjb2xsYXRlcmFsX3Blcl9wYWlyAAAAAAoAAAAAAAAAF2NvbGxhdGVyYWxfcGVyY2VudF9sb25nAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAAC0Z1bmRpbmdJbmZvAAAAAAcAAAAAAAAAHWN1bXVsYXRpdmVfZnVuZGluZ19pbmRleF9sb25nAAAAAAAABwAAAAAAAAAeY3VtdWxhdGl2ZV9mdW5kaW5nX2luZGV4X3Nob3J0AAAAAAAHAAAAAAAAAA5mdW5kaW5nX3BlcmlvZAAAAAAABgAAAAAAAAAYbGFzdDI0aF9hdmdfZnVuZGluZ19yYXRlAAAABwAAAAAAAAARbGFzdF9mdW5kaW5nX3JhdGUAAAAAAAAHAAAAAAAAABRsYXN0X2Z1bmRpbmdfcmF0ZV90cwAAAAYAAAAAAAAAGnNhbml0aXplX2NsYW1wX2Rlbm9taW5hdG9yAAAAAAAH",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAFgAAAAAAAAAAAAAACVRva2VuTG9uZwAAAAAAAAAAAAAAAAAAClRva2VuU2hvcnQAAAAAAAAAAAAAAAAAD1Rva2VuQ29sbGF0ZXJhbAAAAAAAAAAAAAAAABFDb2xsYXRlcmFsUGVyUGFpcgAAAAAAAAAAAAAAAAAAFUNvbGxhdGVyYWxQZXJjZW50TG9uZwAAAAAAAAAAAAAAAAAABFBvb2wAAAAAAAAAAAAAAApDYWxjdWxhdG9yAAAAAAAAAAAAAAAAAAxOb3JtYWxPcmFjbGUAAAAAAAAAAAAAAApHdWFyZFJhaWxzAAAAAAAAAAAAAAAAAApPcmFjbGVEYXRhAAAAAAAAAAAAAAAAABhTYW5pdGl6ZUNsYW1wRGVub21pbmF0b3IAAAABAAAAAAAAABFGdW5kaW5nQ2hlY2twb2ludAAAAAAAAAEAAAATAAAAAAAAAAAAAAAaQ3VtdWxhdGl2ZUZ1bmRpbmdJbmRleExvbmcAAAAAAAAAAAAAAAAAG0N1bXVsYXRpdmVGdW5kaW5nSW5kZXhTaG9ydAAAAAAAAAAAAAAAAA9MYXN0RnVuZGluZ1JhdGUAAAAAAAAAAAAAAAAVTGFzdDI0aEF2Z0Z1bmRpbmdSYXRlAAAAAAAAAAAAAAAAAAARTGFzdEZ1bmRpbmdSYXRlVHMAAAAAAAAAAAAAAAAAAA1GdW5kaW5nUGVyaW9kAAAAAAAAAAAAAAAAAAAMTGFzdFVwZGF0ZVRzAAAAAAAAAAAAAAAOSXNLaWxsZWRDcmVhdGUAAAAAAAAAAAAAAAAADklzS2lsbGVkUmVkZWVtAAAAAAAAAAAAAAAAABVJc0tpbGxlZFVwZGF0ZUZ1bmRpbmcAAAA=",
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
    initialize: this.txFromJSON<null>,
        create: this.txFromJSON<u128>,
        redeem: this.txFromJSON<u128>,
        checkpoint_funding: this.txFromJSON<null>,
        update_oracle_price: this.txFromJSON<null>,
        get_tokens: this.txFromJSON<Array<string>>,
        get_position_tokens: this.txFromJSON<Array<u128>>,
        get_collateral_info: this.txFromJSON<CollateralInfo>,
        get_funding_info: this.txFromJSON<FundingInfo>,
        update_funding_period: this.txFromJSON<null>,
        update_funding_rate: this.txFromJSON<null>,
        set_privileged_addrs: this.txFromJSON<null>,
        get_privileged_addrs: this.txFromJSON<Map<string, Array<string>>>,
        set_oracle: this.txFromJSON<null>,
        kill_create: this.txFromJSON<null>,
        kill_redeem: this.txFromJSON<null>,
        kill_update_funding: this.txFromJSON<null>,
        unkill_create: this.txFromJSON<null>,
        unkill_redeem: this.txFromJSON<null>,
        unkill_update_funding: this.txFromJSON<null>,
        get_is_killed_create: this.txFromJSON<boolean>,
        get_is_killed_redeem: this.txFromJSON<boolean>,
        get_is_killed_update_funding: this.txFromJSON<boolean>,
        commit_transfer_ownership: this.txFromJSON<null>,
        apply_transfer_ownership: this.txFromJSON<null>,
        revert_transfer_ownership: this.txFromJSON<null>,
        get_future_address: this.txFromJSON<string>,
        version: this.txFromJSON<u32>,
        contract_name: this.txFromJSON<string>,
        commit_upgrade: this.txFromJSON<null>,
        apply_upgrade: this.txFromJSON<Buffer>,
        revert_upgrade: this.txFromJSON<null>,
        set_emergency_mode: this.txFromJSON<null>,
        get_emergency_mode: this.txFromJSON<boolean>
  }
}