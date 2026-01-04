use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};

pub fn create_contract(
    e: &Env,
    token_wasm_hash: BytesN<32>,
    asset: &Symbol,
    token_collateral: &Address,
    side: &Symbol,
) -> Address {
    let mut salt = Bytes::new(e);
    salt.append(&asset.to_xdr(e));
    salt.append(&token_collateral.to_xdr(e));
    salt.append(&side.to_xdr(e));
    let salt = e.crypto().sha256(&salt);
    e.deployer()
        .with_current_contract(salt)
        .deploy_v2(token_wasm_hash, ())
}

// Transfer
fn transfer(e: &Env, token: Address, from: &Address, to: &Address, amount: i128) {
    SorobanTokenClient::new(e, &token).transfer(from, to, &amount);
}

fn transfer_from(
    e: &Env,
    token: Address,
    spender: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
) {
    SorobanTokenClient::new(e, &token).transfer_from(spender, from, to, &amount);
}
