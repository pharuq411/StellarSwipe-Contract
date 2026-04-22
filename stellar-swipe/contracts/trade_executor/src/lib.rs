#![no_std]

mod errors;
mod risk_gates;
pub mod sdex;
pub mod triggers;

use errors::{ContractError, InsufficientBalanceDetail};
use risk_gates::{
    check_position_limit, check_user_balance, DEFAULT_ESTIMATED_COPY_TRADE_FEE,
};
use sdex::{execute_sdex_swap, min_received_from_slippage};

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Symbol, Val, Vec};

use triggers::{ORACLE_KEY, PORTFOLIO_KEY};

/// Instance storage keys.
#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Admin,
    UserPortfolio,
    /// When set to `true`, this user bypasses the per-user position cap.
    PositionLimitExempt(Address),
    /// Overrides default estimated fee used in copy-trade balance checks.
    CopyTradeEstimatedFee,
    /// Last balance shortfall for a user (cleared after a successful `execute_copy_trade`).
    LastInsufficientBalance(Address),
    SdexRouter,
}

/// Symbol invoked on the portfolio after a successful position-limit check.
pub const RECORD_COPY_POSITION_FN: &str = "record_copy_position";

const EXECUTION_LOCK: &str = "ExecLock";

#[contract]
pub struct TradeExecutorContract;

fn effective_estimated_fee(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&StorageKey::CopyTradeEstimatedFee)
        .unwrap_or(DEFAULT_ESTIMATED_COPY_TRADE_FEE)
}

