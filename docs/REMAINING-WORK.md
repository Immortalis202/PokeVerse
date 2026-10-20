# PokeVerse - remaining work

Snapshot: **20 October 2026**, application **0.21.0**, engine **0.14.0-core**, protocol **22**, user database schema **4**.

This is the current project backlog. Implemented foundations are summarized first; unfinished features, explicit GM rulings, verification gaps and excluded scope are distinguished below. Counts come from the checked-in registries and should be refreshed after catalog changes.

## Implemented foundation

Offline trainer/Pokémon creation and named progression use shared Rust budgets, costs, prerequisites and derived resources. Campaign synchronization supports accepted offline purchases, indivisible progression chains and scoped GM exception approval. The host appoints the GM and automatically validates normal commands; these roles can be separate people.

Training encounters use the reviewed habitat pool, private preparation, publication, assumed victory, captures and trainer assignment. The GM awards Training Points to a trainer pool; owners distribute accepted points among that trainer's Pokémon offline. Existing Pokémon balances are preserved.

Battles register full teams, three active slots per side, reserves, controlled switching, accuracy/damage automation, multi-target responses, field effects, supported abilities, decoys, overrides and rollback. Evasion is once per turn; eligible Clash can answer each incoming hit within the action budget. The Compendium works offline for all 1,200 species/forms.

## 1. Remaining battle systems

- [ ] **Held items:** model equip/unequip, ownership, consumption/loss/theft, provenance, trigger choices, rollback and synchronization. Integrate berries, item-dependent abilities and Magic Room/Klutz restrictions without implying reference-only inventory is an active held-item engine.
- [ ] **Forms and transformations:** define supported temporary form changes, entry/exit, stat/type/ability replacement and restoration, preserving permanent sheets and frozen attack inputs. Review each ability separately. Excluded mechanics below must remain excluded.
- [ ] **Additional action sequencing:** review and implement Dancer, Parental Bond, Gorilla Tactics, Truant and Defeatist using Pokerole wording, reaction/action accounting and idempotent owner choices.
- [ ] **Other ability families:** damage provenance (Tinted Lens, Unaware, Sheer Force, Stakeout), weight/gender formulas, private information reveals, random stat choices and remaining event definitions. The full ability inventory is in sections 8-9.
- [ ] **Unreviewed trait copying:** Costar and Opportunist now copy supported attribute/defense move and ability channels. Baton Pass transfers supported positive attribute channels, and Snatch steals supported ability modifiers. Review additional trait channels separately; temporary ability identity copying remains distinct. See [stat copying and transfer](battle-stat-channels.md).
- [ ] **Remaining identity effects:** review Entrainment, Doodle and Core Enforcer individually. Role Play, Skill Swap, Gastro Acid, Trace, Gas, contact replacement and witnessed ally inheritance already exist.
- [ ] **Status edge cases:** review severe Burn/Poison escalation with conflicting printed values, unsupported status-producing/cure effects and remaining Will spending/draining effects. Core statuses, durations and expiry already exist; automatic Will recovery remains excluded.
- [ ] **Ice edge cases:** review any still-adjudicated Snow/special-damage interactions against ice against current fixtures before adding mechanics. Ordinary ice attempts and supported special interactions already have tests.
- [ ] **Unreviewed move effects:** extend executable definitions beyond the selected special/targeting registries. Ordinary damage automation does not imply every added effect on all 892 moves is executable. Preserve explicit GM resolution for unsupported cards.

## 2. Remaining named move and battlefield boundaries

