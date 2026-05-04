# TradeFlow

> Paper trading on Stellar's live DEX — zero risk, real mechanics.

---

## Problem

A 22-year-old finance student in Manila wants to learn crypto trading but risks losing real money on volatile assets — so they never start, missing critical hands-on experience before entering real markets.

## Solution

TradeFlow gives every user a funded virtual USDC account (10,000 vUSDC) and lets them execute simulated trades directly against Stellar's built-in DEX on testnet. Every swap, buy, and sell is a real on-chain transaction — so users build genuine muscle memory on actual Stellar infrastructure without financial risk.

---

## Timeline

| Phase | Scope |
|-------|-------|
| Day 1 | Soroban contract: fund_account, execute_trade, get_portfolio |
| Day 2 | Web frontend: wallet connect, trade form, portfolio dashboard |
| Day 3 | AI coach integration, leaderboard, polish & testnet deploy |

---

## Stellar Features Used

| Feature | How TradeFlow uses it |
|---|---|
| **USDC** | Virtual base currency for all trades |
| **Built-in DEX** | Real swap routing on testnet |
| **Trustlines** | Required to hold custom practice assets |
| **Custom tokens** | vXLM, vBTC, vETH issued as practice assets |
| **Soroban smart contracts** | Portfolio state, trade history, P&L on-chain |

---

## Vision & Purpose

TradeFlow solves the cold-start problem in DeFi education: most people are too afraid to put real money in before they understand how things work, but you can't learn without doing. By running every trade through real Stellar infrastructure on testnet, TradeFlow closes that gap — when users graduate to real funds, they already know the mechanics intimately. The long-term vision is a leaderboard-driven community where top traders earn real token rewards.

---

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | `^1.74` |
| Soroban CLI | `^21.0.0` |
| `stellar-cli` | `^21.0.0` |
| Node.js (optional, for frontend) | `^18` |

Install Soroban CLI:

```bash
cargo install --locked stellar-cli --features opt
```

---

## Build

```bash
# Build the Wasm binary optimised for Stellar
soroban contract build
```

Output: `target/wasm32-unknown-unknown/release/trade_flow.wasm`

---

## Test

```bash
# Run all 5 unit tests
cargo test

# Run with output visible
cargo test -- --nocapture
```

---

## Deploy to Testnet

```bash
# 1. Configure testnet identity (one-time)
stellar keys generate --global alice --network testnet
stellar keys fund alice --network testnet

# 2. Deploy the compiled Wasm
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/trade_flow.wasm \
  --source alice \
  --network testnet

# Output: CONTRACT_ID (save this for the CLI invocations below)
```

---

## Sample CLI Invocations

Replace `<CONTRACT_ID>` and `<USER_ADDRESS>` with your values.

### Fund a practice account

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  fund_account \
  --user <USER_ADDRESS>
```

### Execute a trade — Buy 10 XLM at 0.20 vUSDC

```bash
# amount = 10 XLM × 1_000_000 = 10_000_000
# price  = 0.20 vUSDC × 1_000_000 = 200_000

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  execute_trade \
  --user <USER_ADDRESS> \
  --asset XLM \
  --direction BUY \
  --amount 10000000 \
  --price 200000
```

### Check portfolio

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  get_portfolio \
  --user <USER_ADDRESS>
```

### Get trade history

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  get_history \
  --user <USER_ADDRESS>
```

### Get portfolio value (pass current prices)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  get_portfolio_value \
  --user <USER_ADDRESS> \
  --prices '{"XLM": 250000, "BTC": 60000000000}'
```

---

## Project Structure

```
trade_flow/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      # Soroban smart contract
    └── test.rs     # 5 unit tests
```

---

## contracts

🔗 https://stellar.expert/explorer/testnet/tx/73019b2bf4234c86297a4af48ad8cd59d0c73d13988263dcdf7e42d85185ebf3
🔗 https://lab.stellar.org/r/testnet/contract/CCAGFBOVPF6HNFD6OWWWMEHQU7W3I7LFWQ4T27P6P5CKMZ5WPJAJQH3J

## License

MIT — see [LICENSE](LICENSE)