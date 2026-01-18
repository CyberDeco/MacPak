//! Dialogue tab state

use floem::prelude::*;
use floem::reactive::SignalGet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use MacLarian::dialog::{Dialog, DialogNode, NodeConstructor, LocalizationCache, FlagCache, SpeakerCache, DifficultyClassCache};
pub use MacLarian::formats::voice_meta::{VoiceMetaEntry, VoiceMetaCache};
use MacLarian::formats::wem::AudioCache;

/// Source of a dialog file
#[derive(Clone, Debug, PartialEq)]
pub enum DialogSource {
    /// File extracted on disk
    LocalFile(PathBuf),
    /// File inside a PAK archive
    PakFile {
        pak_path: PathBuf,
        internal_path: String,
    },
}

/// A resolved flag for display
#[derive(Clone, Debug)]
pub struct DisplayFlag {
    /// The flag's name (resolved from cache, or UUID if not found)
    pub name: String,
    /// The flag's value (true/false)
    pub value: bool,
    /// Parameter value (for non-boolean checks)
    pub param_val: Option<i32>,
}

/// Entry in the dialog file browser
#[derive(Clone, Debug, PartialEq)]
pub struct DialogEntry {
    /// Display name (filename without path)
    pub name: String,
    /// Full path for display
    pub path: String,
    /// Source for loading
    pub source: DialogSource,
}

/// A processed node ready for display
#[derive(Clone, Debug)]
pub struct DisplayNode {
    /// Index in the flat list
    pub index: usize,
    /// Node UUID
    pub uuid: String,
    /// Parent node UUID (None for root nodes)
    pub parent_uuid: Option<String>,
    /// Node type
    pub constructor: NodeConstructor,
    /// Speaker name (resolved)
    pub speaker_name: String,
    /// Primary text (resolved from localization)
    pub text: String,
    /// Children UUIDs
    pub children: Vec<String>,
    /// Number of children
    pub child_count: usize,
    /// Depth in the tree
    pub depth: usize,
    /// Whether this node's children are expanded
    pub is_expanded: RwSignal<bool>,
    /// Whether this node is currently visible (parent chain is expanded)
    pub is_visible: RwSignal<bool>,
    /// Whether this is an end node
    pub is_end_node: bool,
    /// Whether this has flags
    pub has_flags: bool,
    /// Check flags (conditions that must be met)
    pub check_flags: Vec<DisplayFlag>,
    /// Set flags (flags set when this node is reached)
    pub set_flags: Vec<DisplayFlag>,
    /// Roll info summary (if applicable)
    pub roll_info: Option<String>,
    /// Handle for localization lookup
    pub text_handle: Option<String>,
    /// Jump/Alias target UUID (for resolution)
    pub jump_target_uuid: Option<String>,
    /// Jump/Alias target text handle (for localization)
    pub jump_target_handle: Option<String>,
    /// Editor data key-value pairs for this node (dev notes)
    pub editor_data: HashMap<String, String>,
    /// For RollResult nodes: true = success path, false = failure path
    pub roll_success: Option<bool>,
}

impl DisplayNode {
    pub fn new(index: usize, uuid: String, constructor: NodeConstructor) -> Self {
        Self {
            index,
            uuid,
            parent_uuid: None,
            constructor,
            speaker_name: String::new(),
            text: String::new(),
            children: Vec::new(),
            child_count: 0,
            depth: 0,
            is_expanded: RwSignal::new(true),
            is_visible: RwSignal::new(true), // Root nodes start visible
            is_end_node: false,
            has_flags: false,
            check_flags: Vec::new(),
            set_flags: Vec::new(),
            roll_info: None,
            text_handle: None,
            jump_target_uuid: None,
            jump_target_handle: None,
            editor_data: HashMap::new(),
            roll_success: None,
        }
    }
}

/// State for the Dialogue tab
#[derive(Clone)]
pub struct DialogueState {
    // Data source configuration
    /// Path to BG3 game data folder
    pub game_data_path: RwSignal<Option<PathBuf>>,
    /// Current language for localization
    pub language: RwSignal<String>,
    /// Available languages
    pub available_languages: RwSignal<Vec<String>>,

    // Dialog browser
    /// List of available dialog files
    pub available_dialogs: RwSignal<Vec<DialogEntry>>,
    /// Currently selected dialog path
    pub selected_dialog_path: RwSignal<Option<String>>,
    /// Search filter for dialog list
    pub dialog_search: RwSignal<String>,