| Entry | Remaining work or intentional ruling |
| --- | --- |
| Baton Pass | Supported positive move and ability attribute channels and replacement rules are implemented. |
| Snatch | Review additional effects; supported move/ability modifiers, shields, decoys and typed effects already transfer. |
| Shadow Force | Persist its 24-hour recovery restriction after leaving the encounter; encounter-local debt already works. |
| Bide | Keep the recorded GM choice for contradictory printed User/Target wording. Do not silently select an interpretation. |
| Magic Coat | Extend supported reflected Support effects. Keep the explicit GM duration ruling because the card does not state one. |
| Mycelium Might | Resolve printed Reaction versus granted Late Reaction collisions explicitly. Pokerole grants Never Miss/Late Reaction, not video-game ability bypass. |
| Tangled Feet | Support different accuracy-success penalties per target within one shared roll; currently mixed penalties require adjudication. |
| Teravolt / Turboblaze | Extend held-item and unsupported effect-blocker interactions; retain explicit conflicting unique-ability rulings. Current type/damage/status immunity, supported shields/decoys, redirection and charge-target access are implemented. |
| Neutralizing Gas | Opposing emitters require a recorded GM policy. This is a rules stalemate, not a missing host validation step. |
| Receiver / Power of Alchemy | Narrative acquisition outside witnessed campaign fainting; explicit timing for switching active copies during combat. Witnessed acquisition, one active copy, game-time expiry and persistence already work. |
| Terrain overlaps | Preserve recorded GM decisions where the manual does not define precedence. Do not substitute video-game terrain replacement rules. |
| Torn World | Define or record the retargeting distribution before automating it. |
| Arena examples | Supply GM parameters where examples omit healing amounts or check difficulty; narrative escape and encounter-rate effects remain manual. |
| Magic Room | Integrate the future held-item engine; recording a field effect is already supported. |
| Night parameters | Reconcile the legacy per-Pokémon Moonlight night parameter with the new battlefield night setting, preserving explicitly logged GM overrides. Mind's Eye currently uses the battlefield setting. |

These are not all bugs to eliminate: contradictory or unspecified rules can remain explicit, logged GM decisions. New definitions must include reviewed evidence, previews, host validation, history, retry/rollback behavior and meaningful fixtures.

## 3. Catalog, creation fidelity and assets

- [ ] Finish review of unresolved/unverified species baselines and limits, forms, move ranks, evolution branches/prerequisites and ability availability. Fix the application-owned correction layer, never the supplied source data.
- [ ] Resolve **35 missing catalog references**, preserving exact species/form identities. Missing/ambiguous matches must remain visible until reviewed.
- [ ] Supply or review **168 missing species/form sprites**; current placeholders retain names and navigation.
- [ ] Work through the field-level audit queue in `outputs/rule-coverage/` in the parent workspace. An unverified field is not necessarily wrong, and a resolved link is not proof that prerequisites were reviewed.
- [ ] Continue manual habitat review when desired. Keep accepted CSV assignments, explicit exclusions, conditional notes and persistent user overrides authoritative; do not regenerate type-affinity suggestions over them.
- [ ] Audit the older broad coverage-matrix descriptions as individual mechanics advance. Per-definition registries and executable fixtures define precise coverage.

Current catalog: 1,200 species/forms and 892 moves. The special-move registry covers 221 selected definitions (217 automated, three partial, one excluded/manual); the targeting registry covers 16 moves (15 automated, one partial). These are overlapping scopes, not a complete move catalog or a sum of distinct supported moves.

## 4. Windows, multiplayer and distribution acceptance

Headless localhost WebSocket tests establish separate host/GM/owner authority, receipts, reconnect, rollback and database persistence. They do not establish interactive Windows or physical LAN acceptance.

- [ ] Install, upgrade and uninstall the packaged app in isolated Windows installations; verify preservation of personal data and offline catalog/sprites.
- [ ] Exercise creation, progression, trainer-point distribution, Compendium links/Back navigation and dialogs at 960x650, 1280x850 and 1920x1080, including keyboard access and long content.
- [ ] Play the native three-owner versus GM gym scenario: twelve registered Pokémon, six active, reserves, repeated Clash, once-per-turn Evasion, supported field/ability effects and controlled replacement.
- [ ] Exercise training-encounter draft privacy, publication, mixed capture recipients, trainer rewards, owner library imports and subsequent offline progression.
- [ ] Exercise stale previews, duplicate submissions, disconnect/reconnect, replacement GM, conflicting offline branches and backup restoration through the native interface.
- [ ] Test direct LAN joining on separate physical Windows machines, firewall/connection errors and reconnection. Record private-VPN tests separately from direct LAN tests.
- [ ] Finish and verify the trusted Windows signing/distribution process; building an installer does not establish a trusted signature or remove Windows execution prompts. Do not automatically launch executables or installers during unattended checks.

