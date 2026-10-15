# Battlefield automation

Application 0.13.0, engine 0.6.0-core, protocol 14, storage schema 4.

Adopt the corrected catalog and upgrade campaign rules **between battles**. Every room client must use the same build. Existing battles keep their recorded engine. Temporary typing, field effects and ability changes never update permanent library sheets.

## Controls and authority

The Battlefield section in Battle shows natural and temporary weather, scoped effects, durations, sources and pending checks. The appointed GM configures the environment, adds arena presets, records full-body weather protection, changes encounter typing and removes effects. Each correction requires a reason. Host and GM may be separate people; the host validates commands and supplies dice automatically.

Owners resolve optional heals and choose Hail or Snow for Snow Warning. Required environmental checks and competing weather/terrain ability rolls are resolved by the GM using host dice. Tied contestants reroll. A recorded GM resolution can clear an ambiguous event after correcting its outcome. Events and effect serials persist with campaign state and rollback; command receipts remain responsible for retry protection.

## Automated calculations

- Sunny, Rain, Sandstorm, Snow, Hail, Desolate, Typhoon and Strong Winds affect damage, defenses, permitted moves, status infliction and weather exposure. Sandstorm/Hail tick once per round; full-body protection and reviewed type/ability immunities apply. Extreme-weather checks run once at the start of each round. Natural weather resumes after a temporary source expires or leaves. Air Lock suppresses weather; Cloud Nine exempts its holder.
- Electric, Grassy, Misty and Psychic Terrain use Pokerole scopes and wording. Grassy Terrain targets the user's side, spends one Will when cast as a move and heals within the move-healing limit. Misty Terrain cures conditions and sets Dragon move power to zero. Psychic Terrain restricts grounded reaction moves; it does not turn Evasion and Clash into reaction moves. Conflicting terrain is explicitly resolved rather than silently merged.
- Reflect, Light Screen and Aurora Veil reduce final damage; overlapping protection for the same category uses the strongest reduction. Aurora Veil requires Hail/Snow to activate. Brick Break and Psychic Fangs bypass and remove applicable screens; Defog removes reviewed screens, terrains and the minefield preset.
- Trick Room reverses the remaining order without repeating completed turns. Wonder Room changes defense attribute sources according to the encounter profile. Gravity removes Flying/Levitate Ground immunity and penalizes reviewed jumping/flying moves. Magic Room is recorded; held-item mechanics are still unsupported.
- Soak, Forest's Curse, Trick-or-Treat, Burn Up and Double Shock modify encounter typing. Replacement removes previous typing modifications; an added type occupies one additional slot. Explicit scene-long changes survive recall. Forecast, STAB, effectiveness and status immunities consume effective typing.
- Thunder/Hurricane/Blizzard accuracy, Blizzard's conditional Never Miss, Weather Ball typing/dice, weather-dependent recovery, and reviewed freezing chance dice consume the same field state. Shared attacks capture source statistics, weather and effective typing before the first damage roll.

Snowscape's source JSON incorrectly identifies Hail; the owned correction activates Snow. Slush Rush uses the manual's +1 Dexterity. Sand Veil and Snow Cloak retain weather immunity while the table's once-per-turn Evasion limit remains in effect. Duplicate ability auras do not stack and respect rank-limited abstract range.

## Arena presets and visible boundaries

Fog, mud, underwater breathing, fire, electric poles, flowers, sewage, jungle, high poles, sprinklers, cemetery, minefield, healing pits and Final Destination have typed records and supported hooks. High poles requires a GM-defined check difficulty; healing pits require a healing amount, consume one action and are depleted. Underwater difficulty increases with rounds exposed, rather than the battle's absolute round number.

Torn World's unspecified retargeting distribution requires explicit GM target adjudication; ordinary rolls cannot silently ignore it. Cemetery's Run Away effect and other narrative escape effects remain adjudicated. Item restrictions do not imply an implemented held-item system. Transformations, item triggers, charge sequencing and unreviewed move effects remain outside this release. Manual overrides and historical field records remain available.

## Evidence and verification

Rules: supplied RULING pp.78 and 88–91; reviewed MASTER move/ability entries in the owned correction registry. Original PDFs, JSONs and habitat CSV are unchanged. Fixtures in `combat_tests/field_tests.rs` cover all eight weather bonuses, illegal moves, replacement/extra typing, sheet isolation, exposure, optional/required events, permissions, terrain healing limits, barriers, move action/Will costs and weather ability contests/persistence.

Automated room tests use separate host/GM/player identities on loopback. They do not establish physical-machine LAN or interactive Windows battle acceptance.

Release validation: 193 Rust tests, 21 frontend tests and 9 catalog checks passed. TypeScript, Clippy with warnings denied and the production build passed. The NSIS installer was built and two isolated native startup profiles passed; their current databases passed integrity checks at schema 4. Interactive native battle workflows, three viewport visual comparisons, physical LAN and installer install/uninstall remain unverified.
