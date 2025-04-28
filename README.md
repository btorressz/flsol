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
