#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use errors::SwitchError;
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, Vec};
use types::{DataKey, HeirShare, VaultState};

const MIN_INACTIVITY_SECONDS: u64 = 86_400; // 24 hours minimum
const TOTAL_BPS: u32 = 10_000;
const TTL_THRESHOLD_LEDGERS: u32 = 100_000;
const TTL_EXTEND_TO_LEDGERS: u32 = 500_000;

#[contract]
pub struct HeirSwitchContract;

#[contractimpl]
impl HeirSwitchContract {
    /// Initialize a self-custody recovery vault with inactivity timer and multi-heir splits summing to 10,000 bps.
    pub fn init(
        env: Env,
        owner: Address,
        token: Address,
        inactivity_seconds: u64,
        heirs: Vec<HeirShare>,
        guardian: Option<Address>,
    ) -> Result<(), SwitchError> {
        if env.storage().persistent().has(&DataKey::Vault) {
            return Err(SwitchError::AlreadyInitialized);
        }
        if inactivity_seconds < MIN_INACTIVITY_SECONDS {
            return Err(SwitchError::InvalidInactivityWindow);
        }
        if heirs.is_empty() {
            return Err(SwitchError::InvalidHeirSplits);
        }

        let mut sum_bps: u32 = 0;
        for h in heirs.iter() {
            if h.share_bps == 0 {
                return Err(SwitchError::InvalidHeirSplits);
            }
            sum_bps = sum_bps.saturating_add(h.share_bps);
        }
        if sum_bps != TOTAL_BPS {
            return Err(SwitchError::InvalidHeirSplits);
        }

        owner.require_auth();

        let state = VaultState {
            owner: owner.clone(),
            token: token.clone(),
            guardian,
            inactivity_seconds,
            last_heartbeat: env.ledger().timestamp(),
            locked_principal: 0,
            total_claimed: 0,
            heirs,
        };
        env.storage().persistent().set(&DataKey::Vault, &state);
        Self::bump_ttl(&env);
        env.events().publish((symbol_short!("init"), owner), token);
        Ok(())
    }

    /// Deposit funds into the recovery vault and refresh the heartbeat + TTL.
    pub fn deposit(env: Env, amount: i128) -> Result<i128, SwitchError> {
        if amount <= 0 {
            return Err(SwitchError::InvalidAmount);
        }
        let mut state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;
        state.owner.require_auth();

        let client = token::Client::new(&env, &state.token);
        client.transfer(&state.owner, &env.current_contract_address(), &amount);

        state.locked_principal += amount;
        state.last_heartbeat = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::Vault, &state);
        Self::bump_ttl(&env);

        env.events().publish((symbol_short!("deposit"), state.owner), amount);
        Ok(state.locked_principal)
    }

    /// Owner check-in: resets the inactivity timer AND explicitly bumps Soroban storage TTL.
    pub fn heartbeat(env: Env) -> Result<u64, SwitchError> {
        let mut state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;
        state.owner.require_auth();

        let now = env.ledger().timestamp();
        state.last_heartbeat = now;
        env.storage().persistent().set(&DataKey::Vault, &state);
        Self::bump_ttl(&env);

        env.events().publish((symbol_short!("alive"), state.owner), now);
        Ok(now.saturating_add(state.inactivity_seconds))
    }

    /// Return current vault state.
    pub fn get_vault(env: Env) -> Result<VaultState, SwitchError> {
        env.storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)
    }

    /// Seconds remaining before beneficiaries may claim (0 if switch is triggered).
    pub fn time_remaining(env: Env) -> Result<u64, SwitchError> {
        let state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;
        let unlock_at = state.last_heartbeat.saturating_add(state.inactivity_seconds);
        let now = env.ledger().timestamp();
        Ok(unlock_at.saturating_sub(now))
    }

    /// Optional guardian extends the owner's grace window and bumps TTL if the owner is traveling or hospitalized.
    pub fn guardian_extend(env: Env) -> Result<u64, SwitchError> {
        let mut state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;
        let guardian = state
            .guardian
            .clone()
            .ok_or(SwitchError::NoGuardianConfigured)?;
        guardian.require_auth();

        let now = env.ledger().timestamp();
        state.last_heartbeat = now;
        env.storage().persistent().set(&DataKey::Vault, &state);
        Self::bump_ttl(&env);

        env.events().publish((symbol_short!("g_ext"), guardian), now);
        Ok(now.saturating_add(state.inactivity_seconds))
    }

    /// Owner can withdraw funds at any time while alive, also refreshing heartbeat + TTL.
    pub fn owner_withdraw(env: Env, amount: i128) -> Result<i128, SwitchError> {
        if amount <= 0 {
            return Err(SwitchError::InvalidAmount);
        }
        let mut state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;
        state.owner.require_auth();

        let client = token::Client::new(&env, &state.token);
        let avail = client.balance(&env.current_contract_address());
        if amount > avail {
            return Err(SwitchError::InvalidAmount);
        }

        client.transfer(&env.current_contract_address(), &state.owner, &amount);
        state.locked_principal = state.locked_principal.saturating_sub(amount);
        state.last_heartbeat = env.ledger().timestamp();
        env.storage().persistent().set(&DataKey::Vault, &state);
        Self::bump_ttl(&env);

        env.events().publish((symbol_short!("o_wdraw"), state.owner), amount);
        Ok(state.locked_principal)
    }

    /// Beneficiary claims their exact basis-point share once the inactivity window has elapsed.
    pub fn claim_heir_share(env: Env, heir: Address) -> Result<i128, SwitchError> {
        heir.require_auth();

        let mut state: VaultState = env
            .storage()
            .persistent()
            .get(&DataKey::Vault)
            .ok_or(SwitchError::NotInitialized)?;

        let unlock_at = state.last_heartbeat.saturating_add(state.inactivity_seconds);
        if env.ledger().timestamp() < unlock_at {
            return Err(SwitchError::VaultStillActive);
        }

        let claim_key = DataKey::Claimed(heir.clone());
        if env.storage().persistent().has(&claim_key) {
            return Err(SwitchError::AlreadyClaimed);
        }

        let mut share_bps: Option<u32> = None;
        for h in state.heirs.iter() {
            if h.heir == heir {
                share_bps = Some(h.share_bps);
                break;
            }
        }
        let bps = share_bps.ok_or(SwitchError::HeirNotFound)?;

        let payout = (state.locked_principal * (bps as i128)) / (TOTAL_BPS as i128);
        let client = token::Client::new(&env, &state.token);
        client.transfer(&env.current_contract_address(), &heir, &payout);

        state.total_claimed += payout;
        env.storage().persistent().set(&claim_key, &true);
        env.storage().persistent().set(&DataKey::Vault, &state);
        env.events().publish((symbol_short!("claimed"), heir), payout);
        Ok(payout)
    }

    fn bump_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        env.storage().persistent().extend_ttl(
            &DataKey::Vault,
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
    }
}
