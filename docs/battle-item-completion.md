# Milestone 6: battle items and medicine

Application 0.30.0, engine 0.23.0-core, protocol 31, database schema 4.

The application-owned `rules/item-completion.json` records all 30 remaining battle/species held items and 17 ordinary medicines with exact source paths, SHA256 fingerprints, original descriptions and printed RULING pages. Source data is unchanged. New battles enable completion; saved older encounters retain their flags. Item choices, suppression, action snapshots, source ownership, persistence, privacy and rollback use the existing battle infrastructure. Medicine checkpoints include registered participants and linked trainer supplies; unrelated campaign inventories remain outside battle undo.

## Held effects

Choice items require an owner-selected learned move of the printed category. Changing a selection requires the GM. Choice Scarf's conflicting Late Reaction wording requires a recorded printed/item timing policy. Species items check exact catalog identities, including supported regional forms, rather than arbitrary evolutions. Power Increasers requires an explicitly supported trait choice. Choice Power bonuses apply to damage dice; unsupported formula interpretations remain GM adjudication.

Clear Amulet protects external drops. Loaded Dice changes native move chance dice; Metronome supports reviewed successive actions. Power Herb skips an actual charge; Protective Pads prevents supported contact triggers. Focus Sash prevents one move-caused faint per physical unit per battle and remains held. White Herb consumes its existing unit to clear recorded reductions. Orbs apply on entry; Throat Spray and Weakness Policy use bounded modifier channels. Eject Button and Red Card request controlled registered replacement using existing switching protections. Helmet and Barb use indirect-damage protections; Barb transfers its existing unit.

Air Balloon grants Ground immunity and pops on damage. Boots suppress supported entry hazards (currently Toxic Spikes); unsupported hazard definitions remain manual. Goggles prevents added effects of the supplied Spore/Powder/Pollen move cards, with unsupported user clauses visibly adjudicated. Umbrella removes supported ordinary weather effects for its holder, not global weather. Ring Target requires a GM types-only versus broader reviewed-immunity policy; unique-ability conflicts remain explicit. No unsupported move definition is made executable merely by holding an item.

## Medicine and Recycle

Potion, Super Potion, Hyper Potion, Energy Root and Remedy retain physical partial-dose units. Inventory decreases once per opened unit; remaining doses persist. All other medicines are single-use, including Antidote despite stale source metadata. Medicine requires owned supplies on the actor or linked trainer, a registered eligible ally, confirmed battle-pocket access and the acting Pokemon's turn/action. A GM can attest trainer-on-field use against the trainer action budget. Magic Room, fainting eligibility, healing blocks, Shadow Force encounter debt and the shared three-HP-per-round allowance apply.

Max Potion and Full Restore bypass the three-HP cap and persist a healing lock until the next campaign day. Full Heal and Heal Powder clear ailments, not unrelated volatile conditions. Revive restores one HP. Max Revive/Revival Herb have conflicting full-healing versus exception-list wording: the GM chooses full or capped healing. Revival may target an owned fainted reserve without deploying it. Herbal acceptance and happiness consequences require a recorded GM ruling; Gluttony ignores negative ingestion effects. No numeric happiness penalty is invented.

Recycle supports all 17 medicines and selected registered recipients, including eligible fainted reserves. Existing three-attempt limits remain. Recycled Power Herb and Air Balloon provide bounded virtual charges without manufacturing inventory; Balloon duration requires an explicit GM ruling. White Herb clears recorded reductions. Unsupported reusable effects remain visibly manual. Lethal healing, crafting, capture, narrative ingestion and automatic Will recovery remain excluded.

## Verification

Regression coverage includes real attacks, chance selectors, charge skipping, stat/species restrictions, contact effects, switching, all 17 medicine commands, dose conservation, healing limits, revival, herbal rulings, historical flags, privacy, rollback and host/owner/GM/observer room transport with retries and reconnects. Native installation, upgrades, data preservation and physical LAN acceptance remain milestone 7.

Validated: 651 Rust tests, 58 frontend tests and 17 catalog/script checks; TypeScript and warnings-denied Clippy pass. Production Vite/Tauri build and Windows NSIS packaging passed. Installer: `src-tauri/target/release/bundle/nsis/PokeVerse_0.30.0_x64-setup.exe` (about 215 MiB). The installer was not launched.
