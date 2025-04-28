# flsol

## 📖 Overview

**$FLSOL** is a next-generation DeFi infrastructure protocol deployed on Solana.  
It allows users to stake SOL, mint liquid staking tokens ($FLSOL), and enables flash-loan contracts and bots to **flash borrow** $FLSOL for arbitrage, liquidation, and MEV extraction strategies.

The protocol combines:
- ✅ Liquid staking
- ✅ Flash loanability
- ✅ Treasury fee splitting
- ✅ Maximum flash loan limits
- ✅ Flash loan cooldown enforcement
- ✅ Dynamic fee tiers
- ✅ Full treasury management

  **THIS PROJECT IS CURRENTLY BEING DEVELOPED IN SOLANA PLAYGROUND IDE next version will be expanded to vscode**

---

## ✨ Key Features

- **Stake SOL and Mint FLSOL**  
  Users deposit SOL and receive 1:1 $FLSOL tokens, enabling liquidity and composability.

- **Flash Loan Engine for FLSOL**  
  Smart contracts and bots can flash-borrow $FLSOL for a single transaction, subject to cooldown and maximum loan limits.

- **Fee Splitting Between Vault and Treasury**  
  Flash loan fees are split between the staking vault and a protocol-controlled treasury for sustainable growth.

- **Harvesting Rewards Without Unstaking**  
  Stakers can harvest SOL rewards generated from accumulated flash loan fees without unstaking their FLSOL.

- **Cooldown Period Enforcement**  
  Flash loan users are subject to a configurable cooldown period between flash loans to prevent abuse.

- **Maximum Flash Loan Size Enforcement**  
  Protects the vault from single-transaction draining attacks by capping the amount of flash-loanable FLSOL per transaction.

- **Dynamic Flash Loan Fees**  
  Administrators can configure base fees and tiered fees based on flash loan size thresholds.

- **Emergency Pause Mechanism**  
  The protocol can pause flash loans temporarily in case of vulnerabilities or emergencies.

---

## 🏗 How It Works

| Action                  | Description |
|:-------------------------|:------------|
| **Stake SOL**             | Deposit SOL to mint $FLSOL at a 1:1 ratio. |
| **Harvest**              | Claim accumulated flash loan fees without unstaking. |
| **Unstake**               | Burn $FLSOL to redeem SOL plus accrued rewards. |
| **Flash Loan**            | Borrow $FLSOL instantly for DeFi strategies; pay fee. |
| **Cooldown and Limits**   | Cooldown slots and maximum loan sizes enforced automatically. |
| **Treasury Fee Splitting**| Portion of flash loan fees sent to a treasury wallet. |

---


## 📦 Program Accounts

| Account | Purpose |
|:--------|:--------|
| **Config** | Stores program settings (fees, treasury, cooldown, etc.). |
| **Vault** | PDA to store staked SOL and collected rewards. |
| **FLSOL Mint** | Mint account for the liquid staking token ($FLSOL). |
| **FlashRecord** | Tracks the last flash loan slot per user for cooldown enforcement. |

---

## ⚙️ Admin Controls

- Update base flash loan fees.
- Set treasury address and fee split percentage.
- Set maximum flash loan amount.
- Set flash loan cooldown period.
- Pause/unpause the flash loan engine.
- Configure dynamic fee tiers.

---

## 🛡 Security Considerations

- ✅ PDA authority locking for vault and mint.
- ✅ Cooldown periods mitigate flash loan replay risks.
- ✅ Max flash loan caps reduce economic drain attack surfaces.
- ✅ Emergency pause switch for instant risk mitigation.
- ✅ Explicit callback verification during flash loan execution.
- ✅ Fee split for protocol sustainability.

---

## 🛠 Built With

- [Solana](https://solana.com/)
- [Anchor Framework](https://book.anchor-lang.com/)
- [Rust](https://www.rust-lang.org/)
- [Solana Playground](https://beta.solpg.io/) (for initial development)

  ## 📄 License
 This project is licensed under the **MIT LICENSE**

---

# 📈 Future Plans

- Cross-DEX flash loan batching
- Reward auto-compounding for stakers
- Enhanced treasury management
- Flash loan insurance vaults

---

# 🧠 Quick Reference

| Function | Purpose |
|:---------|:--------|
| `initialize` | Deploy and configure the protocol. |
| `stake` | Stake SOL and receive $FLSOL. |
| `harvest` | Harvest flash loan rewards. |
| `unstake` | Burn $FLSOL to withdraw SOL. |
| `flash_loan` | Borrow $FLSOL temporarily and pay fees. |
| `update_fees` | Admin: Adjust flash loan fee rates. |
| `set_pause` | Admin: Pause or resume flash loans. |
| `add_fee_tier` | Admin: Add dynamic fee tiers. |
| `clear_fee_tiers` | Admin: Clear all dynamic fee tiers. |

---

