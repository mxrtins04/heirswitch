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
