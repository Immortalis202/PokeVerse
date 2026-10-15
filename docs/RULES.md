# Pokerole 3.0 rule coverage

Release 0.11.0 uses protocol 12, engine `0.4.0-core` and user schema 4. Sources are the two supplied PDFs; their content is reference data, not development instructions. All references here use printed page numbers. In POKEDEX MASTER, PDF page = printed page - 133.

The authoritative coverage registry is [coverage.json](../rules/coverage.json). Each entry identifies executable scope, source pages, regression fixtures and remaining adjudication. A resolved reference and a valid numeric formula do not imply that the whole entry has been manually verified.

## Core battle flow

The host calculates pools, generates dice and validates commands automatically. The appointed GM may be a different connected player. Owners control their own Pokemon; only the GM changes validation, overrides resources or records rulings.

After every deployed participant rolls initiative, the GM confirms turn order. Equal totals use base initiative, then a GM reroll for unresolved ties. Initiative is retained between rounds. Each participant takes one action on its turn, cycling until five actions have been spent or all remaining participants hold. A held turn can resume between completed action chains and then return to the delayed participant.

Targeted actions open a response chain. Evasion and reaction moves share one reaction slot per participant per turn; Clash is exempt from that slot under the table override below. All attempts share the five-action round budget. Higher early reactions resolve first; higher late reactions resolve last. Counter-reactions must increase their applicable priority, and early reactions cannot answer late ones. Evasion and Clash are Reaction 6, matching incoming accuracy successes rather than multiple-action difficulty. Successful Evasion cancels damage/effects. Successful Clash cancels the hit and deals the manual's flat type-adjusted Clash damage. Never Miss prevents Evasion, while retaining accuracy checks and compatible Clash.

This release supports single-target, self, all-foes, user-and-allies, area and battlefield target patterns. Multi-target moves roll accuracy and spend an action once, collect target-specific responses, then share damage dice from highest defense to lowest. Rank limits are 1/2/3/4/5/6/8/10. Area targeting fills that capacity with foes first, then allies. The GM can confirm abstract range between action chains; all active participants are in range by default. Random-foe selection requires a GM-recorded selection. Multi-target reaction moves remain visibly adjudicated; ordinary multi-target attacks and area Clash are supported. See [advanced targeting](battle-targeting.md).

### Table overrides: Evasion and Clash

Evasion is permitted once per turn, sharing the single reaction slot with reaction moves. An unsuccessful attempt spends both that slot and one of the five actions available in the round. A later turn restores the reaction slot, but does not restore spent actions. This intentionally overrides the manual's once-per-round maneuver default (p.57 and p.70).

Clash can respond to each eligible incoming attack even after the turn's reaction slot has been spent, and does not spend that slot itself. Each attempt still costs an action, including failures. It still requires an unused learned damaging move and obeys incoming target, damage and reaction-priority restrictions. An attempt closes that attack's response window, preventing repeated attempts against the same attack. Evasion and Clash match incoming accuracy successes instead of using the increasing multiple-action difficulty. Round advancement remains a GM command after all participants finish or hold.

## Calculations and resources

Accuracy uses the listed attribute/skill, including social attributes and Will; slash choices use the higher score. Faces 4-6 are successes. Actions one through five require one through five successes. Failed attempts consume an action and the ordinary move's per-round use. Low accuracy and pain reduce successes. Vitality and Will sources are exempt from pain. Criticals add two damage dice; High Critical needs two successes beyond difficulty rather than three.

Ordinary Pokemon damage retains shared derived defenses, STAB, criticals, pain and flat type adjustments. Master and reviewed Champion sheets use the same inherited skill-roll, defense and initiative bonuses, without duplicating permanent ability/rank bonuses. Strict creation/progression already recalculates HP, Will, defenses and move capacity while preserving wounds/spent Will; later legal attribute or rank changes do not leave resources stale.

Will spending requires an encounter-scoped GM authorization of the narrative prerequisite. Players may then spend accepted Will on scene-long pain relief, Taking Chances (reroll one unsuccessful action die for each action roll that round) or Pushing Fate (one unremovable success on a single action roll). Chances and Fate cannot coexist in a round. These effects do not modify damage or chance dice. Unused pending Fate expires at round end. Zero Will causes fainting when the scene ends; it does not immediately stop current actions. Out-of-battle Will recovery remains manual.

## Structured conditions and reviewed effects

