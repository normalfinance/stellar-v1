use access_control::access::AccessControl;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use soroban_sdk::{ Address, Env, IntoVal, Symbol, Vec };

pub fn checkpoint_user_funding(e: &Env, from: Address, to: Address, amount: i128) {
    let access_control = AccessControl::new(&e);
    let pair_address = access_control.get_role(&Role::Admin);

    // FIXME: how do we checkpoint if a user is depositing/withdrawing in a pool
    if from == pair_address || to == pair_address {
        // no need to checkpoint the pair itself
        return;
    }

    e.invoke_contract::<()>(
        &pair_address,
        &Symbol::new(&e, "checkpoint_funding"),
        Vec::from_array(&e, [
            e.current_contract_address().to_val(),
            from.clone().to_val(),
            to.clone().to_val(),
            (amount as u128).into_val(e),
        ])
    );
}
