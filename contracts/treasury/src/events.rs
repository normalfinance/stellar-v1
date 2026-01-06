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

pub(crate) trait TreasuryEvents {
    fn deposit(&self, user: Address, pair: Address, amount: u128, ts: u64);

    fn withdraw(&self, user: Address, pair: Address, amount: u128, ts: u64);

    fn trade(&self, user: Address, pair: Address, side: u32, direction: u32, amount: u128, ts: u64);

    // Paused Ops
    fn kill_deposit(&self);

    fn unkill_deposit(&self);

    fn kill_withdraw(&self);

    fn unkill_withdraw(&self);

    fn kill_trade(&self);

    fn unkill_trade(&self);
}

impl TreasuryEvents for Events {
    fn deposit(&self, user: Address, pair: Address, amount: u128, ts: u64) {
        self.env().events().publish(
            (Symbol::new(self.env(), "deposit"), pair, user),
            (amount, ts),
        );
    }

    fn withdraw(&self, user: Address, pair: Address, amount: u128, ts: u64) {
        self.env().events().publish(
            (Symbol::new(self.env(), "withdraw"), pair, user),
            (amount, ts),
        );
    }

    fn trade(
        &self,
        user: Address,
        pair: Address,
        side: u32,
        direction: u32,
        amount: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "trade"), pair, user),
            (side, direction, amount, ts),
        );
    }

    fn kill_deposit(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_deposit"),), ())
    }

    fn unkill_deposit(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_deposit"),), ())
    }

    fn kill_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_withdraw"),), ())
    }

    fn unkill_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_withdraw"),), ())
    }

    fn kill_trade(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_trade"),), ())
    }

    fn unkill_trade(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_trade"),), ())
    }
}