    // Current dialog
    /// The currently loaded dialog
    pub current_dialog: RwSignal<Option<Arc<Dialog>>>,
    /// Display nodes for the tree view
    pub display_nodes: RwSignal<Vec<DisplayNode>>,
    /// Flat list of visible node indices (for virtual scrolling)
    pub visible_node_indices: RwSignal<Vec<usize>>,
    /// Selected node index (deprecated, use selected_node_uuid)
    pub selected_node_index: RwSignal<Option<usize>>,
    /// Selected node UUID (preferred - stable across visibility changes)
    pub selected_node_uuid: RwSignal<Option<String>>,

    // UI state
    /// Search query within the dialog
    pub node_search: RwSignal<String>,
    /// Whether to show flags in the tree
    pub show_flags: RwSignal<bool>,
    /// Whether to show tags in the tree
    pub show_tags: RwSignal<bool>,
    /// Whether to show editor data
    pub show_editor_data: RwSignal<bool>,
    /// Node type filter (empty = show all)
    pub filter_node_types: RwSignal<Vec<String>>,

    // UI layout
    /// Width of browser panel (left side) in pixels
    pub browser_panel_width: RwSignal<f64>,
    /// Maximum content width across all nodes (for horizontal scroll)
    pub max_content_width: RwSignal<f32>,
    /// Version counter to trigger tree re-renders without creating per-node subscriptions
    pub tree_version: RwSignal<u64>,
    /// Version counter to trigger browser re-renders for selection updates
    pub browser_version: RwSignal<u64>,

    // Status
    /// Status message
    pub status_message: RwSignal<String>,
    /// Whether a loading operation is in progress
    pub is_loading: RwSignal<bool>,
    /// Whether flag index is being built (shows overlay)
    pub is_building_flag_index: RwSignal<bool>,
    /// Message to display during flag index building
    pub flag_index_message: RwSignal<String>,
    /// Error message if any
    pub error_message: RwSignal<Option<String>>,

    // Localization cache (shared)
    pub localization_loaded: RwSignal<bool>,
    /// Localization cache for text lookup
    pub localization_cache: Arc<RwLock<LocalizationCache>>,
    /// Speaker name cache for resolving UUIDs to DisplayName handles
    pub speaker_cache: Arc<RwLock<SpeakerCache>>,
    /// Flag cache for resolving flag UUIDs to names
    pub flag_cache: Arc<RwLock<FlagCache>>,
    /// Difficulty class cache for resolving DC UUIDs to numeric values
    pub difficulty_class_cache: Arc<RwLock<DifficultyClassCache>>,

    // Current dialog source (for reloading)
    /// The source of the currently loaded dialog (for reload functionality)
    pub current_source: RwSignal<Option<DialogSource>>,

    // Pending load (for loading from Search tab after cache init)
    /// Pending dialog to load once caches are ready
    pub pending_load: RwSignal<Option<DialogSource>>,
    /// Whether caches are ready for the pending load
    pub pending_caches_ready: RwSignal<bool>,

    // Voice/Audio
    /// Voice metadata cache mapping text handles to .wem file info
    pub voice_meta_cache: Arc<RwLock<VoiceMetaCache>>,
    /// Whether voice metadata has been loaded
    pub voice_meta_loaded: RwSignal<bool>,
    /// Path to Voice.pak or extracted voice files directory
    pub voice_files_path: RwSignal<Option<PathBuf>>,
    /// Currently playing audio node UUID (if any)
    pub playing_audio_node: RwSignal<Option<String>>,
    /// Audio cache for decoded WEM files (avoids re-decoding on replay)
    pub audio_cache: Arc<RwLock<AudioCache>>,
}

