use soroban_sdk::{Address, BytesN, Env, Symbol};
use types::pair::Side;

#[derive(Clone)]
pub(crate) struct Events(Env);

impl Events {
    #[inline(always)]
    pub(crate) fn env(&self) -> &Env {
        &self.0
    }

    #[inline(always)]
    pub(crate) fn new(env: &Env) -> Events {
        Events(env.clone())
    }
}

//  ___      ___       __        __    _____  ___
// |"  \    /"  |     /""\      |" \  (\"   \|"  \
//  \   \  //   |    /    \     ||  | |.\\   \    |
//  /\\  \/.    |   /' /\  \    |:  | |: \.   \\  |
// |: \.        |  //  __'  \   |.  | |.  \    \. |
// |.  \    /:  | /   /  \\  \  /\  |\|    \    \ |
// |___|\__/|___|(___/    \___)(__\_|_)\___|\____\)

pub(crate) trait FactoryEvents {
    // Enhanced pair deployment event with comprehensive metadata
    fn pair_deployed(&self, ts: u64, deployer: Address, pair_address: Address);

    // Factory configuration events
    fn factory_admin_updated(&self, ts: u64, old_admin: Address, new_admin: Address);

    fn factory_paused(&self, ts: u64, admin: Address);

    fn factory_unpaused(&self, ts: u64, admin: Address);

    fn mint(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        collateral: u128,
        tokens_minted: u128,
        ts: u64,
    );

    fn redeem(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    );

    fn redeem_one(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        side: Side,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    );
}

impl FactoryEvents for Events {
    // Enhanced event implementations
    fn pair_deployed(&self, ts: u64, deployer: Address, pair_address: Address) {
        // Split into multiple events due to Soroban topic limits
        // Primary deployment event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "pair_deployed"),
                ts,
                deployer.clone(),
                pair_address.clone(),
            ),
            (),
        );

        // Configuration event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "pair_config"),
                ts,
                pair_address.clone(),
            ),
            (),
        );
    }

    fn factory_admin_updated(&self, ts: u64, old_admin: Address, new_admin: Address) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "factory_admin_updated"),
                ts,
                old_admin,
                new_admin,
            ),
            (),
        );
    }

    fn factory_paused(&self, ts: u64, admin: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "factory_paused"), ts, admin), ());
    }

    fn factory_unpaused(&self, ts: u64, admin: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "factory_unpaused"), ts, admin), ());
    }

    fn mint(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        collateral: u128,
        tokens_minted: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "mint"), user, pair, asset),
            (collateral_token, collateral, tokens_minted, ts),
        );
    }

    fn redeem(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redeem"), user, pair, asset),
            (collateral_token, collateral, tokens_redeemed, ts),
        );
    }

    fn redeem_one(
        &self,
        user: Address,
        asset: Symbol,
        pair: Address,
        collateral_token: Address,
        side: Side,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redeem_one"), user, pair, asset),
            (collateral_token, side, collateral, tokens_redeemed, ts),
        );
    }
}

//   ______    ______    _____  ___    _______  __     _______
//  /" _  "\  /    " \  (\"   \|"  \  /"     "||" \   /" _   "|
// (: ( \___)// ____  \ |.\\   \    |(: ______)||  | (: ( \___)
//  \/ \    /  /    ) :)|: \.   \\  | \/    |  |:  |  \/ \
//  //  \ _(: (____/ // |.  \    \. | // ___)  |.  |  //  \ ___
// (:   _) \\        /  |    \    \ |(:  (     /\  |\(:   _(  _|
//  \_______)\"_____/    \___|\____\) \__/    (__\_|_)\_______)

pub(crate) trait FactoryConfigEvents {
    fn pair_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
        version: u32,
    );
}

impl FactoryConfigEvents for Events {
    fn pair_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
        version: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "pair_wasm_updated"),
                ts,
                admin,
                old_wasm,
                new_wasm,
                version,
            ),
            (),
        );
    }
}
