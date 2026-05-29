use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    Admin,
    Oracle,
    OracleAssetPair,
    NextPositionId,
    Position(u64),
    /// V1: mixed open+closed list (preserved for migration reads).
    UserPositions(Address),
    UserOpenPositions(Address),
    UserClosedPositions(Address),
    /// Registered TradeExecutor contract allowed to call `close_position_keeper`.
    TradeExecutor,
    /// Per-user KYC verification flag (bool). No PII stored — boolean only.
    KycVerified(Address),
    /// Global KYC-required mode (bool). When true, only KYC-verified users can trade.
    KycRequiredMode,
    /// Per-user geographic restriction flag (bool). Restricted users cannot trade.
    Restricted(Address),
    /// Per-user current streak (consecutive profitable closes)
    CurrentStreak(Address),
    /// Per-user best streak observed
    BestStreak(Address),
    /// Migration: marks a user as already migrated from V1 to V2 layout.
    MigratedUser(Address),
    /// Migration: queue of users pending V1→V2 migration.
    MigrationQueue,
    /// Per-user notification preferences (Issue #430).
    NotificationPrefs(Address),
    /// Per-user achievement list (Issue #432).
    UserAchievements(Address),
    /// Anchor deposit destination address by token.
    AnchorDepositAddress(Address),
    // Badge-related keys used by badges.rs
    UserBadges(Address),
    UserClosedTradeCount(Address),
    UserProfitStreak(Address),
    LeaderboardRank(Address),
    EarlyAdopterCap,
    TotalUsersFirstOpen,
    /// Per-user signal watchlist (Issue: signal watchlist).
    Watchlist(Address),
}
