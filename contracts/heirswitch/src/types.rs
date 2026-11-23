use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Vault,
    Claimed(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeirShare {
    pub heir: Address,
    pub share_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultState {
    pub owner: Address,
    pub token: Address,
    pub guardian: Option<Address>,
    pub inactivity_seconds: u64,
    pub last_heartbeat: u64,
    pub locked_principal: i128,
    pub total_claimed: i128,
    pub heirs: Vec<HeirShare>,
}
