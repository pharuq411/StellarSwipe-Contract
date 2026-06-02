use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCategory {
    Validation = 1,
    Authorization = 2,
    ExternalDependency = 3,
    Arithmetic = 4,
    Upgrade = 5,
    Network = 6,
    Recovery = 7,
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RecoveryStrategy {
    Retry = 1,
    Defer = 2,
    Escalate = 3,
    ManualReview = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorReport {
    pub category: ErrorCategory,
    pub strategy: RecoveryStrategy,
    pub message: String,
    pub timestamp: u64,
}
