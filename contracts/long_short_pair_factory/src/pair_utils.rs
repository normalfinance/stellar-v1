use soroban_sdk::{symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env};

pub(crate) fn get_pair_salt(e: &Env, manager: &Address, sequence: &u32) -> BytesN<32> {
    let mut salt = Bytes::new(e);

    salt.append(&symbol_short!("0x00").to_xdr(e));
    salt.append(&manager.to_xdr(e));
    salt.append(&sequence.to_xdr(e));

    e.crypto().sha256(&salt).to_bytes()
}
