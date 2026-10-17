# Targeting, reactions and controlled replacements — 0.16.0

Engine `0.9.0-core`; protocol 17; user schema 4. New state uses serialized defaults. Existing campaigns adopt rules/catalog between battles; historical battles keep their recorded engine/flags. Original JSON and PDFs are unchanged. `rules/tactics-reviewed.json` records printed card pages, executable scopes and dependencies; catalog compatibility includes its fingerprint.

## Response queue

The response window belongs to the next pending action. Early reactions resolve from higher to lower priority, the main action follows, then late reactions resolve from lower to higher priority (RULING pp.58–59). Multi-target reactions roll accuracy once, spend one action and use per-target responses with shared damage dice. Field reactions keep a field scope. Resolving a reaction resumes its suspended parent without advancing the turn.

Evasion remains once per turn, sharing its slot with reaction moves. Clash may answer every eligible incoming hit within the five-action budget, using eligible moves. The reaction slot is not refreshed by a counterattack, another target or a successive hit. Bystanders may use a reaction move during an open response window; this does not spend the original target's response. Maneuvers still require the maneuver's actual target.

When several participants Clash one attack, only the highest counter-damage hits its source (RULING p.71). Individual defenders take their own Clash damage. The engine records the pending maximum, applies reductions/Sturdy once after responses, then runs the normal ability/faint hooks. Reconnect/rollback retain that pending resolution.

Use **Multiple targets and battlefield moves → Use selected targets for a reaction move** to select the acting Pokémon and affected targets. The backend checks timing, learned moves, target patterns, range, rank caps, action budget and blockers; the UI does not decide legality.

## Redirection and support interception

- Lightning Rod and Storm Drain redirect eligible single-target Electric/Water moves to a living active holder, prevent their typed damage and grant their first-hit Special +1. Multi-target patterns are not collapsed into a single holder.
- Follow Me, Rage Powder and Spotlight retain their reviewed round scopes. Rage Powder only attracts Physical/Special moves and respects Grass-type/Overcoat powder immunity (RULING p.80). Successful early lures can redirect an unresolved parent before damage inputs commit.
- Stalwart and Propeller Tail bypass move/ability redirection, including lures, attractors and support reflection. Cover is a separate maneuver; existing Sound/Infiltrator exceptions remain distinct.
- Magic Bounce and Magic Coat reflect executable foe Support effects aimed at the holder or its side, once. The original user pays any reviewed Will cost; the reflector becomes the effect source. Reflected effects do not loop. Unsupported effects remain adjudicated; manual field records retain the reflected side.
- Good As Gold blocks incoming ally/foe Support effects. Self effects and side-wide field effects are distinct.

Multiple eligible redirectors do not silently use a video-game precedence rule. The GM records the chosen redirector in **Targeting and replacements**. Eligibility is checked again on each use. Committed damage snapshots cannot be changed by these rulings.

Corrections from visual card review include Follow Me's Reaction 5 (the JSON effect text said 4) and Magic Coat's Self target (the JSON target said the foe's field). Source files are unchanged.

Magic Coat states no duration. Before activating it, the GM records **next reflected effect**, **current round**, or **encounter until recall**. This choice is logged and preserved; the engine does not supply a video-game duration.

## Priority abilities

Prankster grants Reaction 1 to Support moves; Gale Wings grants it to Flying moves without a video-game full-HP condition. Triage extends reviewed Basic Heal and status-cure support reactions to selected allies and groups. Higher printed early priority remains applicable. An ability's Reaction 1 conflicting with printed Late Reaction requires a recorded printed/ability timing ruling. Unreviewed effects and charge reaction exceptions remain explicit.

Dazzling/Queenly Majesty prevent foe reaction moves against the holder, while Evasion/Clash remain maneuvers. Armor Tail protects the holder and in-range allies against Physical/Special reaction-tagged moves; it does not block Support moves or maneuvers. Stall remains last, including Trick Room changes, without replaying completed turns.

## Switcher effects

Reviewed switchers: Roar, Whirlwind, Dragon Tail, Circle Throw, U-Turn, Volt Switch, Teleport, Parting Shot, Ally Switch and supported Baton Pass channels. Suction Cups blocks foe-forced switches. Emergency Exit/Wimp Out queue a replacement when HP crosses to half or below, ignoring Block, using their switcher icon.

Only a conscious registered reserve of the same trainer, owner and side is eligible. The owner or appointed GM chooses it; the host validates and commits automatically. The incoming participant inherits the field slot, initiative and remaining action budget (RULING p.83). These effects are separate from ordinary Trainer Area switching and do not charge its action costs. One completed switcher effect is allowed per affected trainer per round; deviations use logged GM overrides/waivers.

Ordinary queued replacements wait for the attack chain to finish, preserving shared damage snapshots. A reaction switcher replaces its user before the suspended incoming action proceeds. Ally Switch also refunds its action and gives the new ally a response. Already resolved targets are never hit again because of replacement.

No reserve never silently ends the battle: the GM records withdrawal/waiver or explicitly ends it. Pending replacements block unrelated actions and round advancement. Owners cannot waive a forced effect, choose another trainer's Pokémon, or resolve another owner's request. Receipts/revisions prevent duplicate replacements and stale submissions.

Baton Pass transfers supported positive move attribute channels and locks the original user's self-boosts until the recipient faints or the encounter ends. Ability-channel transfer remains unimplemented and explicitly partial. Shed Tail was manual in 0.16.0; [0.17.0 decoys](battle-decoys.md) now implement its creation and reserve handoff. Narrated wild fleeing, unimplemented Support effects and other unreviewed move interactions remain GM adjudication.

## Compatibility and validation

Fifteen previously manual ability definitions now have reviewed automated or partial scopes; 224 of 305 abilities have supported scopes overall. This does not imply complete ability or move coverage. Trainer-target damage, lethal wounds and automatic Will recovery remain excluded.

Release checks (17 October 2026): 285 Rust tests passed, including a real separate host/GM/owner replacement, duplicate receipts, stale requests and reconnects. All 24 frontend tests and 11 catalog/preview checks passed. TypeScript, Clippy with code warnings denied, production Vite build and Git whitespace checks passed. Windows used its incremental-cache copy fallback where hard links were unavailable.

Legacy sheets missing trainer links queue an explicit GM replacement ruling instead of rolling back valid damage. Historical-engine fixtures keep the old reaction path and reject new commands.

Native startup checks do not establish interactive battle, installer upgrade/uninstall or physical multi-machine LAN acceptance. The Windows NSIS installer built successfully. Two hidden isolated profiles started with application 0.16.0 / engine 0.9.0-core; both library and campaign databases passed integrity checks and retained schema 4.