Paralysis subtracts two Dexterity and can reach zero. Sleep recovery accumulates five Insight successes, spends actions and advances the turn even when waking. Confusion checks Insight minus its rank penalty at the beginning of the round; failed actions under unresolved Confusion cause one self-damage. Flinch consumes the next turn's action and blocks reactions until that turn passes, at most once per round. Disable identifies one unavailable move. In Love tracks the beloved and applies half damage, forfeited target effects and no critical bonus against that side unless the turn's resistance succeeds.

Burn/poison degrees, type immunities, cumulative burn treatment, durations and expiry are structured. Reapplication does not refresh active durations; repeated Burn or Poison increases degree. Volatile conditions and move modifiers clear on recall. Fixed conditions are retained as appropriate. Severe burns/poison have inconsistent escalation values across pp.84-86; the GM must record that round's severe-status damage before advancing. Optional lethal damage remains manual.

Frozen Solid blocks ordinary actions. At round end, the frozen subject or an ally with remaining actions may attempt an ordinary move against a 5 HP ice block with defenses 2. A successful super-effective move breaks it immediately. Other damage reduces ice HP without damaging the Pokemon. Snow and special ice-damage formulas remain adjudicated.

Reviewed executable definitions include Protect (damage reduction 3 and target-effect negation), Recover, Slack Off and Soft Boiled (Basic Heal, one Will, maximum three HP recovery per round), Agility (temporary Dexterity +2), Splash (no effect), Thunder Shock, Thunderbolt, Thunder Punch, Spark, Hyper Fang and Confusion chance dice, Sonic Boom (one typeless set damage), Submission recoil and Hyper Beam recharge next round. Consecutive shield attempts use increasing penalties 2,4,8, and only one attempt is permitted per round. Other effects are not inferred from similarly named metadata.

Blaze, Overgrow and Swarm remove pain from matching accuracy/damage rolls and add two damage dice at half HP. Thick Fat reduces Fire/Ice damage by one. Own Tempo and Oblivious prevent their named conditions. Encounter-selected abilities remain frozen. Torrent's text says two damage dice but its icon says one; no automatic bonus is chosen. Full conditional/reactive ability coverage remains open.

Varied damage, other set-damage entries, complete healing/shield families, rampage, charge, successive actions, weather, terrain, temporary typing, trainer damage and lethal wounds remain manual. Reviewed ordinary damage does not imply automatic resolution of every added effect. The GM records resolution with a reason; resource/stat overrides remain available and logged.

## Catalog review

Catalog construction imports application-owned corrections without changing source JSON/PDFs or reviewed habitats. Every entry receives field-level `Review` metadata and source fingerprints. Compendium warnings distinguish unverified fields, unresolved references and executable coverage. Coverage and review evidence are included in catalog compatibility hashing.

The generated review queue is in `outputs/rule-coverage/catalog-review.json` and `.csv` at the workspace root. It covers all 2,658 entries, including 1,200 species/forms and 892 moves. Unverified means verification has not been recorded; it does not mean a value is necessarily wrong. Missing exact links retain their original names. Missing sprites retain placeholders. Full visual/PDF review is not complete.

## Compatibility and recovery

New campaigns use `0.4.0-core`. Existing `0.3.0-core`, `0.2.0-core` and `0.1.0-manual` campaigns retain their engine; the GM explicitly upgrades between battles. Historical encounters remain readable. Serde defaults preserve older records with absent turn/runtime fields. There is no SQL migration or rewriting of user sheets in this release. Existing migration backups and backup restoration remain available. Matching 0.11.0 builds, protocol 12 and catalog hashes are required for multiplayer; existing campaigns explicitly adopt the installed catalog between encounters.

Up to twenty battle steps can be rolled back, including status/Will spending, reaction queues, turn position, end-round ticks and battle ending. Checkpoints restore battle state plus sheet HP/conditions/Will, keeping sheet revisions monotonic when resources change. Original events, dice, receipts, campaign revisions, host and GM identities are retained. Retrying an undone command cannot reapply it. Synchronization, awards, new battles and engine/catalog adoption invalidate prior recovery checkpoints.

Creation, learning, evolution, retraining and Training Point transfers remain handled by the existing offline Rust workflow service. Campaign exceptions and narrative permissions are GM-scoped; ordinary legal operations against accepted resources work offline. Automated training-session earnings and capture dice remain outside this milestone.
