#![cfg(test)]
extern crate std;

use crate::testutils::{Setup, TestConfig};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::xdr::{AccountId, AlphaNum4, Asset, AssetCode4, Limits, PublicKey, WriteXdr};
use soroban_sdk::{testutils::Address as _, Address};
use soroban_sdk::{Bytes, String};

#[test]
fn test_create_token() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    // The 4-byte asset code (exactly 4 bytes)
    let code: [u8; 4] = *b"TEST"; // or any 4-byte ASCII asset code

    // Make an AssetCode4 wrapper
    let asset_code = AssetCode4(code);

    // Build a PublicKey from a Stellar account ID (G… address)
    let issuer_account_str = "GAOATRJGNRPONPBRHUMW6OWBJ6DJNR3B7FBJAI3Y3LQIRIE4S7VCBWMB";
    let issuer_pubkey = issuer_account_str.parse::<PublicKey>().unwrap();

    // Wrap the public key in an AccountId
    let issuer = AccountId(issuer_pubkey);

    // Build the AlphaNum4 value
    let alpha4 = AlphaNum4 { asset_code, issuer };

    // Wrap it in the Asset enum
    let asset = Asset::CreditAlphanum4(alpha4);

    let asset_xdr = asset.to_xdr(Limits::none()).unwrap();

    let serialized_asset = Bytes::from_slice(&setup.env, &asset_xdr);

    let token_address = setup.factory.create_token(&serialized_asset);

    let token_client = SorobanTokenClient::new(&setup.env, &token_address);

    // let admin_client = SorobanTokenAdminClient::new(&setup.env, &token_address);
    // // admin_client.set_admin(&setup.admin);
    // // admin_client.mint(&setup.admin, &10_0000000);
    // // assert_eq!(token_client.balance(&setup.admin), 10_0000000);

    assert_eq!(token_client.symbol(), String::from_str(&setup.env, "TEST"));
    assert_eq!(token_client.decimals(), 7);
}
