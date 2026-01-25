//! Dialog data types for BG3 dialog files
//!
//! These types represent the parsed dialog structure from BG3's .lsf/.lsj dialog files.
//! Ported from bg3-dialog-reader's Json.cs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root dialog structure containing all dialog data
#[derive(Debug, Clone, Default)]
pub struct Dialog {
    /// Unique identifier for this dialog
    pub uuid: String,
    /// Dialog category
    pub category: Option<String>,
    /// Timeline ID for cinematics
    pub timeline_id: Option<String>,
    /// Root node UUIDs - entry points for the dialog
    pub root_nodes: Vec<String>,
    /// All dialog nodes, indexed by UUID for quick lookup
    pub nodes: HashMap<String, DialogNode>,
    /// Ordered list of node UUIDs (preserves original order)
    pub node_order: Vec<String>,
    /// Speaker definitions (index -> speaker info)
    pub speakers: HashMap<i32, SpeakerInfo>,
    /// Default addressed speakers mapping
    pub default_addressed_speakers: HashMap<i32, i32>,
    /// Editor metadata
    pub editor_data: DialogEditorData,
}

/// Editor-specific metadata for the dialog
#[derive(Debug, Clone, Default)]
pub struct DialogEditorData {
    /// How to trigger this dialog (editor notes)
    pub how_to_trigger: Option<String>,
    /// Synopsis/summary of the dialog
    pub synopsis: Option<String>,
    /// Next node ID for editor
    pub next_node_id: Option<i32>,
    /// Whether layout is needed
    pub needs_layout: bool,
    /// Default attitudes per speaker
    pub default_attitudes: HashMap<String, String>,
    /// Default emotions per speaker
    pub default_emotions: HashMap<String, String>,
    /// Peanut speaker flags
    pub is_peanut: HashMap<String, String>,
}

/// A single dialog node
#[derive(Debug, Clone, Default)]
pub struct DialogNode {
    /// Unique identifier
    pub uuid: String,
    /// Node type (`TagAnswer`, `TagQuestion`, `ActiveRoll`, etc.)
    pub constructor: NodeConstructor,
    /// Speaker index (-1 = no speaker, -666 = narrator)
    pub speaker: Option<i32>,
    /// Child node UUIDs
    pub children: Vec<String>,
    /// Dialog text variants with localization handles
    pub tagged_texts: Vec<TaggedText>,
    /// Conditions that must be met
    pub check_flags: Vec<FlagGroup>,
    /// Flags to set when this node is reached
    pub set_flags: Vec<FlagGroup>,
    /// Tags on this node
    pub tags: Vec<String>,
    /// Jump target UUID (for Jump nodes)
    pub jump_target: Option<String>,
    /// Jump target point
    pub jump_target_point: Option<i32>,
    /// Whether this is an end node
    pub end_node: bool,
    /// Source node UUID (for Alias nodes)
    pub source_node: Option<String>,
    /// Show only once
    pub show_once: bool,
    /// Pop levels for nested dialogs
    pub pop_level: Option<i32>,
    /// Transition mode
    pub transition_mode: Option<i32>,
    /// Wait time before proceeding
    pub wait_time: Option<f32>,
    /// Whether this option is optional
    pub optional: bool,

    // Roll-specific fields
    /// Ability for rolls (Strength, Charisma, etc.)
    pub ability: Option<String>,
    /// Skill for rolls (Persuasion, Deception, etc.)
    pub skill: Option<String>,
    /// Difficulty class ID
    pub difficulty_class_id: Option<String>,
    /// Difficulty modifier
    pub difficulty_mod: Option<i32>,
    /// Level override
    pub level_override: Option<i32>,
    /// Advantage modifier (-1 = disadvantage, 0 = normal, 1 = advantage)
    pub advantage: Option<i32>,
    /// Roll type
    pub roll_type: Option<String>,
    /// Roll target speaker index
    pub roll_target_speaker: Option<i32>,
    /// Whether this is a success node (for `RollResult`)
    pub success: Option<bool>,
    /// Exclude companion optional bonuses
    pub exclude_companions_optional_bonuses: bool,
    /// Exclude speaker optional bonuses
    pub exclude_speaker_optional_bonuses: bool,
    /// Persuasion target speaker index
    pub persuasion_target_speaker_index: Option<i32>,
    /// Stat name for certain rolls
    pub stat_name: Option<String>,
    /// Stats attribute
    pub stats_attribute: Option<String>,

    // Grouping fields
    /// Group ID for grouped choices
    pub group_id: Option<String>,
    /// Group index
    pub group_index: Option<i32>,
    /// Whether this is a root of a group
    pub root: bool,

    // Approval
    /// Approval rating ID (companion reactions)
    pub approval_rating_id: Option<String>,

    // Validated flags
    pub validated_has_value: bool,

    // Game data
    pub game_data: Option<GameData>,

