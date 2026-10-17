# Substitute decoys - 0.17.0

Engine `0.10.0-core`, protocol 18, user schema 4. The host validates automatically; GM and host may be separate. Matching builds and catalog hashes are required. Existing campaigns explicitly adopt rules/catalog between battles. Historical battles keep their recorded engine and serialized flags; absent decoy state defaults to disabled. Original Pokerole PDFs/JSON remain unchanged.

## Reviewed behavior

MASTER p.531 defines the general rules and Snatch transfer. Printed move cards are Substitute p.502, Shed Tail p.500 and Tidy Up p.505. Sound bypass follows the supplied RULING/MASTER sound rules. The application correction layer removes the incorrect Switcher tag from Substitute's source JSON.

- Casting pays exactly 2 HP and creates a 2 HP encounter-only decoy. Defenses are captured before paying the cost, including the selected profile, rank and current modifiers. No minimum-HP restriction is invented: if the cost faints the user, normal cleanup removes the decoy.
- Physical/Special move damage reduces decoy HP, without overflow into the user. Frozen defenses cannot increase or be healed. Type calculations continue to use the protected participant's effective typing; decoys are not invented independent species. A decoy does not trigger the owner's contact/damage abilities, consume Sturdy/Disguise or absorb healing.
- Targeted Support and added target effects are blocked, including the effects of the hit that breaks the decoy. Successful attacks still spend actions; supported actor costs, recoil, recharge, user switching and typing changes remain applicable. Multi-target attacks and individual successive hits resolve independently.
- Weather, terrain, existing status damage and other environmental effects still affect the Pokemon. Shields and Cover can protect the decoy using existing rules.
- Sound Moves, Infiltrator, non-ranged Physical Unseen Fist, Chip Away, Hyperspace Hole/Fury, Phantom/Shadow Force and recorded Future Sight/Doom Desire bypass decoys. A generic IgnoreShield tag alone does not imply decoy bypass; Sonic Boom hits a decoy.
- Snatch transfers remaining HP, and resets decoy defenses to the new user's resulting defenses after supported modifier theft. It does not recast or charge another HP cost. Ability-modifier transfer remains manual.
- Shed Tail uses the controlled reserve-selection flow. The selected replacement inherits the created decoy with its casting defenses and remaining HP. Owner/GM permissions, pending switch blocking, explicit no-reserve outcomes and one switcher effect per trainer/round remain enforced. GM waiver retains the decoy on the current user; withdrawal removes it.
- Ordinary departure/fainting removes a decoy. Baton Pass has no printed decoy-transfer effect and does not gain one. Tidy Up removes all decoys and recorded named entry hazards, and applies its reviewed user boosts.

## Interface and persistence

The Special moves section shows decoy HP, frozen Defense/Special Defense and history to owners, players, host and GM. Shared preview/damage calculations use the same Rust rules. Decoy interception remains serialized until the associated move effects finish, including a broken decoy. Campaign persistence, synchronization, receipts and rollback include decoys without changing permanent sheets or SQL schema.

## Explicit boundaries

Dynamax, all Max Moves (including Max Guard), and Z-Moves are out of scope. They remain in the Compendium as reference, visibly marked; current-engine battle validation rejects their automation. No transformation system is introduced.

Unseen Fist currently adds only decoy bypass; its other shield/force-field/Cover hooks remain explicitly partial. Ability copying/suppression/transfer, unsupported actor clauses on other unreviewed moves, and Shadow Force recovery debt outside battle remain separate work. Trainer-target damage, lethal wounds and automatic Will recovery remain excluded.

## Validation

Focused fixtures cover creation/cost/fainting, frozen defenses, ordinary/fixed/set damage, broken-decoy effect blocking, shields, sound/named/ability bypass, no sheet mutations, Snatch's resulting defenses, Shed Tail reserves, environmental/status ticks, multi-target/successive damage, actor switch/typing effects, serialization, departure and rollback. Real loopback host/GM/owner regression checks preserve decoys through owner replacement, duplicate commands, stale revisions and reconnect.

Interactive native battles and physical multi-machine LAN verification remain unverified. Automated release checks: 304 Rust tests, 25 frontend tests and 12 catalog/browser-preview checks pass. TypeScript, Clippy with warnings denied and the production build pass. Windows reports an incremental-cache hard-link fallback warning; compilation succeeds. Two hidden isolated Windows profiles started successfully with app 0.17.0 / engine 0.10.0-core; library and campaign databases passed integrity checks and retained schema 4. Interactive battle play, installer install/upgrade/uninstall and physical multi-machine LAN remain unverified.

Windows NSIS installer built successfully: `PokeVerse_0.17.0_x64-setup.exe` (213.47 MiB).
