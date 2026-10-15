# Training encounters (0.6.0)

Use **Room** to connect to a campaign, then open **Training encounters**. Hosting and the GM role are independent: the appointed GM prepares and resolves encounters; the host validates and persists commands automatically.

1. Choose a campaign trainer, habitat, rank and size (1–50; default five at the trainer's rank). Generate a mixed horde. Each draw is equally weighted, and repeated species are separate individuals.
2. Change individual ranks or ordinary abilities, replace species from the habitat pool, reroll, add or remove individuals. **Save privately** preserves the roster on this installation. It does not publish anything.
3. **Reveal to room** publishes the roster. Edits afterward remain private until **Publish roster update**. Players see species, sprites, ranks and habitat conditions; capture previews remain on the GM client.
4. Choose a recipient trainer for each captured individual. Rust generates a balanced sheet and eligible initial moves. **Adjust captured sheet** opens the creation editor for allocations, socials, skills and moves. Species, rank and ability remain tied to the roster; change those in the roster and publish an update first.
5. Enter manual Training Point awards for existing participating campaign Pokémon. Zero means no award. Record a GM reason, **Review outcome**, then **Confirm captures and rewards**. The review displays Rust validation, catalog adjudications and resource changes.
6. Victory is assumed. Captures, rewards and history are committed together. The owner's **Import my captured Pokémon** action imports the same IDs and trainer links through normal synchronization. Players import accepted trainer pools, distribute points to their Pok�mon in Library, and can then progress offline.

No battle is opened. Capture dice, balls, combat, automatic reward formulas and automated training sessions are outside this feature. Higher-rank captures are permitted; the rules preview displays disobedience warnings. A GM-confirmed manual award uses the entered amount; the application does not apply an automatic reward formula.

## Offline and reconnect behavior

Room's **Offline preparation** selector opens a locally stored campaign or the last received room snapshot without contacting the host. This allows a remote GM to reopen saved private rosters after restarting. The snapshot can be stale; publication and resolution always require reconnecting and host validation. An installation must have joined that campaign at least once to have its snapshot.

Published preparation is stored on the host and available to a replacement GM. Capture selections, allocation edits and reward review are transient until confirmation; saved private preparation preserves the roster. Reopening a published encounter allows fresh capture previews. Resolved and cancelled encounters cannot be overwritten or resolved again.

## Catalog and storage

The build reads `../outputs/habitat-review/habitat-list.csv` and `pokemon-habitats-review.csv`. Only accepted `habitat_ids` enter pools. Review decisions, conditional notes, pending assignments and exclusions remain in SQLite provenance. Contradictory accepted rows fail the build. No historical type suggestions or rarity weights are applied, and source CSVs are not rewritten.

Catalog compatibility includes habitat data. Existing campaigns may open a room with a changed installed catalog, but shared changes require the appointed GM's **Adopt installed habitat catalog** step. Finish old battles, cancel incompatible published preparations and reject stale proposals first. Prior records remain unchanged. All clients require application 0.7.0, protocol 9 and the same catalog.

Encounter storage introduced schema 3; registered battle teams advance both databases to schema 4. Before these changes, consistent `*-pre-encounters-*.db` and `*-pre-teams-*.db` copies are created. Restoration accepts same-identity schema 2–4 backups; older backups are migrated after restoration. Private rosters and remote campaign caches live in `library.db`; accepted outcomes and command receipts live in `campaigns.db`.

## Validation

Automated checks cover exact CSV/SQLite equivalence, CSV quoting and contradictory decisions, deterministic generation, size limits, separate identities, all eight ranks, permanent ability modifiers, higher-rank warnings, invalid edits, mixed capture recipients, GM replacement, stale previews, restart, backups, atomic failures and duplicate commands. A Windows Rust integration test runs a real host and separate GM client through adoption, publication, capture/reward resolution, reconnect, repeated imports and offline progression.

Native GUI acceptance checklist for two isolated installations: appoint the other client GM; save and reopen private preparation while disconnected; reveal and explicitly update the roster; select mixed capture recipients; edit allocations and moves; inspect final rewards/adjudications; confirm; reconnect; import twice; progress a capture offline. Verify no battle was opened and no unpublished roster or sheet preview appeared on the player installation.