    /// Editor-specific key-value data
    pub editor_data: HashMap<String, String>,
}

/// Node constructor/type
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NodeConstructor {
    #[default]
    TagAnswer,
    TagQuestion,
    ActiveRoll,
    PassiveRoll,
    Alias,
    VisualState,
    RollResult,
    TagCinematic,
    Trade,
    NestedDialog,
    FallibleQuestionResult,
    Jump,
    Pop,
    TagGreeting,
    Other(String),
}

impl NodeConstructor {
    #[must_use] 
    pub fn from_str(s: &str) -> Self {
        match s {
            "TagAnswer" => NodeConstructor::TagAnswer,
            "TagQuestion" => NodeConstructor::TagQuestion,
            "ActiveRoll" => NodeConstructor::ActiveRoll,
            "PassiveRoll" => NodeConstructor::PassiveRoll,
            "Alias" => NodeConstructor::Alias,
            "VisualState" | "Visual State" => NodeConstructor::VisualState,
            "RollResult" => NodeConstructor::RollResult,
            "TagCinematic" => NodeConstructor::TagCinematic,
            "Trade" => NodeConstructor::Trade,
            "NestedDialog" | "Nested Dialog" => NodeConstructor::NestedDialog,
            "FallibleQuestionResult" => NodeConstructor::FallibleQuestionResult,
            "Jump" => NodeConstructor::Jump,
            "Pop" => NodeConstructor::Pop,
            "TagGreeting" => NodeConstructor::TagGreeting,
            other => NodeConstructor::Other(other.to_string()),
        }
    }

    #[must_use] 
    pub fn as_str(&self) -> &str {
        match self {
            NodeConstructor::TagAnswer => "TagAnswer",
            NodeConstructor::TagQuestion => "TagQuestion",
            NodeConstructor::ActiveRoll => "ActiveRoll",
            NodeConstructor::PassiveRoll => "PassiveRoll",
            NodeConstructor::Alias => "Alias",
            NodeConstructor::VisualState => "VisualState",
            NodeConstructor::RollResult => "RollResult",
            NodeConstructor::TagCinematic => "TagCinematic",
            NodeConstructor::Trade => "Trade",
            NodeConstructor::NestedDialog => "NestedDialog",
            NodeConstructor::FallibleQuestionResult => "FallibleQuestionResult",
            NodeConstructor::Jump => "Jump",
            NodeConstructor::Pop => "Pop",
            NodeConstructor::TagGreeting => "TagGreeting",
            NodeConstructor::Other(s) => s,
        }
    }

    /// Returns a display-friendly name for UI
    #[must_use] 
    pub fn display_name(&self) -> &str {
        match self {
            NodeConstructor::TagAnswer => "Answer",
            NodeConstructor::TagQuestion => "Question",
            NodeConstructor::ActiveRoll => "Active Roll",
            NodeConstructor::PassiveRoll => "Passive Roll",
            NodeConstructor::Alias => "Alias",
            NodeConstructor::VisualState => "Visual State",
            NodeConstructor::RollResult => "Roll Result",
            NodeConstructor::TagCinematic => "Cinematic",
            NodeConstructor::Trade => "Trade",
            NodeConstructor::NestedDialog => "Nested Dialog",
            NodeConstructor::FallibleQuestionResult => "Fallible Result",
            NodeConstructor::Jump => "Jump",
            NodeConstructor::Pop => "Pop",
            NodeConstructor::TagGreeting => "Greeting",
            NodeConstructor::Other(s) => s,
        }
    }
}

/// Tagged text entry with optional rule conditions
#[derive(Debug, Clone, Default)]
pub struct TaggedText {
    /// Whether this text has tag rules
    pub has_tag_rule: bool,
    /// Rule groups for conditional text
    pub rule_groups: Vec<RuleGroup>,
    /// The actual text entries
    pub tag_texts: Vec<TagTextEntry>,
}

/// A single text entry within `TaggedText`
#[derive(Debug, Clone, Default)]
pub struct TagTextEntry {
    /// Line ID for this text
    pub line_id: Option<String>,
    /// Localization handle
    pub handle: String,
    /// Cached/embedded text value (may be empty)
    pub value: Option<String>,
    /// Version of the translation
    pub version: Option<u16>,
    /// Whether this is a stub entry
    pub stub: bool,
}

/// Rule group for conditional text display
#[derive(Debug, Clone, Default)]
pub struct RuleGroup {
    /// Combination operator for tags
    pub tag_combine_op: i32,
    /// Rules in this group
    pub rules: Vec<Rule>,
}

/// A single rule within a `RuleGroup`
#[derive(Debug, Clone, Default)]
pub struct Rule {
    /// Whether this rule has child rules
    pub has_child_rules: bool,
    /// Tag combination operator
    pub tag_combine_op: i32,
    /// Tag UUIDs required for this rule
    pub tags: Vec<String>,
    /// Speaker index for this rule
    pub speaker: Option<i32>,
}

