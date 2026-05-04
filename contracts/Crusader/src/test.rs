#[cfg(test)]
mod tests {
    use soroban_sdk::{
        symbol_short, testutils::Address as _, Address, Env, Map,
    };

    use crate::{TradeFlowContract, TradeFlowContractClient};

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Bootstraps a fresh environment, deploys the contract, and returns
    /// both a client handle and a funded test user address.
    fn setup() -> (Env, TradeFlowContractClient<'static>, Address) {
        let env   = Env::default();
        env.mock_all_auths(); // Skip signature verification in tests
        let id     = env.register_contract(None, TradeFlowContract);
        let client = TradeFlowContractClient::new(&env, &id);
        let user   = Address::generate(&env);
        (env, client, user)
    }

    // ── Test 1: Happy path ────────────────────────────────────────────────────
    // Verifies the full MVP flow: fund → buy XLM → check holdings update.

    #[test]
    fn test_buy_xlm_happy_path() {
        let (_, client, user) = setup();

        // Step 1: Fund the practice account (10,000 vUSDC)
        client.fund_account(&user);

        // Step 2: Buy 100 XLM at 0.20 vUSDC each
        // amount = 100 * 1_000_000 = 100_000_000  (6-decimal scaled)
        // price  = 0.20 * 1_000_000 = 200_000
        let amount = 100_000_000_i128;
        let price  = 200_000_i128;
        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &amount,
            &price,
        );

        // Step 3: Retrieve portfolio and verify XLM balance
        let portfolio = client.get_portfolio(&user);
        let xlm_bal   = portfolio.holdings.get(symbol_short!("XLM")).unwrap_or(0);
        assert_eq!(xlm_bal, amount, "XLM balance should equal amount bought");
    }

    // ── Test 2: Edge case ────────────────────────────────────────────────────
    // Verifies that a trade larger than the vUSDC balance is rejected.

    #[test]
    #[should_panic(expected = "insufficient vUSDC balance")]
    fn test_buy_fails_when_insufficient_balance() {
        let (_, client, user) = setup();

        client.fund_account(&user);

        // Try to buy XLM worth 50,000 vUSDC — more than the 10,000 starting balance
        // amount = 100_000 XLM, price = 0.50 vUSDC → cost = 50,000 vUSDC
        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &100_000_000_000_i128, // 100,000 XLM
            &500_000_i128,          // 0.50 vUSDC each
        );
    }

    // ── Test 3: State verification ────────────────────────────────────────────
    // Confirms that vUSDC is correctly debited after a BUY trade.

    #[test]
    fn test_usdc_debited_correctly_after_buy() {
        let (_, client, user) = setup();

        client.fund_account(&user);

        // Buy 1 XLM at 0.20 vUSDC → cost = 0.20 vUSDC = 200_000 units
        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &1_000_000_i128, // 1 XLM
            &200_000_i128,   // 0.20 vUSDC
        );

        let portfolio = client.get_portfolio(&user);
        let usdc_bal  = portfolio
            .holdings
            .get(symbol_short!("USDC"))
            .unwrap_or(0);

        // Started with 10_000_000_000 (10,000 vUSDC), spent 200_000 (0.20 vUSDC)
        let expected_usdc = 10_000_000_000_i128 - 200_000_i128;
        assert_eq!(
            usdc_bal, expected_usdc,
            "vUSDC should be reduced by exact trade cost"
        );
    }

    // ── Test 4: Sell flow ────────────────────────────────────────────────────
    // Verifies a BUY followed by a SELL returns vUSDC to the user (minus spread).

    #[test]
    fn test_sell_returns_usdc() {
        let (_, client, user) = setup();

        client.fund_account(&user);

        // Buy 10 XLM at 0.10 vUSDC → spend 1,000,000 units (1 vUSDC)
        let amount     = 10_000_000_i128; // 10 XLM
        let buy_price  = 100_000_i128;    // 0.10 vUSDC
        let sell_price = 200_000_i128;    // 0.20 vUSDC (price doubled)

        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &amount,
            &buy_price,
        );

        let before = client
            .get_portfolio(&user)
            .holdings
            .get(symbol_short!("USDC"))
            .unwrap_or(0);

        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("SELL"),
            &amount,
            &sell_price,
        );

        let after = client
            .get_portfolio(&user)
            .holdings
            .get(symbol_short!("USDC"))
            .unwrap_or(0);

        // Selling at twice the buy price should yield more vUSDC than before the sell
        assert!(after > before, "USDC should increase after selling at a higher price");
    }

    // ── Test 5: Trade history is recorded ────────────────────────────────────
    // Confirms that each executed trade is appended to on-chain history.

    #[test]
    fn test_trade_history_recorded() {
        let (_, client, user) = setup();

        client.fund_account(&user);

        // Execute two separate trades
        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &5_000_000_i128,
            &200_000_i128,
        );
        client.execute_trade(
            &user,
            &symbol_short!("XLM"),
            &symbol_short!("BUY"),
            &3_000_000_i128,
            &210_000_i128,
        );

        let history = client.get_history(&user);
        assert_eq!(history.len(), 2, "trade history should contain exactly 2 entries");
    }
}