#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env,
};

fn setup(
    env: &Env,
) -> (
    HeirSwitchContractClient<'_>,
    Address,
    TokenClient<'_>,
    StellarAssetClient<'_>,
) {
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token_id = sac.address();
    let token = TokenClient::new(env, &token_id);
    let token_admin = StellarAssetClient::new(env, &token_id);

    let contract_id = env.register(HeirSwitchContract, ());
    let client = HeirSwitchContractClient::new(env, &contract_id);
    (client, token_id, token, token_admin)
}

#[test]
fn test_heartbeat_resets_timer_and_heirs_claim_after_inactivity() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, token_id, token, token_admin) = setup(&env);
    let owner = Address::generate(&env);
    let heir_a = Address::generate(&env);
    let heir_b = Address::generate(&env);
    let guardian = Address::generate(&env);

    let heirs = vec![
        &env,
        HeirShare {
            heir: heir_a.clone(),
            share_bps: 6_000,
        },
        HeirShare {
            heir: heir_b.clone(),
            share_bps: 4_000,
        },
    ];
    let window = 30 * 86_400; // 30 days
    client.init(&owner, &token_id, &window, &heirs, &Some(guardian.clone()));

    token_admin.mint(&owner, &1_000_0000000);
    client.deposit(&1_000_0000000);

    // Advance 20 days: heirs cannot claim yet
    env.ledger().with_mut(|l| l.timestamp += 20 * 86_400);
    assert_eq!(
        client.try_claim_heir_share(&heir_a),
        Err(Ok(SwitchError::VaultStillActive))
    );

    // Owner checks in -> timer resets for another 30 days
    client.heartbeat();
    assert_eq!(client.time_remaining(), window);

    // Advance 25 days -> guardian extends grace window
    env.ledger().with_mut(|l| l.timestamp += 25 * 86_400);
    client.guardian_extend();
    assert_eq!(client.time_remaining(), window);

    // Advance full 30 days -> switch triggers, heirs claim exact 60% / 40% shares
    env.ledger().with_mut(|l| l.timestamp += window);
    assert_eq!(client.time_remaining(), 0);

    let payout_a = client.claim_heir_share(&heir_a);
    let payout_b = client.claim_heir_share(&heir_b);
    assert_eq!(payout_a, 600_0000000);
    assert_eq!(payout_b, 400_0000000);
    assert_eq!(token.balance(&heir_a), 600_0000000);
    assert_eq!(token.balance(&heir_b), 400_0000000);

    // Double claim rejected
    assert_eq!(
        client.try_claim_heir_share(&heir_a),
        Err(Ok(SwitchError::AlreadyClaimed))
    );
}

#[test]
fn test_invalid_heir_splits_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, token_id, _token, _admin) = setup(&env);
    let owner = Address::generate(&env);
    let heir = Address::generate(&env);

    let bad_heirs = vec![
        &env,
        HeirShare {
            heir,
            share_bps: 9_000, // Does not sum to 10,000
        },
    ];
    let res = client.try_init(&owner, &token_id, &86_400, &bad_heirs, &None);
    assert_eq!(res, Err(Ok(SwitchError::InvalidHeirSplits)));
}

#[test]
fn test_update_heirs_preview_payout_and_owner_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, token_id, token, token_admin) = setup(&env);
    let owner = Address::generate(&env);
    let heir_a = Address::generate(&env);
    let heir_b = Address::generate(&env);
    let stranger = Address::generate(&env);

    let initial_heirs = vec![
        &env,
        HeirShare {
            heir: heir_a.clone(),
            share_bps: 10_000,
        },
    ];
    client.init(&owner, &token_id, &86_400, &initial_heirs, &None);
    token_admin.mint(&owner, &1_000_0000000);
    client.deposit(&1_000_0000000);

    // Owner withdraws 200 USDC while alive
    let remaining = client.owner_withdraw(&200_0000000);
    assert_eq!(remaining, 800_0000000);
    assert_eq!(token.balance(&owner), 200_0000000);

    // Update splits to 75% / 25%
    let updated = vec![
        &env,
        HeirShare {
            heir: heir_a.clone(),
            share_bps: 7_500,
        },
        HeirShare {
            heir: heir_b.clone(),
            share_bps: 2_500,
        },
    ];
    client.update_heirs(&updated);
    assert_eq!(client.preview_heir_payout(&heir_a), 600_0000000);
    assert_eq!(client.preview_heir_payout(&heir_b), 200_0000000);
    assert_eq!(
        client.try_preview_heir_payout(&stranger),
        Err(Ok(SwitchError::HeirNotFound))
    );
    assert!(!client.has_claimed(&heir_a));
}

#[test]
fn test_update_inactivity_window_and_claimable_state() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let (client, token_id, _token, token_admin) = setup(&env);
    let owner = Address::generate(&env);
    let heir = Address::generate(&env);

    let heirs = vec![&env, HeirShare { heir: heir.clone(), share_bps: 10_000 }];
    client.init(&owner, &token_id, &86_400, &heirs, &None);
    token_admin.mint(&owner, &500_0000000);
    client.deposit(&500_0000000);

    assert!(!client.is_claimable());
    assert_eq!(client.unclaimed_balance(), 500_0000000);

    // Extend inactivity window to 7 days
    let seven_days = 7 * 86_400;
    assert_eq!(client.update_inactivity_window(&seven_days), seven_days);
    assert_eq!(client.time_remaining(), seven_days);

    // Advance 7 days -> claimable
    env.ledger().with_mut(|l| l.timestamp += seven_days);
    assert!(client.is_claimable());
    client.claim_heir_share(&heir);
    assert_eq!(client.unclaimed_balance(), 0);
}
