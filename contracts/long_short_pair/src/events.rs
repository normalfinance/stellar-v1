use soroban_sdk::{Address, Env, Symbol};

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

pub(crate) trait LongShortPairEvents {
    fn mint(&self, user: Address, pair: Address, collateral: u128, tokens_minted: u128, ts: u64);

    fn redemption(
        &self,
        user: Address,
        pair: Address,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    );

    // Paused Ops
    fn kill_mint(&self);

    fn unkill_mint(&self);

    fn kill_redeem(&self);

    fn unkill_redeem(&self);
}

impl LongShortPairEvents for Events {
    fn mint(&self, user: Address, pair: Address, collateral: u128, tokens_minted: u128, ts: u64) {
        self.env().events().publish(
            (Symbol::new(self.env(), "mint"), user, pair),
            (collateral, tokens_minted, ts),
        );
    }

    fn redemption(
        &self,
        user: Address,
        pair: Address,
        collateral: u128,
        tokens_redeemed: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redemption"), user, pair),
            (collateral, tokens_redeemed, ts),
        );
    }

    fn kill_mint(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_mint"),), ())
    }

    fn unkill_mint(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_mint"),), ())
    }

    fn kill_redeem(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_redeem"),), ())
    }

    fn unkill_redeem(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_redeem"),), ())
    }
}
