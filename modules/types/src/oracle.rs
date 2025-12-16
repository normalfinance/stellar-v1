use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone)]
pub enum OracleSource {
    Reflector,
}
