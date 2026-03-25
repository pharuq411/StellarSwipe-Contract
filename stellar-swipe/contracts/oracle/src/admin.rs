use soroban_sdk::{Address, Env, Map, String, contracttype};
use stellar_swipe_common::emergency::{PauseState, CAT_ALL};

use crate::errors::OracleError;
use crate::types::StorageKey;

pub fn pause_category(
    env: &Env,
    caller: &Address,
    category: String,
    duration: Option<u64>,
    reason: String,
) -> Result<(), OracleError> {
    require_admin(env, caller)?;
    caller.require_auth();

    let now = env.ledger().timestamp();
    let auto_unpause_at = duration.map(|d| now + d);

    let pause_state = PauseState {
        paused: true,
        paused_at: now,
        auto_unpause_at,
        reason: reason.clone(),
    };

    let mut states = get_pause_states(env);
    states.set(category.clone(), pause_state);
    env.storage().instance().set(&StorageKey::PauseStates, &states);

    Ok(())
}

pub fn unpause_category(env: &Env, caller: &Address, category: String) -> Result<(), OracleError> {
    require_admin(env, caller)?;
    caller.require_auth();

    let mut states = get_pause_states(env);
    if states.contains_key(category.clone()) {
        states.remove(category.clone());
        env.storage().instance().set(&StorageKey::PauseStates, &states);
    }
    Ok(())
}

pub fn get_pause_states(env: &Env) -> Map<String, PauseState> {
    env.storage().instance().get(&StorageKey::PauseStates).unwrap_or(Map::new(env))
}

pub fn is_paused(env: &Env, category: String) -> bool {
    let states = get_pause_states(env);
    
    if let Some(all_pause) = states.get(String::from_str(env, CAT_ALL)) {
        if is_state_active(env, &all_pause) {
            return true;
        }
    }
    
    if let Some(pause) = states.get(category) {
        return is_state_active(env, &pause);
    }
    
    false
}

fn is_state_active(env: &Env, state: &PauseState) -> bool {
    if !state.paused { return false; }
    if let Some(auto) = state.auto_unpause_at {
        if env.ledger().timestamp() >= auto { return false; }
    }
    true
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), OracleError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&StorageKey::Admin)
        .ok_or(OracleError::Unauthorized)?;

    if caller != &admin {
        return Err(OracleError::Unauthorized);
    }
    Ok(())
}
