# Trainer Training Points — 0.9.0

The GM awards a trainer’s pool; the owner decides which Pokémon receive the points. An accepted transfer is one-way and one-for-one: 10 points leave the trainer and 10 reach the Pokémon. No automatic disobedience reduction applies to distribution. Specific earning formulas remain manual.

## Using the workflow

1. In **Room → Campaign rewards**, the appointed GM selects an active trainer, enters a positive amount and records a reason. **Training encounters** also award trainer pools, alongside capture confirmation.
2. The player selects the trainer in Room and uses **Pull campaign outcomes**. This imports the accepted pool into the personal library.
3. In **Library → Training Point pool**, choose one of that trainer’s Pokémon, amount and reason. **Review distribution** shows both balances; **Confirm distribution** saves it locally. Connection to the host and further GM approval are unnecessary.
4. Spend the Pokémon’s balance through the existing progression workflow. Synchronize the Pokémon or trainer when reconnecting: pending linked transfers automatically include both sheets and all other recipients needed by that chain.

Both sheets must be reviewed, active, owned by the same player and connected by the trainer link. Transfer amounts are whole numbers from 1–100,000; balances are bounded by the existing one-million-point limit. Known active-battle participants remain frozen.

## Authority and conflicts

The Rust service calculates and validates both balances. A command carries a stable operation ID, both expected revisions and the catalog fingerprint. One SQLite transaction updates both sheets, two history records and the receipt. Retries of the same command return the receipt; reusing its ID for another amount is rejected.

The host replays the paired debit/credit and dependent progression against intermediate trainer/Pokémon states. It rejects missing partners, forged balances, changed prerequisites, stale catalogs and competing branches. Ordinary descriptive fields still use three-way reconciliation; point balances and purchases are indivisible progression branches.

Choose **campaign values** for `/progression` to discard the entire pending linked branch, including dependent purchases. Selecting personal values cannot overwrite already accepted campaign financial history; request a GM-reviewed rebuild/correction when both branches need reconciliation. Pulling campaign outcomes validates the entire resulting chain before writing anything, so conflict handling cannot retain only a debit or only a credit. Transfers cannot be reversed by a player. GM award/correction drafts remain subject to recorded campaign approval; the explicitly labelled legacy Pokémon balance correction remains available for manual fixes.

## Compatibility and preserved data

Application **0.9.0**, multiplayer protocol **10**, user database schema **4**. Every room participant must use the same build. No database migration or new tables are required: trainers use the existing versioned Training Point field, initially zero. Legacy trainer XP is unchanged and never becomes pool credit. Existing Pokémon points, historical awards, encounters, ownership and IDs remain unchanged. Backups include sheets, financial history and command receipts.

The browser preview displays reference UI; mutations require the desktop application. Award formulas, automatic earning sessions and unrestricted player refunds remain outside this change.

## Validation

- Rust regression suite (106 passed): trainer awards, 100 → 40 + 30 + 30 allocation, resource preservation, invalid amounts/ownership/trainer links, stale revisions/catalogs, transaction rollback, idempotency, restart and SQLite backup preservation.
- Coupled replay: allocations before transfers, purchases after/between transfers, orphaned credits/debits, forged balances, conflicting branches, explicit full-branch resolution and repeated imports.
- Real loopback host/separate GM integration: encounter award, capture, reconnect, offline transfer/purchase, host synchronization and repeated import. Host authority remains separate from GM authority.
- 16 frontend tests, 6 catalog/browser-preview checks, TypeScript, production build and Rust diagnostics passed.
- Windows portable startup passed in two isolated profiles; all four databases passed integrity checks and retained schema 4. Full interactive native acceptance and installer installation/upgrade/uninstall remain unverified; they are tracked in [remaining work](REMAINING-WORK.md).
