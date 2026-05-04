#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, Symbol, Vec,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const LEDGER_LIFETIME: u32 = 17_280 * 30; // ~30 days of ledger entries

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Portfolio(Address),   // Each user's current holdings
    TradeHistory(Address), // Each user's list of trades
    Initialized(Address), // Whether a user has been funded
    Leaderboard,          // Global sorted scores (P&L %)
}

// ─── Data Structures ─────────────────────────────────────────────────────────

/// A single simulated trade record stored on-chain.
#[contracttype]
#[derive(Clone)]
pub struct Trade {
    pub asset: Symbol,       // e.g. "XLM", "BTC", "ETH"
    pub direction: Symbol,   // "BUY" or "SELL"
    pub amount: i128,        // units of asset (scaled by 1_000_000 for 6 decimals)
    pub price: i128,         // price in vUSDC at execution (scaled by 1_000_000)
    pub timestamp: u64,      // ledger timestamp at trade execution
}

/// A user's portfolio: maps asset symbol → quantity held (scaled 1_000_000).
/// vUSDC (virtual USDC) is the base currency; every user starts with 10_000_000_000 (10,000 vUSDC).
#[contracttype]
#[derive(Clone)]
pub struct Portfolio {
    pub holdings: Map<Symbol, i128>, // asset → quantity
    pub start_usdc: i128,            // initial vUSDC (for P&L calculation)
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct TradeFlowContract;

#[contractimpl]
impl TradeFlowContract {

    /// Fund a new practice account with 10,000 vUSDC.
    /// Can only be called once per address (enforced via Initialized key).
    pub fn fund_account(env: Env, user: Address) {
        // Require the user to authorize this action (prevents funding others)
        user.require_auth();

        // Prevent double-funding: each address gets one free allocation
        let init_key = DataKey::Initialized(user.clone());
        if env.storage().persistent().has(&init_key) {
            panic!("account already funded");
        }

        // Build an empty portfolio with 10,000 vUSDC (6 decimal precision → 10_000_000_000)
        let mut holdings: Map<Symbol, i128> = Map::new(&env);
        let usdc_key = symbol_short!("USDC");
        holdings.set(usdc_key, 10_000_000_000_i128);

        let portfolio = Portfolio {
            holdings,
            start_usdc: 10_000_000_000_i128,
        };

        // Persist portfolio and mark user as initialized
        let port_key = DataKey::Portfolio(user.clone());
        env.storage().persistent().set(&port_key, &portfolio);
        env.storage().persistent().extend_ttl(&port_key, LEDGER_LIFETIME, LEDGER_LIFETIME);

        env.storage().persistent().set(&init_key, &true);
        env.storage().persistent().extend_ttl(&init_key, LEDGER_LIFETIME, LEDGER_LIFETIME);
    }

    /// Execute a simulated DEX trade.
    /// direction: "BUY" (spend vUSDC to get asset) or "SELL" (sell asset for vUSDC).
    /// price is in vUSDC per 1 unit of asset (scaled by 1_000_000).
    /// amount is the quantity of the asset to buy/sell (scaled by 1_000_000).
    pub fn execute_trade(
        env: Env,
        user: Address,
        asset: Symbol,
        direction: Symbol,
        amount: i128,
        price: i128,
    ) {
        // The caller must authorize every trade — no one can trade on behalf of another
        user.require_auth();

        // Sanity checks
        if amount <= 0 { panic!("amount must be positive"); }
        if price <= 0  { panic!("price must be positive"); }

        // Load portfolio; user must be funded first
        let port_key = DataKey::Portfolio(user.clone());
        let mut portfolio: Portfolio = env
            .storage()
            .persistent()
            .get(&port_key)
            .expect("account not funded — call fund_account first");

        let usdc_key = symbol_short!("USDC");
        // Cost in vUSDC = (amount × price) / 1_000_000  (both are 6-decimal scaled)
        let cost = amount
            .checked_mul(price)
            .expect("overflow")
            .checked_div(1_000_000)
            .expect("div error");

        let buy_sym  = symbol_short!("BUY");
        let sell_sym = symbol_short!("SELL");

        if direction == buy_sym {
            // Deduct vUSDC, credit the asset
            let usdc_bal = portfolio.holdings.get(usdc_key.clone()).unwrap_or(0);
            if usdc_bal < cost { panic!("insufficient vUSDC balance"); }
            portfolio.holdings.set(usdc_key, usdc_bal - cost);

            let asset_bal = portfolio.holdings.get(asset.clone()).unwrap_or(0);
            portfolio.holdings.set(asset.clone(), asset_bal + amount);

        } else if direction == sell_sym {
            // Deduct the asset, credit vUSDC
            let asset_bal = portfolio.holdings.get(asset.clone()).unwrap_or(0);
            if asset_bal < amount { panic!("insufficient asset balance"); }
            portfolio.holdings.set(asset.clone(), asset_bal - amount);

            let usdc_bal = portfolio.holdings.get(usdc_key.clone()).unwrap_or(0);
            portfolio.holdings.set(usdc_key, usdc_bal + cost);

        } else {
            panic!("direction must be BUY or SELL");
        }

        // Save updated portfolio
        env.storage().persistent().set(&port_key, &portfolio);
        env.storage().persistent().extend_ttl(&port_key, LEDGER_LIFETIME, LEDGER_LIFETIME);

        // Append trade record to history
        let hist_key = DataKey::TradeHistory(user.clone());
        let mut history: Vec<Trade> = env
            .storage()
            .persistent()
            .get(&hist_key)
            .unwrap_or(Vec::new(&env));

        history.push_back(Trade {
            asset,
            direction,
            amount,
            price,
            timestamp: env.ledger().timestamp(),
        });

        env.storage().persistent().set(&hist_key, &history);
        env.storage().persistent().extend_ttl(&hist_key, LEDGER_LIFETIME, LEDGER_LIFETIME);
    }

    /// Return the current portfolio for a given user.
    pub fn get_portfolio(env: Env, user: Address) -> Portfolio {
        let port_key = DataKey::Portfolio(user);
        env.storage()
            .persistent()
            .get(&port_key)
            .expect("account not found")
    }

    /// Return the full trade history for a given user.
    pub fn get_history(env: Env, user: Address) -> Vec<Trade> {
        let hist_key = DataKey::TradeHistory(user);
        env.storage()
            .persistent()
            .get(&hist_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Calculate a user's current vUSDC-equivalent total value.
    /// This requires the caller to pass in current prices for every asset held
    /// (keyed by the same Symbol used in trades), since the contract cannot
    /// reach out to an oracle on its own.
    pub fn get_portfolio_value(
        env: Env,
        user: Address,
        prices: Map<Symbol, i128>, // asset → current price in vUSDC (6-decimal scaled)
    ) -> i128 {
        let portfolio = Self::get_portfolio(env.clone(), user);
        let usdc_key  = symbol_short!("USDC");
        let mut total = portfolio.holdings.get(usdc_key).unwrap_or(0);

        // For each non-USDC holding, multiply quantity by current price
        for (asset, quantity) in portfolio.holdings.iter() {
            if asset == symbol_short!("USDC") { continue; }
            let price = prices.get(asset).unwrap_or(0);
            let value = quantity
                .checked_mul(price)
                .unwrap_or(0)
                .checked_div(1_000_000)
                .unwrap_or(0);
            total = total.checked_add(value).unwrap_or(total);
        }

        total
    }
}