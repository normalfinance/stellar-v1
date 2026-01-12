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

pub(crate) trait TreasuryEvents {
    fn deposit(
        &self,
        user: Address,
        pair: Address,
        pairs_to_deposit: u128,
        usdc_deposit: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        user_shares_before: u128,
        user_shares_after: u128,
        ts: u64,
    );

    fn withdraw(
        &self,
        user: Address,
        pair: Address,
        shares_to_withdraw: u128,
        out_long: u128,
        out_short: u128,
        out_usdc: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        user_shares_before: u128,
        user_shares_after: u128,
        ts: u64,
    );

    fn trade(
        &self,
        user: Address,
        pair: Address,
        combined: bool,
        side: bool,
        direction: bool,
        in_amount: u128,
        out_amount: u128,
        price: u128,
        fee: u128,
        fee_amount: u128,
        ts: u64,
    );

    fn claim_protocol_fee(&self, pair: Address, token: Address, destination: Address, amount: u128);

    // Paused Ops
    fn kill_deposit(&self);

    fn unkill_deposit(&self);

    fn kill_withdraw(&self);

    fn unkill_withdraw(&self);

    fn kill_trade(&self);

    fn unkill_trade(&self);
}

impl TreasuryEvents for Events {
    fn deposit(
        &self,
        user: Address,
        pair: Address,
        pairs_to_deposit: u128,
        usdc_deposit: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        user_shares_before: u128,
        user_shares_after: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "deposit"), pair, user),
            (
                pairs_to_deposit,
                usdc_deposit,
                total_shares_before,
                total_shares_after,
                user_shares_before,
                user_shares_after,
                ts,
            ),
        );
    }

    fn withdraw(
        &self,
        user: Address,
        pair: Address,
        shares_to_withdraw: u128,
        out_long: u128,
        out_short: u128,
        out_usdc: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        user_shares_before: u128,
        user_shares_after: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "withdraw"), pair, user),
            (
                shares_to_withdraw,
                out_long,
                out_short,
                out_usdc,
                total_shares_before,
                total_shares_after,
                user_shares_before,
                user_shares_after,
                ts,
            ),
        );
    }

    fn trade(
        &self,
        user: Address,
        pair: Address,
        combined: bool,
        side: bool,
        direction: bool,
        in_amount: u128,
        out_amount: u128,
        price: u128,
        fee: u128,
        fee_amount: u128,
        ts: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "trade"), pair, user),
            (
                combined, side, direction, in_amount, out_amount, price, fee, fee_amount, ts,
            ),
        );
    }

    fn claim_protocol_fee(
        &self,
        pair: Address,
        token: Address,
        destination: Address,
        amount: u128,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "claim_protocol_fee"), pair, token),
            (destination, amount as i128),
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
