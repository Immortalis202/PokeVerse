# Advanced battle targeting — 0.11.0

Engine `0.4.0-core`, protocol 12, schema 4. Rules reviewed against the supplied RULING pages 68–72. The original PDFs and source JSON remain unchanged.

## Using the flow

1. Upgrade an existing campaign's engine and adopt the installed catalog between battles. All clients must run the matching build.
2. Open **Multiple targets and battlefield moves**, select the acting Pokémon and a learned move. Select targets for all-foes or user-and-allies moves. The latter must include the user.
3. Area moves automatically select up to the rank cap from the confirmed range, with foes before allies. The GM may change range between complete action chains, with a reason. Without a range ruling, all active, conscious, available Pokémon are in range. No distance or map position is inferred. The user is excluded from their own area attack.
4. Preview to see legality, accuracy dice and affected names. Commit rolls accuracy once and spends one action, including failures. Caps by rank: Starter 1, Rookie 2, Standard 3, Advanced 4, Expert 5, Ace 6, Master 8, Champion 10.
5. Each target reacts or passes. Evasion protects that individual. Clash requires an unused learned damaging move; an area attack requires an area move to Clash. A successful Area Clash deflects the whole incoming area attack, with type-adjusted Clash damage applied to the two participants. This scope is an explicit engine interpretation of the area-Clash rule.
6. Resolve damage from highest relevant defense to lowest. Earlier dice are retained; the host rolls only additional dice needed by the next pool. Type immunity, resistance, weakness, shields and supported abilities remain target-specific. GM damage modifiers are fixed once the shared roll starts. Added effects retain their reviewed/manual handling for each target.

## Battlefield effects

Whole-field, allied-side and opposing-side moves have a field scope rather than a Pokémon target. They cannot be Evaded or Clashed. After accuracy, the GM records the mechanical ruling and a duration (or encounter-long). Effects appear in the shared battle state, expire after the recorded rounds, and can be removed explicitly. Their records survive reconnect and rollback.

Recording Reflect, Rain Dance or a terrain move does **not** automatically apply weather, barriers or terrain stat/damage effects. Those definitions remain a separate milestone. The GM uses logged modifiers/status controls as required.

## Cover an Ally

An owner or GM may attempt **Cover an Ally** while an ally is targeted, before a Reaction 1 or higher chain blocks it. It uses Will, requires reviewed Loyalty above 2, spends an action and the ordinary reaction slot, and may be used once per round. It cannot exceed five actions, and failure still spends the attempt. Will accuracy ignores pain.

Success redirects the attack to the covering Pokémon and records ongoing cover. Holding preserves it. Acting by either participant, fainting or switching out removes it. When both coverer and ally occur in one area target list, redirection does not hit the coverer twice. Cover does not create chains of recursive redirection.

## Compatibility and limits

Defaulted JSON fields preserve older battles, and engine `0.3.0-core` remains usable for existing campaigns. Upgrading requires no active battle and never rewrites finished battles. Protocol 12 and a new catalog compatibility fingerprint prevent old clients joining the updated room. No SQL schema change is required; normal migration backups remain in place.

Multi-target **reaction moves**, random-target selection without GM confirmation, spatial movement, weather/terrain mechanics and unreviewed added effects remain explicit adjudication. Human targets and lethal damage remain outside this milestone.

Tests cover rank caps, user inclusion, illegal/duplicate/out-of-range targets, area friendly fire, Evasion isolation, area Clash, shared damage prefixes, field scope/expiry, Cover prerequisites/costs, serialization, rollback, separate host/GM/player permissions, receipts and reconnect.
