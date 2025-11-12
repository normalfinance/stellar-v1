#![cfg(test)]
extern crate std;

use crate::testutils::{Setup, TestConfig};
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::xdr::{Asset, WriteXdr};
use soroban_sdk::Bytes;

#[test]
fn test_create_token() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );

    let asset = Asset::CreditAlphanum4({ issuer });
    let asset_xdr = asset.to_xdr();

    let serialized_asset = Bytes::from(asset_xdr);

    let token_address = setup.factory.create_token(&serialized_asset);

    let token_client = SorobanTokenClient::new(&setup.env, &token_address);

    assert_eq!(token_client.name(), "NAME");
    assert_eq!(token_client.symbol(), "SYMBOL");
    assert_eq!(token_client.decimals(), 7);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_create_token_malformed_serialization() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );

    let asset = Asset::CreditAlphanum4({ issuer });
    let asset_xdr = asset.to_xdr();

    let malformed_serialized_asset = Bytes::from(asset_xdr);

    setup.factory.create_token(&malformed_serialized_asset);
}
