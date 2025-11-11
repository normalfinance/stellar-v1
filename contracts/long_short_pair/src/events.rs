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
    fn tokens_created(&self, ts: u64, sponsor: Address, collateral_used: u128, tokens_minted: u128);

    fn tokens_redeemed(
        &self,
        ts: u64,
        sponsor: Address,
        collateral_returned: u128,
        tokens_redeemed: u128,
    );

    fn position_settled(
        &self,
        ts: u64,
        sponsor: Address,
        collateral_returned: u128,
        long_tokens: u128,
        short_tokens: u128,
    );

    // Paused Ops
    fn kill_create(&self);

    fn unkill_create(&self);

    fn kill_redeem(&self);

    fn unkill_redeem(&self);

    fn kill_settle(&self);

    fn unkill_settle(&self);
}

impl LongShortPairEvents for Events {
    fn tokens_created(
        &self,
        ts: u64,
        sponsor: Address,
        collateral_used: u128,
        tokens_minted: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "tokens_created"),
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
        collateral_returned: u128,
        tokens_redeemed: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "tokens_redeemed"),
                ts,
                sponsor,
                collateral_returned,
                tokens_redeemed,
            ),
            (),
        );
    }

    fn position_settled(
        &self,
        ts: u64,
        sponsor: Address,
        collateral_returned: u128,
        long_tokens: u128,
        short_tokens: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "position_settled"),
                ts,
                sponsor,
                collateral_returned,
                long_tokens,
                short_tokens,
            ),
            (),
        );
    }

    fn kill_create(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_create"),), ())
    }

    fn unkill_create(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_create"),), ())
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

    fn kill_settle(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_settle"),), ())
    }

    fn unkill_settle(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_settle"),), ())
    }
}
