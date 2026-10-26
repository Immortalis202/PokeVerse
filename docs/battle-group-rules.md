# Selected group battle rules

User-selected on **26 October 2026**, including the clarification that Magic Coat lasts until its user next acts. Implemented in application **0.31.0**, engine **0.24.0-core**, protocol **32**, user schema **4**. These are the group's explicit interpretations. Supplied Pokerole JSON/PDF descriptions remain unchanged; `rules/group-rules.json` records separate provenance and a coverage fingerprint.

| Rule | Behavior |
| --- | --- |
| Torrent | Matches Blaze, Overgrow and Swarm: matching-type accuracy/damage ignore pain; two extra Water damage dice at half HP or below. No bonus above half HP, and shared damage preserves its original saved inputs. |
| Rivalry | When targeting a foe, same gender gives +1 Strength and opposite gender -1, subject to the existing stat/channel rules. Unknown/genderless cases are neutral. Individual gender is recorded explicitly, never inferred from a name/species ratio. |
| Full revival | Max Revive and Revival Herb restore full eligible HP without the shared three-HP round cap, including through Recycle. Ordinary Revive is still one HP. Fainting eligibility, healing blocks/debt and herbal acceptance/Gluttony remain enforced. |
| Ring Target | Removes the holder's supported type and ability immunities, including Wonder Guard and other unique immunity cases, while active. It does not erase every ability effect: retaliation and unrelated boosts remain. Other items and battlefield protection remain; Klutz/Magic Room can suppress Ring Target. |
| Choice Scarf | The selected learned move gains Reaction 5 and overrides Late Reaction. This includes Late Reaction granted by Mycelium Might. Other Choice effects remain unchanged. |
| Mycelium Might | Every resolved Support move gains Never Miss and Late Reaction 1, overriding printed early reaction. It does not grant ability bypass. A selected Choice Scarf takes timing precedence when both apply. |
| Magic Coat | Reflects supported incoming effects until its user's next action begins, including a reaction/free move attempt. Enemy actions do not consume it; round/turn changes alone do not expire it. Recall/fainting ends it. It is not consumed by the first reflection. |
| Bide | Keeps the existing two-attack preparation and doubles the damage received by its user. Release targets the last Pokémon to attack the user; a later missed damaging attempt updates that identity without adding damage. If that last attacker is no longer eligible, it does not silently choose a substitute. |

## Compatibility and controls

New battles in engine 0.24.0-core snapshot `TurnFlow.house_rules = true`. Missing flags default false. Historical engines, active encounters, archives and rollback snapshots retain their previous decisions. Existing campaigns adopt current rules **between battles** before starting a group-rule encounter. All clients need matching v0.31.0/protocol 32 builds and catalog fingerprints.

The battle UI displays the selected rules and removes obsolete Torrent/Rivalry, full-revival/Ring Target/Scarf, Bide-basis and Magic Coat-duration prompts. Actual move choices, gender/weight profiles, accessible supplies and herbal acceptance remain explicit inputs. The host rejects conflicting fixed-policy commands. Generic manual effect adjudication remains available for unsupported definitions.

## Verification

Focused regressions cover all eight policies, historical flag defaults, shared bonus inputs, unique/type/status immunities, retained contact retaliation, ordinary item protection, full revival/Recycle, Scarf/Mycelium/Prankster ordering, repeated reflections and next-action expiration with rollback, Bide's real incoming hits/miss and forced target, and host/GM/owner receipts/reconnect/persistence/rollback. Catalog checks verify the separate user provenance and compatibility fingerprint. Windows and Linux validation: **662 Rust tests on each platform**, **60 frontend tests**, and **21 catalog/script checks** passed. The final Magic Coat queue-renewal regression also passed with the 11 focused group-rule tests. TypeScript, Rust formatting, and Clippy checks passed. Matching Windows NSIS and Linux Debian/AppImage production builds are supplied for private testing.

Native UI, installation and physical Windows/Linux networking acceptance remain separate manual checks. Builds are for private group testing; public release/signing work is not planned.

## Mirror Coat correction — v0.31.1

The supplied v3.0 Mirror Coat card is distinct from Magic Coat. Mirror Coat is an immediate **Late Reaction 5** response to a Special move attacking its user. It spends one action and the shared Evasion/reaction slot, even if its accuracy roll fails. The incoming attack resolves first; Mirror Coat then targets that attacker and rolls the exact incoming damage pool plus two dice, ignoring defenses. It cannot be activated as an ordinary move, stored until another action, used for an ally, or reuse an unrelated old pool. Ordinary reaction priority, accuracy, action limits and damage immunities still apply. Magic Coat's previously selected rule is unchanged.

New optional action/history sequence fields default safely when loading older snapshots. This correction applies to current group-rule encounters; encounters without that flag retain historical behavior. Version 0.31.1 has a new catalog compatibility fingerprint; use matching builds for rooms.

Validation for v0.31.1: **665 Rust tests passed on each of Windows and Linux** (Linux release mode), **60 frontend tests** and **21 script/catalog checks** passed; TypeScript, Clippy and formatting checks passed. Three new Mirror Coat regressions cover exact incoming pool/timing and serialization, rejected physical/ally/ordinary/spent-slot uses, failed accuracy, and rollback. Windows NSIS and Linux Debian/AppImage production builds passed. GUI and physical Tailscale acceptance remain manual. Upgrade both clients between battles.
