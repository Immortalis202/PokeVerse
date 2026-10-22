# Battle forms and transformations

Application **0.26.0**, engine **0.19.0-core**, protocol **27** delivers ordered milestone 2. New encounters enable `TurnFlow.forms`; missing saved form state defaults to disabled. Engine 0.18 retains held items without transformations, and earlier feature gates retain their previous behavior. Rules can be upgraded between battles.

## Evidence and identities

- RULING page 133: alternate forms use separate builds. Rank points may be redistributed within each form's limits; ordinary forms share moves, HP and Will using the highest resource scores. One or two form-specific moves may differ.
- MASTER page 505: Transform copies a chosen visible model's ability, attributes, increases/reductions and moves, retains Transform and lasts the scene. It does not grant a copy of the model's HP, Will, skills, rank or equipment.
- MASTER page 606: exact held Mega stone, trainer Key Stone, activation on the Pokemon's turn, one Will, one Mega per trainer per battle, same rank with redistributed points, optionally different moves, whole-scene duration through recall, restored Mega HP/Will/statuses, original resource debt retained, protected stones and fainted restoration.
- The supplied v3.0 ability and species JSON cards are preserved. `rules/battle-forms.json` records exact application identities, baseline attributes/limits, typing, abilities, original source paths and byte hashes for **127 records**, plus **47 distinct Mega-stone mappings**. No runtime name guessing selects a form. A catalog check verifies every recorded source/hash/value and identity.

Some existing broad catalog baseline corrections confuse adjacent alternate forms, including Aegislash and Darmanitan. The form registry uses the exact supplied form records rather than those ambiguous aggregate corrections. This does not declare the broader species audit complete or rewrite existing permanent character attributes.

The form registry is fingerprinted by the coverage registry and participates in the catalog compatibility hash. No supplied source files are edited.

## State, builds and authority

Form views hold scene-local species identity, attributes, skills, moves, types, ability channels and labels. The permanent sheet is retained. Original resource snapshots are created only when a Pokemon actually changes form, avoiding a full party-sheet copy in every ordinary battle snapshot.

Owners prepare form builds, choose their eligible form/Transform options and control Multitype. The host validates rank budgets, exact form limits, skill/social caps, move definitions and capacity. Schooling's build uses its temporary two-rank increase. Ordinary builds allow at most two reviewed form-specific move differences; Mega builds can select a different rank-eligible movepool. Existing learned moves remain valid. Blank optional social/skill/move groups retain the original build's group.

GM commands require a reason and handle source-unspecified arrival/equipment/joint-form decisions. Missing masks, Discs and Key Stones are recorded as attestations, never minted catalog items or inventory counts. A Disc/mask attestation requires an otherwise empty held slot. Ordinary equipment still uses the owned-unit engine.

The battle panel displays the selected form and corresponding sprite, appearance-only Illusion, both joint ability channels, preparation controls and resource decisions. Attack, reaction, special-move and conflicting-timing selectors use the current scene movepool. Permanent creation/progression sheets keep their original forms.

## Reviewed ability entries