## 5. Deliberately deferred product features

These require a separate scope decision; they are not missing parts of the assumed-victory training-encounter flow.

- Automated training-session rolls, earning formulas and attempt limits.
- Capture dice, ball consumption and automatic capture success.
- Combat simulation within training encounters.
- Full evolution-tree diagrams rather than direct relationship navigation.
- Habitat editing and character-creation shortcuts inside the Compendium.
- Narrative/outside-battle activities and private information workflows listed in the ability inventory.

## 6. Excluded scope

**Trainer-target damage, lethal wounds, automatic Will recovery, Dynamax, all Max Moves including Max Guard, and Z-Moves are excluded.** Their references may remain visible in the Compendium. Do not turn these into roadmap tasks because an old registry names a related dependency. Manual GM resource corrections remain available.

## Stat copying/transfer milestone verification

Version 0.21.0 / engine 0.14.0-core / protocol 22 implements the supported stat-channel batch described in [stat copying and transfer](battle-stat-channels.md). The full Rust suite passed 396 tests, including localhost host/GM/owner, receipt, reconnect, persistence and rollback coverage. After the final event-counter removal adjustment, the focused stat-channel suite passed all 19 tests. Frontend validation passed 31 tests and 12 catalog checks; production build and warnings-denied Clippy passed. Windows NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.21.0_x64-setup.exe` (213.67 MiB). The executable and installer were not launched. Native interaction, installed-app acceptance and physical LAN were not tested.

## 7. Suggested implementation order

1. Remaining action sequencing (Dancer, Parental Bond, Gorilla Tactics, Truant and Defeatist); supported stat copying/transfer is implemented.
2. Damage-provenance abilities and remaining event/accuracy definitions.
3. Held items and their dependent abilities.
4. Supported form changes and transformations, retaining the exclusions above.
5. Remaining reviewed move/status interactions and catalog audit work.
6. Native Windows and physical LAN acceptance, followed by distribution/signing work as needed.

Each milestone should preserve historical engines, bump authoritative versions when needed, update coverage and this document, run appropriate regressions, package Windows and commit separately.

## 8. All manual abilities (55)

Every currently manual registry entry appears below. A dependency is a reason for the current boundary, not authorization to implement an excluded mechanic. Descriptions come from the checked-in ability registry.

| Ability | Current boundary |
| --- | --- |
| Anticipation | Requires a private GM/owner information-reveal operation with reviewed move classification. |
| As One | Requires encounter form/state transformation and reviewed replacement metadata. |
| Ball Fetch | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. |
| Cheek Pouch | Requires held-item ownership, consumption, transfer or activation hooks. |
| Commander | Requires encounter form/state transformation and reviewed replacement metadata. |
| Cud Chew | Requires held-item ownership, consumption, transfer or activation hooks. |
| Dancer | Requires repeated moves, restricted move selection or per-action choices/checks beyond the current action chain. |
| Defeatist | Requires repeated moves, restricted move selection or per-action choices/checks beyond the current action chain. |
| Embody Aspect | Requires encounter form/state transformation and reviewed replacement metadata. |
| Forewarn | Requires a private GM/owner information-reveal operation with reviewed move classification. |
| Frisk | Requires held-item ownership, consumption, transfer or activation hooks. |
| Gluttony | Requires held-item ownership, consumption, transfer or activation hooks. |
| Gorilla Tactics | Requires repeated moves, restricted move selection or per-action choices/checks beyond the current action chain. |
| Gulp Missile | Requires encounter form/state transformation and reviewed replacement metadata. |
| Harvest | Requires held-item ownership, consumption, transfer or activation hooks. |
| Heavy Metal | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Honey Gather | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. |
| Hunger Switch | Requires encounter form/state transformation and reviewed replacement metadata. |
| Ice Face | Requires encounter form/state transformation and reviewed replacement metadata. |
| Illusion | Requires encounter form/state transformation and reviewed replacement metadata. |
| Imposter | Requires encounter form/state transformation and reviewed replacement metadata. |
| Klutz | Requires held-item ownership, consumption, transfer or activation hooks. |
| Light Metal | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Magician | Requires held-item ownership, consumption, transfer or activation hooks. |
| Moody | Requires independent unbiased attribute increase/reduction choices and their recorded provenance. |
| Multitype | Requires encounter form/state transformation and reviewed replacement metadata. |
| Parental Bond | Requires repeated moves, restricted move selection or per-action choices/checks beyond the current action chain. |
| Perish Body | Existing hook families are available; this individual definition and its ordering still require implementation and fixtures. |
| Pickpocket | Requires held-item ownership, consumption, transfer or activation hooks. |
| Pickup | Requires held-item ownership, consumption, transfer or activation hooks. |
| Power Construct | Requires encounter form/state transformation and reviewed replacement metadata. |
| Pressure | Existing hook families are available; this individual definition and its ordering still require implementation and fixtures. |
| RKS System | Requires encounter form/state transformation and reviewed replacement metadata. |
| Regenerator | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. |
| Ripen | Requires held-item ownership, consumption, transfer or activation hooks. |
| Rivalry | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Schooling | Requires encounter form/state transformation and reviewed replacement metadata. |
| Sheer Force | Requires reviewed effect suppression, resistance, switch timing or counterpart attribute provenance. |
| Shields Down | Requires encounter form/state transformation and reviewed replacement metadata. |
| Stakeout | Requires reviewed effect suppression, resistance, switch timing or counterpart attribute provenance. |
| Stance Change | Requires encounter form/state transformation and reviewed replacement metadata. |
| Stench | Existing hook families are available; this individual definition and its ordering still require implementation and fixtures. |
| Sticky Hold | Requires held-item ownership, consumption, transfer or activation hooks. |
| Symbiosis | Requires held-item ownership, consumption, transfer or activation hooks. |
| Tera Shell | Requires encounter form/state transformation and reviewed replacement metadata. |
| Tera Shift | Requires encounter form/state transformation and reviewed replacement metadata. |
| Teraform Zero | Requires encounter form/state transformation and reviewed replacement metadata. |
| Tinted Lens | Requires reviewed effect suppression, resistance, switch timing or counterpart attribute provenance. |
| Torrent | Printed text grants 2 damage dice but icon grants 1. GM ruling required. |
| Truant | Requires repeated moves, restricted move selection or per-action choices/checks beyond the current action chain. |
| Unaware | Requires reviewed effect suppression, resistance, switch timing or counterpart attribute provenance. |
| Unburden | Requires held-item ownership, consumption, transfer or activation hooks. |
| Unnerve | Requires held-item ownership, consumption, transfer or activation hooks. |
| Zen Mode | Requires encounter form/state transformation and reviewed replacement metadata. |
| Zero to Hero | Requires encounter form/state transformation and reviewed replacement metadata. |

## 9. All partial abilities (62)

These already have implemented mechanics. The scope is reproduced to avoid treating their whole definition as missing. Named dependencies identify the remaining work; empty dependency lists require a per-definition audit rather than an invented feature.

| Ability | Implemented scope and boundary | Remaining dependency |
| --- | --- | --- |
| Costar | Entry snapshot of one ally's supported positive move/ability attribute and defense channels; frozen owner choice for multiple allies. | unreviewed-trait-channels |
| Opportunist | Supported foe increases copied into their original stat channel, keeping the highest without reciprocal loops. | unreviewed-trait-channels |
| Arena Trap | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Aroma Veil | In-range allies and user ignore In Love and reviewed Taunt, Torment, Spite, Attract, Disable, Encore and Heal Block effects. | unreviewed-move-effects |
| Aura Break | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Big Pecks | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Clear Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Competitive | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Contrary | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Corrosion | Source-tracked Poison bypasses type/status poison immunity; global battlefield status restrictions still apply. | ability-bypass-interactions |
| Dark Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Dauntless Shield | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Defiant | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Disguise | Negate the first damaging hit in a scene. Cosmetic disguise and species/form transformations remain manual. | form-transformation |
| Early Bird | Structured Sleep duration is halved (rounded up); waking requires two cumulative Insight successes. Rest is excluded. Narrative sleep remains manual. | outside-battle-activities |
| Electromorphosis | A qualifying damaging hit charges the next Electric attack for two extra damage dice. Shared damage snapshots preserve the charge across targets. | support-move-charge-consumption |
| Fairy Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Flower Veil | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Full Metal Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Gale Wings | Add Reaction 1 to Flying moves without imposing a video-game full-HP requirement. Retain higher printed early priority; conflicting Late Reaction needs a recorded timing ruling. Charge reactions still need adjudication. | conflicting-late-priority-ruling, charge-reaction-definition |
| Guard Dog | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Hadron Engine | Add two Special under Electric Terrain or two Strength under Sunny/Desolate respectively. Weather/terrain activation follows the reviewed field definition. | held-booster-item-system |
| Hospitality | On entry the owner may spend one Will to heal one wounded active ally by three HP. No revival; lethal wounds are excluded from this project. | Lethal-resource extension is excluded; do not schedule it. |
| Hyper Cutter | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Illuminate | In-range user/allies ignore the supported fog/visibility accuracy penalty; narrative encounter effects remain manual. | narrative-encounter-effects |
| Intimidate | Entry-only rank-limited Strength reduction, non-stacking, removed when source leaves or faints. GM range defaults to all deployed foes; stable IDs break excess-target ties. An explicit GM range correction updates affected targets. | Audit the documented scope; no dependency named. |
| Intrepid Sword | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Justified | Dark hits raise the separate Strength ability channel by one, capped at three; narrative injustice triggers remain GM adjudicated. | outside-battle-activities |
| Levitate | Ground move immunity automated; terrain and entry-hazard effects remain manual. | Audit the documented scope; no dependency named. |
| Magic Bounce | Reflect executable foe Support effects aimed at the holder or its side, once, without extra actions/accuracy. Preserve original resource payer and reflected effect source. Unimplemented Support effects still require adjudication; manual field records retain reflected scope. | unsupported-support-effects |
| Magic Guard | Prevents reviewed ability damage, burn/poison ticks and Submission recoil. Confusion and direct move damage remain; weather, held items and entry hazards await their mechanics. | Audit the documented scope; no dependency named. |
| Magnet Pull | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Mega Launcher | Pulse/Aura damaging moves gain two damage dice. Heal Pulse uses five-HP Complete Healing and one Will. | other-pulse-healing-definitions |
| Mirror Armor | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Mycelium Might | Support moves gain Never Miss and Late Reaction 1; no ability bypass is granted by the printed card. Printed Reaction collisions require explicit GM adjudication. | conflicting-printed-reaction |
| Neutralizing Gas | Negates foes in abstract range subject to rank target cap; restores on source departure, fainting or range change. Unique abilities prevail. Opposing Gas stalemates require GM ruling. | GM ruling for opposing Gas stalemates |
| Oblivious | Immune to In Love; other feeling-related and narrative effects remain GM-adjudicated. | Audit the documented scope; no dependency named. |
| Orichalcum Pulse | Add two Special under Electric Terrain or two Strength under Sunny/Desolate respectively. Weather/terrain activation follows the reviewed field definition. | held-booster-item-system |
| Power of Alchemy | On active allies fainting, store multiple eligible ability snapshots for 1440 game minutes, with one active copy. Acquisition requires Power of Alchemy to be active; selecting within combat requires a GM timing ruling. Campaign clock, provenance and rollback persist. | GM ruling for selection timing during combat, Narrative acquisition outside witnessed campaign fainting |
| Prankster | Add Reaction 1 to Support moves. Retain higher printed early reaction priority; conflicting Late Reaction requires a recorded printed/ability timing ruling. Unsupported move effects and charge reaction exceptions remain explicit. | conflicting-late-priority-ruling, charge-reaction-definition |
| Protosynthesis | Under qualifying weather/terrain boost the highest attribute by one, or Dexterity by two; tied highest attributes require an owner choice. | held-booster-item-system |
| Quark Drive | Under qualifying weather/terrain boost the highest attribute by one, or Dexterity by two; tied highest attributes require an owner choice. | held-booster-item-system |
| Receiver | On an active ally fainting, the owner chooses one eligible ability snapshot. One unexpired copy, preserved in campaign state for 1440 game minutes; the GM advances game time between encounters. Unique donor abilities cannot be copied. | Narrative acquisition outside witnessed campaign fainting |
| Reckless | Recoil-tagged moves and reviewed Submission gain two damage dice; other unreviewed recoil formulas still need GM resolution. | unreviewed-recoil-moves |
| Rock Head | Ignore the supported Submission recoil; other recoil moves require reviewed definitions. | unreviewed-recoil-moves |
| Run Away | Ignore supported Blocked recall restrictions. Narrative escape and capture consequences require GM adjudication. | narrative-escape-effects |
| Sand Veil | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. | Audit the documented scope; no dependency named. |
| Serene Grace | Add two chance dice to supported move added effects; unreviewed chance effects still need GM resolution. | unreviewed-chance-moves |
| Shadow Tag | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Simple | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Snow Cloak | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. | Audit the documented scope; no dependency named. |
| Supersweet Syrup | Foes in confirmed range cannot Evade the holder moves; Clash remains available. Narrative encounter-rate changes remain GM adjudicated. | narrative-encounter-effects |
| Synchronize | Reflect source-tracked statuses once, respecting immunities. GM status edits without an inflictor have no inferred source. | Audit the documented scope; no dependency named. |
| Tangled Feet | While confused, impose two additional reduced-accuracy successes on foe moves targeting the holder; Keen Eye prevents this reduction. Mixed-target penalties on one shared accuracy roll require GM adjudication. | mixed-target-accuracy |
| Teravolt | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates, held items and unsupported effect blockers require GM adjudication. | unique-stalemates, held-item-system, unsupported-support-effects |
| Triage | Reviewed Basic Heal and status-cure Support moves gain Reaction 1, including selected allies and multi-target cures. Existing action, capacity, target, Will and phase validation apply. Unsupported healing/cure definitions remain manual. | unsupported-healing-cure-definitions |
| Turboblaze | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates, held items and unsupported effect blockers require GM adjudication. | unique-stalemates, held-item-system, unsupported-support-effects |
| Unseen Fist | Non-ranged Physical Moves bypass Substitute/Shed Tail decoys. Remaining shield, force-field and Cover bypass hooks require implementation. | shield-force-field-cover-bypass |
| White Smoke | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Wind Power | A qualifying damaging hit charges the next Electric attack for two extra damage dice. Shared damage snapshots preserve the charge across targets. | support-move-charge-consumption |
| Wonder Guard | Prevent non-super-effective move damage and supported indirect damage; condition damage still applies. | ability-bypass-interactions |
| Wonder Skin | Reduce chance dice for supported incoming added effects by two; independent selectors remain independent. | unreviewed-chance-moves |

## References and completion discipline

[Ability registry](../rules/ability-coverage.json), [special move registry](../rules/special-moves.json), [targeting registry](../rules/tactics-reviewed.json), [reviewed ability batch](battle-reviewed-ability-batch.md), [rules](RULES.md), [headless 3v3](headless-3v3.md), [training points](training-points.md), [catalog storage](catalog-storage.md).

Before closing a task, record what changed and what was actually verified. Build success, native startup, interactive use and physical-machine LAN testing are distinct results. Keep this inventory synchronized with registry status changes.