#[contractimpl]
impl TradeExecutorContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&StorageKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&StorageKey::Admin, &admin);
    }

    pub fn set_user_portfolio(env: Env, portfolio: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        env.storage()
            .instance()
            .set(&StorageKey::UserPortfolio, &portfolio);
    }

    pub fn get_user_portfolio(env: Env) -> Option<Address> {
        env.storage().instance().get(&StorageKey::UserPortfolio)
    }

    pub fn set_copy_trade_estimated_fee(env: Env, fee: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        if fee < 0 {
            panic!("fee must be non-negative");
        }
        env.storage()
            .instance()
            .set(&StorageKey::CopyTradeEstimatedFee, &fee);
    }

    pub fn get_copy_trade_estimated_fee(env: Env) -> i128 {
        effective_estimated_fee(&env)
    }

    pub fn set_position_limit_exempt(env: Env, user: Address, exempt: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        let key = StorageKey::PositionLimitExempt(user);
        if exempt {
            env.storage().instance().set(&key, &true);
        } else {
            env.storage().instance().remove(&key);
        }
    }

    pub fn is_position_limit_exempt(env: Env, user: Address) -> bool {
        let key = StorageKey::PositionLimitExempt(user);
        env.storage().instance().get(&key).unwrap_or(false)
    }

    pub fn set_oracle(env: Env, oracle: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, ORACLE_KEY), &oracle);
    }

    pub fn set_stop_loss_portfolio(env: Env, portfolio: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        env.storage()
            .instance()
            .set(&Symbol::new(&env, PORTFOLIO_KEY), &portfolio);
    }

    pub fn set_stop_loss_price(
        env: Env,
        user: Address,
        trade_id: u64,
        stop_loss_price: i128,
    ) {
        user.require_auth();
        triggers::set_stop_loss(&env, &user, trade_id, stop_loss_price);
    }

    pub fn set_take_profit_price(
        env: Env,
        user: Address,
        trade_id: u64,
        take_profit_price: i128,
    ) {
        user.require_auth();
        triggers::set_take_profit(&env, &user, trade_id, take_profit_price);
    }

    pub fn check_and_trigger_stop_loss(
        env: Env,
        user: Address,
        trade_id: u64,
        asset_pair: u32,
    ) -> Result<bool, ContractError> {
        triggers::check_and_trigger_stop_loss(&env, user, trade_id, asset_pair)
    }

    pub fn check_and_trigger_take_profit(
        env: Env,
        user: Address,
        trade_id: u64,
        asset_pair: u32,
    ) -> Result<bool, ContractError> {
        triggers::check_and_trigger_take_profit(&env, user, trade_id, asset_pair)
    }

    pub fn get_insufficient_balance_detail(
        env: Env,
        user: Address,
    ) -> Option<InsufficientBalanceDetail> {
        let key = StorageKey::LastInsufficientBalance(user);
        env.storage().instance().get(&key)
    }

    pub fn execute_copy_trade(
        env: Env,
        user: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        user.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let lock_key = Symbol::new(&env, EXECUTION_LOCK);
        if env.storage().temporary().get::<_, bool>(&lock_key).unwrap_or(false) {
            return Err(ContractError::ReentrancyDetected);
        }
        env.storage().temporary().set(&lock_key, &true);

        let portfolio: Address = env
            .storage()
            .instance()
            .get(&StorageKey::UserPortfolio)
            .ok_or(ContractError::NotInitialized)?;

        let fee = effective_estimated_fee(&env);
        let bal_key = StorageKey::LastInsufficientBalance(user.clone());
        match check_user_balance(&env, &user, &token, amount, fee) {
            Ok(()) => {
                env.storage().instance().remove(&bal_key);
            }
            Err(detail) => {
                env.storage().instance().set(&bal_key, &detail);
                env.storage().temporary().remove(&Symbol::new(&env, EXECUTION_LOCK));
                return Err(ContractError::InsufficientBalance);
            }
        }

        let exempt = {
            let key = StorageKey::PositionLimitExempt(user.clone());
            env.storage().instance().get(&key).unwrap_or(false)
        };

        check_position_limit(&env, &portfolio, &user, exempt)?;

        let sym = Symbol::new(&env, RECORD_COPY_POSITION_FN);
        let mut args = Vec::<Val>::new(&env);
        args.push_back(user.clone().into_val(&env));
        env.invoke_contract::<()>(&portfolio, &sym, args);

        env.storage().temporary().remove(&Symbol::new(&env, EXECUTION_LOCK));
        Ok(())
    }

    pub fn set_sdex_router(env: Env, router: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .expect("not initialized");
        admin.require_auth();
        env.storage()
            .instance()
            .set(&StorageKey::SdexRouter, &router);
    }

    pub fn get_sdex_router(env: Env) -> Option<Address> {
        env.storage().instance().get(&StorageKey::SdexRouter)
    }

    pub fn swap(
        env: Env,
        from_token: Address,
        to_token: Address,
        amount: i128,
        min_received: i128,
    ) -> Result<i128, ContractError> {
        let router = env
            .storage()
            .instance()
            .get(&StorageKey::SdexRouter)
            .ok_or(ContractError::NotInitialized)?;
        execute_sdex_swap(
            &env,
            &router,
            &from_token,
            &to_token,
            amount,
            min_received,
        )
    }

    pub fn swap_with_slippage(
        env: Env,
        from_token: Address,
        to_token: Address,
        amount: i128,
        max_slippage_bps: u32,
    ) -> Result<i128, ContractError> {
        let min_received = min_received_from_slippage(amount, max_slippage_bps)
            .ok_or(ContractError::InvalidAmount)?;
        Self::swap(env, from_token, to_token, amount, min_received)
    }

    /// Cancel a copy trade: SDEX swap to close, then portfolio `close_position`, emit `TradeCancelled`.
    pub fn cancel_copy_trade(
        env: Env,
        caller: Address,
        user: Address,
        trade_id: u64,
        from_token: Address,
        to_token: Address,
        amount: i128,
        min_received: i128,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        if caller != user {
            return Err(ContractError::Unauthorized);
        }

        let portfolio: Address = env
            .storage()
            .instance()
            .get(&StorageKey::UserPortfolio)
            .ok_or(ContractError::NotInitialized)?;

        let exists: bool = {
            let sym = Symbol::new(&env, "has_position");
            let mut args = Vec::<Val>::new(&env);
            args.push_back(user.clone().into_val(&env));
            args.push_back(trade_id.into_val(&env));
            env.invoke_contract(&portfolio, &sym, args)
        };
        if !exists {
            return Err(ContractError::TradeNotFound);
        }

        let router: Address = env
            .storage()
            .instance()
            .get(&StorageKey::SdexRouter)
            .ok_or(ContractError::NotInitialized)?;

        let exit_price = execute_sdex_swap(
            &env,
            &router,
            &from_token,
            &to_token,
            amount,
            min_received,
        )?;

        let realized_pnl = exit_price - amount;
        let close_sym = Symbol::new(&env, "close_position");
        let mut close_args = Vec::<Val>::new(&env);
        close_args.push_back(user.clone().into_val(&env));
        close_args.push_back(trade_id.into_val(&env));
        close_args.push_back(realized_pnl.into_val(&env));
        env.invoke_contract::<()>(&portfolio, &close_sym, close_args);

        env.events().publish(
            (Symbol::new(&env, "TradeCancelled"),),
            (user, trade_id, exit_price, realized_pnl),
        );

        Ok(())
    }
}

#[cfg(test)]
mod test;
