# Special moves — 0.15.0

Engine `0.8.0-core`; protocol 16; user database schema 4. Original Pokerole JSON/PDF files are unchanged. Reviewed corrections are application owned, and participate in catalog compatibility fingerprints. Historical battles retain their recorded engine; campaigns adopt the new rules/catalog between battles.

## Supported behavior

- Healing: Basic/Complete/Minor caps, successful-use Will costs, weather-dependent amounts, drains, Liquid Ooze, recovery blocks, status cures, Rest, Roost, Stockpile/Swallow, delayed Wish, Aqua Ring/Ingrain/Life Dew, Leech Seed and explicit revival/sacrifice exceptions. Ordinary healing never revives a fainted participant. Training resources and permanent sheets are not rewritten by these effects.
- Shields: once-per-round use, escalating consecutive-round accuracy penalties, reduction and effect protection by shield type, contact retaliation, priority restrictions and Endure rearming on its user's turn.
- Fixed/varied damage: card-specific formulas, rank pools, type immunity, defense bypass and named caps. Final Gambit and Endeavor deliberately follow the supplied Pokerole wording. Bide requires a recorded GM decision about its ambiguous damage basis before preparation is accepted.
- Conditional damage: HP, Dexterity, Happiness, Will, weight, battlefield and ordered-hit conditions. Shared attacks freeze special damage contributions with their existing damage context.
- Charge: roll accuracy during preparation, spend a second action on a later user turn to release, then open responses without rerolling accuracy. Weather shortcuts, extra preparation, preparation bonuses, out-of-range and vulnerable states, lifting restrictions and charge cancellation apply. Clash uses the charging move without preparation and cancels stored preparation.
- Rampage: up to three consecutive uses per round, including Clash while already rampaging; explicit ending applies Confusion. Ending is required before Evasion or another move.
- Successive attacks: commit accuracy rolls and actions together, same target and turn, separate hit resolution/responses, reviewed per-hit changes. Reaction sequences suspend and then resume the original incoming attack. Five actions per round and one Evasion per turn remain enforced.
- Recoil/recharge: recoil rerolls raw damage successes, with reviewed ability exceptions; successful hits record next-round recharge obligations, including shared attacks and recall. No recharge is added for failed or cancelled attacks.

The Special moves panel provides successive sequences, preparation/release, explicit choices and host random draws. Only the GM can change encounter weight/lifting/night parameters. Existing accuracy, response, damage and effect controls handle ordinary resolution. Parameters and exceptions are logged through authoritative commands; the host and GM can be separate identities.

## Evidence and explicit limits

Family rules: RULING pp.61,73–83; lifting and Snatch details in MASTER pp.531–532. Individual printed card pages and exact scopes are recorded in `rules/special-moves.json`, also exposed on individual catalog entries as `SpecialRules`. Generated inventory: `outputs/rule-coverage/special-move-coverage.json` in the parent workspace.

The registry covers 213 selected family definitions: 209 have automated scopes, three are partial and one is manual. This is not a claim that all 892 moves or every interaction is automated.

- Bide: explicit GM choice resolves the printed User/Target ambiguity; the chosen basis is recorded and then calculated automatically.
- Snatch: supported move modifiers, shields, healing targeting and typed side effects transfer. Decoys are implemented in 0.17.0; ability-modifier transfer and unmodelled effects remain GM adjudication.
- Shadow Force: preparation, bypass and healing debt inside the encounter are supported. Enforcing its 24-hour debt after leaving the encounter remains manual.
- Max Guard: explicitly out of scope with Dynamax, all Max Moves and Z-Moves.
- Missing weights, painful Sky Drop lifting and other unspecified parameters require a logged GM value. No video-game lifting or move formula is substituted.

Trainer-target damage, lethal wounds and automatic Will recovery remain excluded. Transformation/copying/suppression and held-item systems remain separate work.

## Validation

Rust fixtures exercise preparation/release, weather, rampage, successive own-turn/reaction sequences, fixed and varied pools, recoil-related abilities, healing caps/costs, revival, sacrifice recipients, delayed/ongoing effects, modifier channels, lifting, ice activation, healing blocks, Endure, rollback, receipts, stale revisions and real separate host/GM reconnects. Frontend checks cover panel visibility and permissions; catalog checks preserve registry hashes, boundaries, source corrections and stable references.

Native startup checks use two isolated profiles. Interactive Windows battle play and physical multi-machine LAN acceptance are not covered by these automated checks.

### Release checks (16 October 2026)

- 254 Rust tests passed, including exported binding checks and actual loopback host/GM reconnects.
- 23 frontend tests and 10 catalog/browser-preview checks passed.
- TypeScript, production Vite build, Clippy with warnings denied, and Git whitespace checks passed. Windows reported an incremental-cache hard-link fallback warning; compilation and checks completed.
- Windows NSIS installer built successfully (`PokeVerse_0.15.0_x64-setup.exe`). Two hidden isolated installations started with app 0.15.0 / engine 0.8.0-core. Both library and campaign databases passed integrity checks and retained schema 4.
- Interactive Windows battles, installer install/uninstall and physical multi-machine LAN play remain unverified.

### Decoy extension (0.17.0)

The current registry has 216 selected definitions (212 automated scopes, three partial, one excluded/manual). [Decoy rules](battle-decoys.md) add Substitute, Shed Tail and Tidy Up, and extend Snatch. This does not claim complete coverage of all 892 moves.
