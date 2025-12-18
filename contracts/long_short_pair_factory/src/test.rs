#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::{
    contract::CreatorParams,
    testutils::{Setup, TestConfig},
};
use soroban_sdk::{
    testutils::Address as _,
    xdr::{AccountId, AlphaNum12, Asset, AssetCode12, Limits, PublicKey, WriteXdr},
    Bytes,
};
use soroban_sdk::{Address, Symbol};

#[test]
fn test_deploy_contract() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    let pair_calculator = Address::generate(&setup.env);
    let oracle = Address::generate(&setup.env);
    let collateral_token = Address::generate(&setup.env);
    let pool = Address::generate(&setup.env);

    let long_code: [u8; 12] = *b"nSOL-LONG___";
    let short_code: [u8; 12] = *b"nSOL-SHORT__";

    let issuer_account_str = "GAOATRJGNRPONPBRHUMW6OWBJ6DJNR3B7FBJAI3Y3LQIRIE4S7VCBWMB";
    let issuer_pubkey = issuer_account_str.parse::<PublicKey>().unwrap();

    let issuer = AccountId(issuer_pubkey);

    let long_asset = Asset::CreditAlphanum12(AlphaNum12 {
        asset_code: AssetCode12(long_code),
        issuer: issuer.clone(),
    });
    let short_asset = Asset::CreditAlphanum12(AlphaNum12 {
        asset_code: AssetCode12(short_code),
        issuer,
    });

    let long_xdr = long_asset.to_xdr(Limits::none()).unwrap();
    let short_xdr = short_asset.to_xdr(Limits::none()).unwrap();

    let serialized_long_asset = Bytes::from_slice(&setup.env, &long_xdr);
    let serialized_short_asset = Bytes::from_slice(&setup.env, &short_xdr);

    let params = CreatorParams {
        admin: setup.admin,
        pair_name: Symbol::new(&setup.env, "Normal Solana"),
        collateral_per_pair: 100_000000,
        serialized_long_asset,
        serialized_short_asset,
        collateral_token,
        oracle,
        pair_calculator,
        pool,
    };

    let pair_address = setup.factory.deploy_lsp_contract(&params);

    let pair_tokens =
        testutils::long_short_pair::Client::new(&setup.env, &pair_address).get_tokens();

    assert_eq!(pair_tokens.len(), 2);
}
