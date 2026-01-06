use soroban_sdk::{symbol_short, xdr::ToXdr, Bytes, BytesN, Env, Symbol};

pub(crate) fn get_pair_salt(e: &Env, asset: &Symbol) -> BytesN<32> {
    let mut salt = Bytes::new(e);

    salt.append(&symbol_short!("0x00").to_xdr(e));
    salt.append(&asset.to_xdr(e));

    e.crypto().sha256(&salt).to_bytes()
}