/// A group of flags (check or set)
#[derive(Debug, Clone, Default)]
pub struct FlagGroup {
    /// Type of flag group
    pub flag_type: FlagType,
    /// Flags in this group
    pub flags: Vec<Flag>,
}

/// Type of flag
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FlagType {
    #[default]
    Local,
    Tag,
    Global,
    Object,
    Dialog,
    Quest,
    Other(String),
}

impl FlagType {
    #[must_use] 
    pub fn from_str(s: &str) -> Self {
        match s {
            "Local" => FlagType::Local,
            "Tag" => FlagType::Tag,
            "Global" => FlagType::Global,
            "Object" => FlagType::Object,
            "Dialog" => FlagType::Dialog,
            "Quest" => FlagType::Quest,
            other => FlagType::Other(other.to_string()),
        }
    }

    #[must_use] 
    pub fn as_str(&self) -> &str {
        match self {
            FlagType::Local => "Local",
            FlagType::Tag => "Tag",
            FlagType::Global => "Global",
            FlagType::Object => "Object",
            FlagType::Dialog => "Dialog",
            FlagType::Quest => "Quest",
            FlagType::Other(s) => s,
        }
    }
}

/// A single flag entry
#[derive(Debug, Clone, Default)]
pub struct Flag {
    /// Flag UUID
    pub uuid: String,
    /// Flag value (true/false for boolean flags)
    pub value: bool,
    /// Parameter value (for non-boolean flags)
    pub param_val: Option<i32>,
    /// Optional flag name (may be resolved from database)
    pub name: Option<String>,
}

/// Speaker information
#[derive(Debug, Clone, Default)]
pub struct SpeakerInfo {
    /// Speaker index in the dialog
    pub index: i32,
    /// Speaker mapping ID (UUID)
    pub speaker_mapping_id: String,
    /// List of speaker UUIDs (for speaker group)
    pub speaker_list: Vec<String>,
    /// Whether this is a "peanut" speaker (minor character)
    pub is_peanut_speaker: bool,
}

/// Game data associated with a node
#[derive(Debug, Clone, Default)]
pub struct GameData {
    /// AI personalities
    pub ai_personalities: Vec<String>,
    /// Origin sounds
    pub origin_sounds: Vec<String>,
    /// Music instrument sounds
    pub music_instrument_sounds: Vec<String>,
}

// Convenience methods for Dialog
impl Dialog {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a node by UUID
    #[must_use] 
    pub fn get_node(&self, uuid: &str) -> Option<&DialogNode> {
        self.nodes.get(uuid)
    }

    /// Get all root nodes
    #[must_use] 
    pub fn get_root_nodes(&self) -> Vec<&DialogNode> {
        self.root_nodes
            .iter()
            .filter_map(|uuid| self.nodes.get(uuid))
            .collect()
    }

    /// Get children of a node
    #[must_use] 
    pub fn get_children(&self, node: &DialogNode) -> Vec<&DialogNode> {
        node.children
            .iter()
            .filter_map(|uuid| self.nodes.get(uuid))
            .collect()
    }

    /// Get speaker info by index
    #[must_use] 
    pub fn get_speaker(&self, index: i32) -> Option<&SpeakerInfo> {
        self.speakers.get(&index)
    }

    /// Get the primary text for a node (first text entry, first tag text)
    #[must_use] 
    pub fn get_node_text<'a>(&self, node: &'a DialogNode) -> Option<&'a TagTextEntry> {
        node.tagged_texts.first()
            .and_then(|tt| tt.tag_texts.first())
    }

    /// Count total nodes
    #[must_use] 
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl DialogNode {
    #[must_use] 
    pub fn new(uuid: String, constructor: NodeConstructor) -> Self {
        Self {
            uuid,
            constructor,
            ..Default::default()
        }
    }

    /// Check if this node has any text
    #[must_use] 
    pub fn has_text(&self) -> bool {
        self.tagged_texts.iter()
            .any(|tt| !tt.tag_texts.is_empty())
    }

    /// Get all text handles from this node
    #[must_use] 
    pub fn get_text_handles(&self) -> Vec<&str> {
        self.tagged_texts.iter()
            .flat_map(|tt| tt.tag_texts.iter())
            .map(|t| t.handle.as_str())
            .collect()
    }

    /// Check if this node is a roll type
    #[must_use] 
    pub fn is_roll(&self) -> bool {
        matches!(
            self.constructor,
            NodeConstructor::ActiveRoll | NodeConstructor::PassiveRoll
        )
    }

    /// Check if this node is a question type
    #[must_use] 
    pub fn is_question(&self) -> bool {
        matches!(self.constructor, NodeConstructor::TagQuestion)
    }

    /// Check if this node is an answer type
    #[must_use] 
    pub fn is_answer(&self) -> bool {
        matches!(self.constructor, NodeConstructor::TagAnswer)
    }
}
