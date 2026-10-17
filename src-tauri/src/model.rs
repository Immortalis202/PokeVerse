use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

pub const ENGINE: &str = "0.11.0-core";
pub const DECOY_ENGINE: &str = "0.10.0-core";
pub const TACTICS_ENGINE: &str = "0.9.0-core";
pub const SPECIAL_ENGINE: &str = "0.8.0-core";
pub const EXTRA_ENGINE: &str = "0.7.0-core";
pub const FIELD_ENGINE: &str = "0.6.0-core";
pub const ABILITY_ENGINE: &str = "0.5.0-core";
pub const TARGET_ENGINE: &str = "0.4.0-core";
pub const TURN_ENGINE: &str = "0.3.0-core";
pub const PREVIOUS_ENGINE: &str = "0.2.0-core";
pub fn core_engine(engine: &str) -> bool {
    [
        ENGINE,
        DECOY_ENGINE,
        TACTICS_ENGINE,
        SPECIAL_ENGINE,
        EXTRA_ENGINE,
        FIELD_ENGINE,
        ABILITY_ENGINE,
        TARGET_ENGINE,
        TURN_ENGINE,
        PREVIOUS_ENGINE,
    ]
    .contains(&engine)
}
pub const LEGACY_ENGINE: &str = "0.1.0-manual";
pub const PROTOCOL: u32 = 19;
pub fn enabled() -> bool {
    true
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub conflicts: Vec<String>,
}
impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        crate::diagnostics::record(code);
        Self {
            code: code.into(),
            message: message.into(),
            conflicts: vec![],
        }
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("storage", e.to_string())
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::new("io", e.to_string())
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("format", e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Sheet {
    pub id: String,
    pub owner_id: String,
    pub revision: u32,
    pub kind: String,
    pub name: String,
    pub trainer_id: Option<String>,
    pub species_id: Option<String>,
    pub rank: String,
    pub nature: String,
    pub ability: String,
    pub stats: BTreeMap<String, i32>,
    pub skills: BTreeMap<String, i32>,
    pub moves: Vec<String>,
    pub hp: i32,
    pub max_hp: i32,
    pub xp: i32,
    pub inventory: BTreeMap<String, i32>,
    pub notes: String,
    pub archived: bool,
    pub squad_order: i32,
    pub conditions: Vec<String>,
    #[serde(default)]
    pub workflow: Option<CharacterBuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct RulesProfile {
    pub version: u32,
    pub defenses: String,
    pub hp: String,
}
impl Default for RulesProfile {
    fn default() -> Self {
        Self {
            version: 1,
            defenses: "split".into(),
            hp: "vitality".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CharacterBuild {
    pub version: u32,
    pub status: String,
    pub origin: String,
    pub age: String,
    pub allocations: BTreeMap<String, i32>,
    pub socials: BTreeMap<String, i32>,
    pub happiness: i32,
    pub loyalty: i32,
    pub training_points: i32,
    pub legacy_xp: i32,
    pub will: i32,
    pub max_will: i32,
    pub profile: RulesProfile,
    pub hp_attribute: String,
    pub available_abilities: Vec<String>,
    pub learned: BTreeMap<String, String>,
    pub forgotten: BTreeMap<String, String>,
    pub overranked: Option<String>,
    pub evolution_ready: Option<String>,
    pub ledger: Vec<WorkflowRecord>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WorkflowRecord {
    pub id: String,
    pub before_fingerprint: String,
    pub input_json: String,
    pub cost: i32,
    pub trainer_fingerprint: Option<String>,
    pub campaign_id: Option<String>,
    pub approved_by: Option<String>,
    #[serde(default)]
    pub adjudications: Vec<String>,
    #[serde(default)]
    pub approval_reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WorkflowRequest {
    pub operation_id: String,
    pub expected_revision: u32,
    pub trainer_revision: Option<u32>,
    pub rules_version: u32,
    pub catalog_hash: String,
    pub kind: String,
    pub sheet: Sheet,
    pub move_name: String,
    pub replacement: String,
    pub method: String,
    pub target_species: String,
    pub amount: i32,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WorkflowPreview {
    pub sheet: Sheet,
    pub errors: Vec<String>,
    pub approvals: Vec<String>,
    pub warnings: Vec<String>,
    pub cost: i32,
    pub remaining: BTreeMap<String, i32>,
    pub baselines: BTreeMap<String, i32>,
    pub modifiers: BTreeMap<String, i32>,
    pub derived: BTreeMap<String, i32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WorkflowProposal {
    pub id: String,
    pub actor_id: String,
    pub request: WorkflowRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Progression {
    pub id: String,
    pub sheet_id: String,
    pub revision: u32,
    pub description: String,
    pub xp_cost: i32,
    pub before: Sheet,
    pub after: Sheet,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Library {
    pub player_id: String,
    pub sheets: Vec<Sheet>,
    pub history: Vec<Progression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Combatant {
    #[serde(default)]
    pub runtime: crate::combat::CombatState,
    #[serde(default = "enabled")]
    pub deployed: bool,
    #[serde(default)]
    pub field_slot: Option<u32>,
    #[serde(default)]
    pub available_round: u32,
    pub id: String,
    pub sheet_id: String,
    pub owner_id: String,
    pub name: String,
    pub side: String,
    pub hp: i32,
    pub max_hp: i32,
    pub conditions: Vec<String>,
    pub actions: u32,
    pub reactions: u32,
    pub initiative: i32,
    #[serde(default)]
    pub used_moves: Vec<String>,
    #[serde(default)]
    pub initiative_rolled: bool,
    #[serde(default)]
    pub stat_overrides: BTreeMap<String, i32>,
    #[serde(default)]
    pub skill_overrides: BTreeMap<String, i32>,
    #[serde(default)]
    pub pending_damage: Option<PendingDamage>,
    #[serde(default)]
    pub rules_profile: RulesProfile,
    #[serde(default)]
    pub selected_ability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct PendingDamage {
    pub sequence: u32,
    pub target_id: String,
    pub move_name: String,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Battle {
    #[serde(default)]
    pub flow: Option<crate::combat::TurnFlow>,
    #[serde(default)]
    pub slots_per_side: u32,
    #[serde(default)]
    pub trainer_switches: BTreeMap<String, u32>,
    #[serde(default)]
    pub trainer_actions: BTreeMap<String, u32>,
    pub id: String,
    pub active: bool,
    pub round: u32,
    pub phase: String,
    pub combatants: Vec<Combatant>,
    #[serde(default = "enabled")]
    pub automatic_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct BattleResources {
    #[serde(default)]
    pub will: Option<i32>,
    pub hp: i32,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct BattleCheckpoint {
    pub event_sequence: u32,
    pub description: String,
    pub battle: Battle,
    pub resources: BTreeMap<String, BattleResources>,
    pub finished_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Event {
    pub sequence: u32,
    pub command_id: String,
    pub actor_id: String,
    pub description: String,
    pub dice: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub gm_id: String,
    #[serde(default)]
    pub host_id: String,
    pub revision: u32,
    pub catalog_hash: String,
    pub engine: String,
    pub sheets: BTreeMap<String, Sheet>,
    pub battle: Option<Battle>,
    #[serde(default)]
    pub finished_battles: Vec<Battle>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub battle_undo: Vec<BattleCheckpoint>,
    #[serde(default)]
    pub rules_profile: RulesProfile,
    #[serde(default)]
    pub proposals: Vec<WorkflowProposal>,
    #[serde(default)]
    pub training_encounters: Vec<crate::encounter::TrainingEncounter>,
}
impl Campaign {
    pub fn host_id(&self) -> &str {
        if self.host_id.is_empty() {
            &self.gm_id
        } else {
            &self.host_id
        }
    }
    pub fn new(name: String, gm_id: String, hash: String) -> Self {
        Self {
            id: id(),
            name,
            host_id: gm_id.clone(),
            gm_id,
            revision: 0,
            catalog_hash: hash,
            engine: ENGINE.into(),
            sheets: BTreeMap::new(),
            battle: None,
            finished_battles: vec![],
            events: vec![],
            battle_undo: vec![],
            rules_profile: RulesProfile::default(),
            proposals: vec![],
            training_encounters: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct SyncSheet {
    pub sheet: Sheet,
    pub baseline: Option<Sheet>,
    pub resolutions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum Action {
    AdjustWill {
        combatant_id: String,
        will: i32,
        reason: String,
    },
    AuthorizeWill {
        combatant_id: String,
        enabled: bool,
        reason: String,
    },
    AdjudicateStatus {
        combatant_id: String,
        damage: i32,
        reason: String,
    },
    BreakIce {
        combatant_id: String,
        target_id: String,
        move_name: String,
        reason: String,
    },
    TakeHeldTurn {
        combatant_id: String,
        reason: String,
    },
    StartTurns,
    ResolveAbility {
        event_id: String,
        choice: String,
        reason: String,
    },
    InflictStatus {
        combatant_id: String,
        source_id: String,
        kind: String,
        detail: String,
        rounds: Option<u32>,
        reason: String,
    },
    SetBattleRange {
        combatant_id: String,
        in_range_ids: Vec<String>,
        reason: String,
    },
    CoverAlly {
        combatant_id: String,
        ally_id: String,
        reason: String,
    },
    Tactics {
        command: crate::combat::tactics::TacticsCommand,
        reason: String,
    },
    SpecialMove {
        command: crate::combat::special::SpecialCommand,
        reason: String,
    },
    BattleStat {
        source_id: String,
        combatant_id: String,
        stat: String,
        change: i32,
        reason: String,
    },
    Battlefield {
        command: crate::combat::field::FieldCommand,
        reason: String,
    },
    AdjudicateField {
        sequence: u32,
        rounds: Option<u32>,
        reason: String,
    },
    RemoveFieldEffect {
        sequence: u32,
        reason: String,
    },
    AccuracyTargets {
        combatant_id: String,
        target_ids: Vec<String>,
        move_name: String,
        pool_modifier: i32,
        success_modifier: i32,
        reason: String,
    },
    PassTurn {
        combatant_id: String,
        reason: String,
    },
    React {
        combatant_id: String,
        kind: String,
        move_name: String,
        reason: String,
    },
    PassReaction {
        combatant_id: String,
        reason: String,
    },
    ResolveEffect {
        sequence: u32,
        reason: String,
    },
    AdjudicateEffect {
        sequence: u32,
        reason: String,
    },
    SetStatus {
        combatant_id: String,
        kind: String,
        detail: String,
        rounds: Option<u32>,
        remove: bool,
        reason: String,
    },
    StatusCheck {
        combatant_id: String,
        kind: String,
        reason: String,
    },
    SpendWill {
        combatant_id: String,
        mode: String,
        reason: String,
    },

    StartTeamBattle {
        sheet_ids: Vec<String>,
        sides: BTreeMap<String, String>,
        active_sheet_ids: Vec<String>,
        slots_per_side: u32,
    },
    SwitchPokemon {
        outgoing_id: String,
        incoming_id: String,
        gm_override: bool,
        reason: String,
    },
    AdoptCatalog,
    PublishTraining {
        encounter: crate::encounter::TrainingEncounter,
    },
    CancelTraining {
        encounter_id: String,
        reason: String,
    },
    ResolveTraining {
        resolution: crate::encounter::EncounterResolution,
    },
    ProposeWorkflow {
        request: Box<WorkflowRequest>,
    },
    ApproveWorkflow {
        proposal_id: String,
        accept: bool,
        reason: String,
    },
    SetRulesProfile {
        profile: RulesProfile,
        reason: String,
    },
    BattleConfig {
        automatic_validation: bool,
        reason: String,
    },
    RollbackBattle {
        event_sequence: u32,
        reason: String,
    },
    BattleStats {
        combatant_id: String,
        stats: BTreeMap<String, i32>,
        skills: BTreeMap<String, i32>,
        max_hp: i32,
        reason: String,
    },
    AssignGm {
        player_id: String,
    },
    UpgradeRules,
    Initiative {
        combatant_id: String,
    },
    Accuracy {
        combatant_id: String,
        #[serde(default)]
        target_id: Option<String>,
        move_name: String,
        pool_modifier: i32,
        success_modifier: i32,
        reason: String,
    },
    Damage {
        combatant_id: String,
        sequence: u32,
        pool_modifier: i32,
        success_modifier: i32,
        damage_modifier: i32,
        reason: String,
    },
    CancelDamage {
        combatant_id: String,
        sequence: u32,
        reason: String,
    },
    Sync {
        sheets: Vec<SyncSheet>,
    },
    StartBattle {
        sheet_ids: Vec<String>,
        sides: BTreeMap<String, String>,
    },
    NextRound,
    SetPhase {
        phase: String,
    },
    EndBattle,
    Roll {
        combatant_id: String,
        dice: u32,
        reason: String,
        use_action: bool,
    },
    Resolve {
        combatant_id: String,
        target_id: String,
        move_name: String,
        dice: u32,
        hp_delta: i32,
        condition: Option<String>,
        use_action: bool,
        reason: String,
    },
    Override {
        combatant_id: String,
        hp: i32,
        conditions: Vec<String>,
        actions: u32,
        reactions: u32,
        initiative: i32,
        reason: String,
    },
    Reward {
        sheet_id: String,
        xp: i32,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Command {
    pub id: String,
    pub expected_revision: u32,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Hello {
    pub player_id: String,
    pub token: String,
    pub credential: Option<String>,
    pub protocol: u32,
    #[serde(default)]
    pub build_version: String,
    pub catalog_hash: String,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum ServerMessage {
    Welcome {
        campaign: Campaign,
        credential: String,
    },
    Snapshot {
        campaign: Campaign,
    },
    Accepted {
        command_id: String,
        campaign: Campaign,
    },
    Error {
        command_id: Option<String>,
        error: AppError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct RoomInfo {
    pub address: String,
    pub token: String,
    pub campaign_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct JoinRequest {
    pub address: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct SaveRequest {
    pub operation_id: String,
    pub sheet: Sheet,
    pub expected_revision: u32,
    pub xp_cost: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct PointTransferRequest {
    pub kind: String,
    pub operation_id: String,
    pub trainer_id: String,
    pub pokemon_id: String,
    pub trainer_revision: u32,
    pub pokemon_revision: u32,
    pub catalog_hash: String,
    pub amount: i32,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct PointTransferResult {
    pub trainer: Sheet,
    pub pokemon: Sheet,
}
