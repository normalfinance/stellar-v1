#![cfg(test)]
extern crate std;

use crate::funding::FundingCheckpoint;
use crate::testutils::Setup;

#[test]
fn test_transferring_pair_tokens_updates_both_users_funding_checkpoints() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let user2 = setup.users[2].clone();
    let tokens_to_mint = 1_0000000;

    setup.token_usdc_admin_client.mint(&user1, &100_0000000);
    assert_eq!(setup.token_usdc.balance(&user1) as u128, 100_0000000);

    // Mint pair tokens
    setup.pair.mint(&user1, &tokens_to_mint);

    // User should have 1 of each token
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // Transfer LONG tokens
    setup
        .token_long
        .transfer(&user1, &user2, &(tokens_to_mint as i128));
    assert_eq!(setup.token_long.balance(&user1) as u128, 0);
    assert_eq!(setup.token_long.balance(&user2) as u128, tokens_to_mint);

    // Funding checkpoint updated for sending user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1.clone(),
            long_index: 0,
            short_index: 0,
            long_balance: 0, // because they transferred all of them
            short_balance: tokens_to_mint,
        }
    );

    // And receiving user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user2),
        FundingCheckpoint {
            account: user2.clone(),
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_mint,
            short_balance: 0,
        }
    );

    // Transfer SHORT tokens
    let short_tokens_to_transfer = tokens_to_mint / 2;
    setup
        .token_short
        .transfer(&user1, &user2, &(short_tokens_to_transfer as i128));
    assert_eq!(
        setup.token_short.balance(&user1) as u128,
        tokens_to_mint - short_tokens_to_transfer
    );
    assert_eq!(
        setup.token_short.balance(&user2) as u128,
        short_tokens_to_transfer
    );

    // Funding checkpoint updated for sending user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1.clone(),
            long_index: 0,
            short_index: 0,
            long_balance: 0,
            short_balance: tokens_to_mint - short_tokens_to_transfer,
        }
    );

    // And receiving user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user2),
        FundingCheckpoint {
            account: user2.clone(),
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_mint,
            short_balance: short_tokens_to_transfer,
        }
    );
}

// TODO: long and short index
