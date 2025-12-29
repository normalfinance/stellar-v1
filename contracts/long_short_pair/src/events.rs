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
    fn tokens_minted(&self, ts: u64, sponsor: Address, collateral_used: u128, tokens_minted: u128);

    fn tokens_redeemed(
        &self,
        ts: u64,
        sponsor: Address,
        collateral: u128,
        collateral_returned: u128,
        tokens_redeemed: u128,
    );

    fn funding_rate_record(&self, ts: u64, funding_rate: i64);

    fn migration(&self, ts: u64, new_lower_bound: u128, new_upper_bound: u128);

    // Paused Ops
    fn kill_mint(&self);

    fn unkill_mint(&self);

    fn kill_redeem(&self);

    fn unkill_redeem(&self);

    fn kill_update_funding(&self);

    fn unkill_update_funding(&self);
}

impl LongShortPairEvents for Events {
    fn tokens_minted(&self, ts: u64, sponsor: Address, collateral_used: u128, tokens_minted: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "tokens_minted"),
                ts,
                sponsor,
                collateral_used,
                tokens_minted,
            ),
            (),
        );
    }

    fn tokens_redeemed(
        &self,
        ts: u64,
        sponsor: Address,
        collateral: u128,
        collateral_returned: u128,
        tokens_redeemed: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "tokens_redeemed"),
                ts,
                sponsor,
                collateral,
                collateral_returned,
                tokens_redeemed,
            ),
            (),
        );
    }

    fn funding_rate_record(&self, ts: u64, funding_rate: i64) {
        self.env().events().publish(
            (Symbol::new(self.env(), "funding_rate_record"), ts),
            (funding_rate,),
        );
    }

    fn migration(&self, ts: u64, new_lower_bound: u128, new_upper_bound: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "migration"),
                ts,
                new_lower_bound,
                new_upper_bound,
            ),
            (),
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

    fn kill_update_funding(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_update_funding"),), ())
    }

    fn unkill_update_funding(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_update_funding"),), ())
    }
}
