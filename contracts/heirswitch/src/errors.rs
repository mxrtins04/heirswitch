use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SwitchError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidInactivityWindow = 3,
    InvalidHeirSplits = 4,
    InvalidAmount = 5,
    VaultStillActive = 6,
    HeirNotFound = 7,
    AlreadyClaimed = 8,
    NoGuardianConfigured = 9,
}