| Entry | Reviewed behavior and explicit boundary |
| --- | --- |
| As One | GM selects a reviewed joint-form build and one available ability from each conscious deployed ally. Movepools combine; supported passive, entry and event hooks recognize both channels. The component cannot act separately and returns at scene end. The card does not give a universal numeric fusion formula, so the selected build and reason are explicit. Conflicting conversion/Contrary-Simple order, Imposter nesting and further changes of a joint body require a GM resolution instead of an invented precedence. Registry remains partial for these boundaries. |
| Commander | Entry selects an active allied Dondozo (the supplied card spells it "Dodonzo"). It gains two attributes and defenses and is blocked for the scene; damage to Commander is redirected to it. This does not introduce the video-game targeting/hiding rules absent from the card. |
| Disguise | Existing first-move-hit protection remains once per scene. The spent flag is visible. A nonexistent Busted catalog entry is not fabricated; weather/condition damage does not spend the protection. |
| Embody Aspect | A recorded Teal/Wellspring mask grants two Dexterity; Hearthflame/Cornerstone grants two Strength. No invented mask stock or general Terastallization feature is implied. |
| Gulp Missile | Surf/Dive damage selects exact Gulping above half HP or Gorging at half HP or less. A subsequent damaging foe hit saves one retaliation: the host rolls two damage dice and returns Cramorant to regular form. The printed unspecified added effect is recorded by the GM; no automatic video-game effect is guessed. |
| Hunger Switch | Toggle Hangry/Full-Belly at round end. Aura Wheel uses the corresponding type. The catalog has one Morpeko identity, so the alternate appearance is a scene label; no duplicate species is fabricated. Outside-battle hunger is excluded. |
| Ice Face | Track the two additional ice HP separately from ordinary HP. After both are damaged, select the exact No Ice form. Restore the ice after a complete conscious, deployed Snow/Hail round, not immediately when snow starts or the ice breaks. |
| Illusion | Owner chooses another party Pokemon on entry; display its appearance without copying mechanical stats. Any damage breaks the appearance. The application does not add a separate detection contest or outside-battle illusion system. |
| Imposter | Entry chooses a visible active model and applies the reviewed Transform implementation. |
| Multitype | Arceus can choose one of 18 types at any time; identity remains unique. Existing attack snapshots retain their saved type/stat inputs. |
| Power Construct | At round end, if at half HP or less, use the next exact Cell/10%/50%/100% mapping, restoring full HP, Will and statuses on the change. No shortcut to the video-game complete form is substituted. |
| RKS System | The type follows a GM-attested Disc. Missing Disc catalog entries remain an explicit evidence requirement; no stock is fabricated. |
| Schooling | Call the school at half HP; GM confirms the unspecified arrival timing. Healing, fainting or recall cancels the current call. Arrival uses the exact Swarm/School record, restores full HP and raises rank by two. Restore the ordinary form at scene end. |
| Shields Down | Half-HP Minior changes to exact Core at full HP, with status immunity. Core identity is saved in campaign memory until the GM records release and later recapture. Memory and the triggering battle state roll back together. |
| Stance Change | At round start choose an exact Shield/Blade form and its rank-point build. Shield allows Support only; Blade allows Physical/Special only. The highest HP/Will are shared. |
| Tera Shell | Full-health Terapagos receives Not-Very-Effective Physical/Special damage in ordinary, reviewed formula and Clash paths. |
| Tera Shift | At round start optionally use exact Terastal Terapagos, restore full HP/Will/statuses and materialize its Tera Shell identity. |
| Teraform Zero | Active Terapagos removes supported field effects, terrain, natural/ability weather and pending field events, and suppresses ordinary abilities. They can become active again after its departure; removed field effects are not resurrected. Unique-ability stalemates remain a GM ruling and registry partial boundary. |
| Zen Mode | End a round at half HP or less to choose the exact regional Zen form and its prepared limits/build, retaining current resources. |
| Zero to Hero | Recall enables exact Hero Palafin; its Hero build remains available on return. Recalled participants remain registered for transformation/ownership checks. |

Other temporary forms are encounter-local; outside-battle narrative persistence and changes to permanent sheets remain GM-managed. The explicit persistent Core rule is tracked separately rather than silently restoring Minior at scene end. Scene-local form handling does not implement every possible arbitrary creature/object copy or resolve unprinted fusion formulas.

## Mega Evolution and restoration

Activation checks the exact base identity and held stone, linked trainer, recorded worn Key Stone, turn, conscious readiness, one Will and that trainer's unused Mega slot. Klutz does not suppress Mega stones; Magic Room prevents item use. Mega stones cannot be thrown, stolen, exchanged or removed during battle, including by a generic GM item-removal command.

The Mega body starts with full form HP/Will and cleared statuses. Existing positive/negative battle channels remain, and Mega attributes may exceed the ordinary ten-point buff ceiling. Recall and fainting do not cancel the Mega body. Ending the scene restores the original HP, Will after its activation cost, statuses and conditions recorded at activation. A fainted Mega returns unconscious even if its original body had HP remaining. Battle-end rollback restores the transformed encounter, not an intermediate restored body.

General automatic Will recovery remains excluded; the explicit restoration printed on Mega Evolution, Power Construct and Tera Shift is scoped to those transformations.

## Snapshots, privacy and compatibility

Current form sheets feed shared accuracy, damage, defense and typing calculations. Existing `DamageContext`/field snapshots preserve the inputs of an already saved shared attack through later type/form changes. Supported copied stat channels retain move-versus-ability provenance. As One's secondary channel is retained in frozen ability snapshots.

Foreign projections redact equipment inside form/original sheets and hide other owners' prepared builds, including archives and undo checkpoints. The host keeps the full authoritative state. Stable commands use the normal permission, revision, receipt, duplicate, reconnect and persistence mechanisms; form decisions create battle checkpoints. Old saves omit forms/Core memory safely. No schema migration or supplied-source modification is required.

## Verification

Release checks for application 0.26.0:

- 583 Rust checks passed: 100 binding exports and 483 behavioral tests. After the final HP-only restoration correction, all 21 local transformation tests and the localhost transformation room test passed again. The room fixture covers owner permissions, duplicate receipts, stale revisions, reconnect, persistence, privacy and rollback.
- 51 frontend tests and 13 catalog checks passed. The form registry fixture verifies all 127 source records and all 47 Mega mappings; Rust exercises every Mega activation and restoration.
- TypeScript, Rust formatting, warnings-denied Clippy and the production frontend build passed.
- Windows release and NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.26.0_x64-setup.exe` (214.22 MiB). Neither executable nor installer was launched. The sandboxed bundler could not resolve GitHub for NSIS; the authorized network-enabled retry succeeded.

Native interaction, installed upgrades, trusted signing and physical LAN remain separate acceptance work in milestone 7. Build checks do not launch the executable or installer.
