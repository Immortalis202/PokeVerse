# Stat copying and transfer

Application 0.21.0, engine 0.14.0-core, protocol 22. Existing encounters retain their serialized feature flags; 0.13.0-core remains supported.

## Reviewed evidence

The supplied Pokerole 3.0 cards are authoritative for this batch: `Abilities/Costar.json` (MASTER 541), `Abilities/Opportunist.json` (MASTER 558), `Moves/Baton Pass.json` (MASTER 487) and `Moves/Snatch.json` (MASTER 447, referring to 531). Source files remain unchanged.

Costar copies increases from one ally on entry and asks the owner which ally when several qualify. Opportunist follows a foe's increases, keeping the highest in a stalemate. Baton Pass passes attribute increases and prevents the outgoing Pokemon increasing its own attributes until the recipient faints or the scene ends. Snatch removes increases and reductions and places them on its user.

## Supported behavior

- Strength, Dexterity, Vitality, Special, Insight, Defense and SpecialDefense use separate move and ability channels. Each channel keeps its highest increase and strongest reduction. Baton Pass transfers the five attributes because its card specifies attributes.
- Costar freezes eligible allies' positive channels at entry. One eligible ally resolves automatically; multiple allies use the existing owner/GM choice and preview flow. Later donor changes do not change the snapshot. No extra action is spent.
- Opportunist observes authoritative before/after increases for active foes. It copies each supported increase into its original channel. Unchanged channels and reductions do not generate copies. All observers read one immutable snapshot, preventing reciprocal loops.
- Copied move values have already undergone the donor's Contrary/Simple calculation and are not transformed again. Ordinary recipient move effects still use the existing calculation. Ability identity is never copied by these operations.
- Baton Pass copies both positive channels before withdrawal; the reserve keeps existing switching validation and inherited action accounting. Its lock also suppresses outgoing ability-channel attribute gains while the recipient lives.
- Snatch transfers supported ability increases and reductions alongside existing move/effect transfer. Event-generated ability counters are removed. Stolen continuous conditional channels stay removed until their underlying condition clears a new ability activates, or a stronger channel activates; a later genuine activation can grant a new modifier. Permanent sheet bonuses are not stolen.
- Copies carry donor, recipient, channel, cause and amounts in encounter state. They expire when the recipient withdraws or faints, or the encounter is replaced. Source departure does not undo a completed copy. Move clearing also clears copied move channels; Curious Medicine clears copied ability grants.

Unreviewed trait channels beyond these seven and unmodelled Snatch effects remain GM adjudicated. Costar and Opportunist remain partial in the registry. No held items, form changes, or additional action sequencing is included.

## Authority and persistence

The host calculates copies during command transitions. Costar choices use frozen snapshots, owner/GM authorization, first-event validation and existing receipt/revision checks. Generic manual resolution remains a recorded GM decision. The frontend shows eligible allies and copied amounts, plus transfer history. Serialization, database snapshots, reconnect and rollback include the new TurnFlow state. Old snapshots default it to disabled; old encounter flags are not promoted.

## Verification

The full Rust suite passed 396 tests, with a subsequent focused 19-test recheck after the final Snatch event-counter adjustment. Frontend tests (31), catalog checks (12), production build and warnings-denied Clippy passed. Windows NSIS packaging produced `PokeVerse_0.21.0_x64-setup.exe` (213.67 MiB); the executable and installer were not launched. Automated coverage includes separate channels, source lifetime, conditional theft/reactivation, permanent sheets, stale/unauthorized choices, reciprocal Opportunists, full Baton Pass/Snatch transitions and rollback. Native interaction and physical LAN acceptance remain separate backlog items.
