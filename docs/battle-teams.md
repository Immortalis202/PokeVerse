# Registered battle teams (0.7.0)

The GM can register a whole Pokémon roster and deploy 1–6 Pokémon per side, defaulting to three active slots. The host automatically validates every command. Hosting does not confer GM permission when a different player is appointed.

For the gym scenario, synchronize three player trainers with two Pokémon each and a GM-owned gym trainer with six Pokémon. In Battle, register all twelve, assign the player Pokémon to Players and the gym Pokémon to Opponents, and mark one starter per player trainer plus three gym starters. Start the encounter with three slots per side. Only those six are active; the other six are reserves in the same encounter. The GM records team choices in this version; players do not submit separate team proposals.

## Switching and replacement

An owner can recall their active Pokémon and send out a conscious reserve belonging to the same trainer and side. The GM can manage either team's switches. A fainted active Pokémon can be replaced through the same controls. Each switch transfers its slot; it cannot add a fourth active Pokémon to a three-slot side. The roster and trainer ownership remain fixed until the encounter ends.

This implements the Trainer Area rules from corebook pp.64–65: the first two recall/send-out operations per trainer per round are free. A paired switch uses two operations, so the first switch is free, the second costs two trainer actions and the third costs another two. The tracker resets on Next round and caps paid switching actions at five. Other trainer actions and Trainer-in-the-Fray costs remain manual.

Incoming Pokémon can be targeted immediately but cannot act or roll initiative until the next round. HP, conditions, used moves, initiative and action/reaction counters remain attached to each individual through recall and redeployment; ordinary round advancement resets round counters. Returning Pokémon retain their initiative and can reroll it through the existing controls. Resolve or cancel pending damage before switching. Recalling on the correct Pokémon turn, end-of-round timing, trapping and switch-triggered abilities remain GM-adjudicated.

The GM can explicitly waive switching cost and the incoming Pokémon's waiting period, with a required logged reason. Normal structural and ownership checks still apply. Switching and overrides use the existing battle rollback system. Rollback restores deployments, resource state and trainer counters while preserving command receipts.

## Persistence and compatibility

Active slots, reserves, waiting rounds and per-trainer switching counters are stored in the host campaign snapshot, broadcast to clients and restored on reconnect. Retrying a command does not duplicate a switch or consume more actions. Ending the battle persists HP and conditions for the entire roster. Synchronization cannot replace a registered Pokémon sheet mid-battle, including a reserve.

Version 0.7.0 uses protocol 9; all clients need matching builds and catalogs. Library/campaign schema 4 creates consistent pre-team migration copies before upgrading. Existing battles load with every historical combatant active and no registered-slot configuration; finish them before starting a registered-team encounter. Historical records and character identities are preserved. The original unrestricted combatant mode remains available by clearing Active slots and reserves during preparation.

## Verification

Regression checks cover a twelve-Pokémon roster with six deployed, reserve action/target rejection, same-trainer ownership, waiting rounds, trainer switching costs, fainted replacements, pending-attack locks, resource preservation, GM overrides, rollback, migration idempotency and legacy defaults. A Windows host with three remote clients exercises a separate GM, owner commands, duplicate receipts, reconnect and persisted state. Frontend rendering checks verify starter controls, reserve filtering and separate host/GM permissions. Full interactive GUI play through multiple installations remains a manual acceptance check.