impl DialogueState {
    pub fn new() -> Self {
        Self {
            // Data source
            game_data_path: RwSignal::new(None),
            language: RwSignal::new("English".to_string()),
            available_languages: RwSignal::new(vec!["English".to_string()]),

            // Dialog browser
            available_dialogs: RwSignal::new(Vec::new()),
            selected_dialog_path: RwSignal::new(None),
            dialog_search: RwSignal::new(String::new()),

            // Current dialog
            current_dialog: RwSignal::new(None),
            display_nodes: RwSignal::new(Vec::new()),
            visible_node_indices: RwSignal::new(Vec::new()),
            selected_node_index: RwSignal::new(None),
            selected_node_uuid: RwSignal::new(None),

            // UI state
            node_search: RwSignal::new(String::new()),
            show_flags: RwSignal::new(true),
            show_tags: RwSignal::new(true),
            show_editor_data: RwSignal::new(false),
            filter_node_types: RwSignal::new(Vec::new()),

            // UI layout
            browser_panel_width: RwSignal::new(400.0),
            max_content_width: RwSignal::new(0.0),
            tree_version: RwSignal::new(0),
            browser_version: RwSignal::new(0),

            // Status
            status_message: RwSignal::new("Ready".to_string()),
            is_loading: RwSignal::new(false),
            is_building_flag_index: RwSignal::new(false),
            flag_index_message: RwSignal::new(String::new()),
            error_message: RwSignal::new(None),

            // Localization
            localization_loaded: RwSignal::new(false),
            localization_cache: Arc::new(RwLock::new(LocalizationCache::new())),
            speaker_cache: Arc::new(RwLock::new(SpeakerCache::new())),
            flag_cache: Arc::new(RwLock::new(FlagCache::new())),
            difficulty_class_cache: Arc::new(RwLock::new(DifficultyClassCache::new())),

            // Current dialog source
            current_source: RwSignal::new(None),

            // Pending load
            pending_load: RwSignal::new(None),
            pending_caches_ready: RwSignal::new(false),

            // Voice/Audio
            voice_meta_cache: Arc::new(RwLock::new(HashMap::new())),
            voice_meta_loaded: RwSignal::new(false),
            voice_files_path: RwSignal::new(None),
            playing_audio_node: RwSignal::new(None),
            audio_cache: Arc::new(RwLock::new(AudioCache::new())),
        }
    }

    /// Apply persisted state (call after new())
    pub fn apply_persisted(&self, persisted: &super::PersistedDialogueState) {
        // Restore language preference
        if !persisted.language.is_empty() {
            self.language.set(persisted.language.clone());
        }

        // Restore UI preferences
        self.show_flags.set(persisted.show_flags);
        self.show_tags.set(persisted.show_tags);
        self.show_editor_data.set(persisted.show_editor_data);

        // Restore layout
        self.browser_panel_width.set(persisted.browser_panel_width);
    }

    /// Clear the current dialog and display
    pub fn clear_dialog(&self) {
        self.current_dialog.set(None);
        self.display_nodes.set(Vec::new());
        self.visible_node_indices.set(Vec::new());
        self.selected_node_index.set(None);
        self.selected_node_uuid.set(None);
    }

    /// Get the currently selected display node
    pub fn selected_display_node(&self) -> Option<DisplayNode> {
        let uuid = self.selected_node_uuid.get()?;
        let nodes = self.display_nodes.get();
        nodes.iter().find(|n| n.uuid == uuid).cloned()
    }

    /// Get the currently selected dialog node
    pub fn selected_dialog_node(&self) -> Option<DialogNode> {
        let display_node = self.selected_display_node()?;
        let dialog = self.current_dialog.get()?;
        dialog.get_node(&display_node.uuid).cloned()
    }

    /// Get voice metadata for a text handle
    /// The handle should be in the format "hXXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
    /// It will be normalized to the VoiceMeta format (dashes replaced with 'g')
    pub fn get_voice_meta(&self, text_handle: &str) -> Option<VoiceMetaEntry> {
        // Normalize the handle: replace dashes with 'g' to match VoiceMeta format
        let normalized = text_handle.replace('-', "g");
        let cache = self.voice_meta_cache.read().ok()?;
        cache.get(&normalized).cloned()
    }

    /// Check if a text handle has associated audio
    pub fn has_audio(&self, text_handle: &str) -> bool {
        self.get_voice_meta(text_handle).is_some()
    }
}

impl Default for DialogueState {
    fn default() -> Self {
        Self::new()
    }
}

/// Available node type options for filtering
pub const NODE_TYPE_OPTIONS: &[(&str, &str)] = &[
    ("TagAnswer", "Answer"),
    ("TagQuestion", "Question"),
    ("ActiveRoll", "Active Roll"),
    ("PassiveRoll", "Passive Roll"),
    ("RollResult", "Roll Result"),
    ("Alias", "Alias"),
    ("Jump", "Jump"),
    ("TagCinematic", "Cinematic"),
    ("Trade", "Trade"),
    ("NestedDialog", "Nested Dialog"),
];
