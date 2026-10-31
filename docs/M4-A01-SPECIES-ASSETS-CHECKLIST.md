# M4-A01: exact species and form images

Completed 31 October 2026. All **168 baseline missing assets** have exact application-owned image mappings. The generated catalog audit reports **zero missing assets**. M4-S01 and the wider M4 milestone remain open.

## Sources and identity review

- [x] Preserve all 1,200 catalog IDs and the supplied `Pokerole-Data` files. Only the 168 baseline image gaps receive supplemental mappings.
- [x] Retrieve images linked by the matching Serebii species/form reference pages: **167 official game renders and one official Rotom Dex artwork**. Keep the original downloaded PNG bytes. No AI-generated art, recoloring or substitution with another form.
- [x] Decode every PNG and visually inspect all 168 in labelled contact sheets. Match form captions/diagrams, including regional variants, gender variants, riders, masks, Terapagos, Tatsugiri, Poltchageist/Sinistcha, Palafin, Cramorant, Eternamax and Bloodmoon Ursaluna.
- [x] Record each exact ID, name, original image filename, application filename, source-page URL, source image URL, retrieved page hash, PNG SHA-256, byte count, dimensions and identity note in [the asset registry](../rules/species-asset-review.json). The HTML hashes identify the retrieved reference versions; live pages may subsequently change.
- [x] Bundle the files in [the tracked asset directory](../assets/species). Catalog generation verifies their hashes and copies them into `public/sprites` before auditing. Generation and the resulting app need no network access for these images.
- [x] Expose the reference link on the species reference card and retain image provenance in full catalog records. Original sprites for other species remain supplied by the existing dataset.

Representative reference pages: [regional/gender/form examples](https://www.serebii.net/pokedex-sv/basculegion/), [Terapagos forms](https://www.serebii.net/pokedex-sv/terapagos/), [Bloodmoon Ursaluna](https://www.serebii.net/pokedex-sv/ursaluna/), [Ash-Greninja](https://www.serebii.net/pokedex-sm/658.shtml), [Eternamax](https://www.serebii.net/pokedex-swsh/eternatus/) and [Rotom Pokédex artwork](https://www.serebii.net/pokemon/rotom/). Every individual reference is in the registry. Serebii is the reference host, not the owner of Pokémon imagery.

## User decisions

- Unsuffixed Maushold uses Family of Four; Dudunsparce uses Two-Segment; Squawkabilly uses Green Plumage.
- Greninja (Battle Bond Form) uses Ash-Greninja.
- Exact official artwork is permitted when no suitable game render is available. Rotom Dex uses the Pokédex artwork, not Rotom Phone or ordinary Rotom.
- Catalog Basculin (Hisuian Form) maps to the referenced White-Striped form. Kitakami Ursaluna maps to Bloodmoon. These image mappings do not change species names, evolution rules, learning costs or battle support.

## Credits

Pokémon imagery © The Pokémon Company / Nintendo / Creatures / GAME FREAK. Reference hosting: Serebii.net. Images remain the property of their respective rights holders. Asset bytes and source links are preserved; no ownership is claimed.

## Validation

- [x] Exact coverage of all 168 baseline IDs; unique application filenames; no external image URLs in the runtime `Image` fields.
- [x] Bundled and generated image bytes match the recorded hashes and dimensions.
- [x] Fresh-output fixture copies both supplemental and supplied sprites before audit. Corrupted files, changed identities and unsafe supplemental paths are rejected.
- [x] Named form pairs have distinct reviewed source images and byte hashes.
- [x] **794 Rust tests passed, 1 diagnostic ignored; 71 frontend tests and 45 script/catalog checks passed. TypeScript, relevant Prettier, Rust formatting and diff checks passed.** All 168 PNGs decode successfully and have nonempty content; total supplemental PNG size is 6,394,544 bytes (about 6.10 MiB).

Native app launch, installers and physical-network acceptance are outside this asset batch. No native packages or Clippy are requested.
