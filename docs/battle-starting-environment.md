# Starting multiplayer battlefield

Application **0.32.0**, engine **0.25.0-core**, protocol **33**, user database schema **4**.

The GM chooses the environment in **Battle control**, before starting a multiplayer encounter, for ordinary combatants or registered teams. This is encounter-specific setup. Wild Pokémon/training encounter generation, habitat choices, encounters and rewards are unchanged; no global weather default is applied there.

| Choice | Supported starting conditions |
| --- | --- |
| Time | Day or night, independent of weather and sunlight access |
| Natural weather | Clear, Sun, Rain, Snow, Hail, Sandstorm, Harsh Sun/Desolate, Heavy Rain/Typhoon, Strong Winds |
| Arena access | Indoors and whether sunlight can reach the arena, with existing weather activation constraints |
| Terrain | None or one of Grassy, Electric, Misty or Psychic Terrain |
| Rooms and effects | Trick Room, Wonder Room, Magic Room, Gravity, Reflect, Light Screen, Aurora Veil |
| Reviewed arena presets | Fog, mud, underwater, fire, electric poles, flowers, sewage, jungle, high poles, sprinklers, cemetery, minefield, healing pits, Torn World and Final Destination |

Terrain and each extra effect have their own remaining-round counter. Zero in the form means until removed; positive counters are 1–1000 remaining rounds. Room/screen effects start with four remaining rounds in the form and can be changed. Arena presets default to until removed. High poles retain an explicit check difficulty; healing pits retain an explicit healing amount. The host rejects invalid names, conflicting terrain types, duplicate effects, invalid parameters and durations. Torn World's random targeting remains an explicit GM ruling; this setup does not invent missing targeting mechanics.

Starting conditions are part of the GM's start-battle command. The host validates and creates the battle atomically, without spending actions or Will. A failed start leaves the campaign untouched. The saved `Battlefield.initial` snapshot records the GM's original choices; the live field remains separate and can change through normal moves, abilities and idle GM field controls. Players can read conditions but cannot submit the setup. No setup form is shown to the host merely because they host; GM authority is required.

Setup is installed before entry abilities resolve. Weather abilities still respect suppression, ordinary/extreme weather rules and source contests. A Surge meeting a different preexisting terrain retains the existing GM replacement decision. Natural weather remains the fallback under temporary weather. Conditions do not expire merely because initiative is being rolled; positive effect counters decrease once at the normal round transition. Trick Room affects the first initiative order, Gravity affects grounding immediately, and Magic/Wonder Room use existing item/defense channels.

Moonlight in a prepared battle reads the **live** battlefield day/night. The GM can choose “use battlefield”, “treat as day” or “treat as night” for an individual Pokémon in encounter parameters, with a recorded reason. Existing positive legacy night rulings remain deliberate overrides. Historical/unprepared encounters retain their saved individual-night behavior. Moonlight otherwise retains its existing weather, Will and healing rules; no change is made to Morning Sun or Synthesis.

Both start commands have an optional setup field; omitted fields load as absent. Missing initial snapshots default to absent, so active, archived and historical encounters are not rewritten. Upgrade the campaign between battles to use the new setup and use matching v0.32.0/protocol 33 clients. The current engine also retains all previous group-rule and item/ability feature gates. The setup form starts fresh for a new battle instead of silently copying a previous arena.

## Verification

Regressions cover every supported weather, atomic validation and GM authorization, current/historical engines, first-turn Trick Room order, Gravity grounding, independent duration expiry, entry weather/terrain conflicts, Moonlight night/override/history behavior, serialization and rollback. A real localhost room with separate host, GM and three trainer owners covers start permission, duplicate receipts, reconnect, privacy-filtered snapshots and SQLite persistence. Wild encounter state is checked for changes; the full encounter workflow suite remains in the full regression run. UI checks cover all choices, independent durations, disabled controls, and GM-only/current-engine/pre-battle visibility.

**672 Rust tests passed on Windows and Linux** (Linux release mode), together with **63 frontend tests** and **21 catalog/script checks**. TypeScript, Clippy with warnings denied, Rust formatting and diff checks passed. Windows NSIS and Linux Debian/AppImage production builds passed; extracted Linux catalogs match the Windows catalog. Native GUI, installation and physical Windows/Linux Tailscale testing remain separate manual acceptance checks.
