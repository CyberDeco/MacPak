//! Speaker name resolution - maps speaker UUIDs to character names
//!
//! Provides runtime speaker name loading from RootTemplates and Level Characters.
//! Currently, speaker names are resolved via the embedded database, but these
//! functions can be used for dynamic loading when needed.

use std::collections::HashMap;
use std::path::Path;
use MacLarian::formats::lsf::parse_lsf_bytes;
use MacLarian::converter::to_lsx;
use MacLarian::pak::PakOperations;
use crate::gui::state::DialogueState;

/// Load speaker DisplayName handles from RootTemplates and Level Characters in the PAK
#[allow(dead_code)]
pub fn try_load_speaker_names(state: &DialogueState, pak_path: &Path) {
    let cache = state.speaker_cache.clone();

    if let Ok(mut cache) = cache.write() {
        // Load character instances from Level Characters (these are what dialogs reference)
        let _ = cache.load_characters_from_pak(pak_path);

        // Load templates from RootTemplates (fallback for template references)
        let _ = cache.load_display_names_from_pak(pak_path);

        // Also try to load from Shared.pak if we're loading from Gustav.pak
        if let Some(parent) = pak_path.parent() {
            let shared_pak = parent.join("Shared.pak");
            if shared_pak.exists() && shared_pak != pak_path {
                let _ = cache.load_characters_from_pak(&shared_pak);
                let _ = cache.load_display_names_from_pak(&shared_pak);
            }
        }
    }
}

/// Cache of speaker UUID to name mappings
#[derive(Debug, Clone, Default)]
pub struct SpeakerNameCache {
    /// UUID → Character name (hardcoded companions)
    pub names: HashMap<String, String>,
    /// UUID → DisplayName localization handle (from RootTemplates)
    pub display_name_handles: HashMap<String, String>,
}

impl SpeakerNameCache {
    pub fn new() -> Self {
        let mut cache = Self {
            names: HashMap::new(),
            display_name_handles: HashMap::new(),
        };
        // Pre-populate with known companions
        cache.add_known_companions();
        cache
    }

    /// Add the known main companion UUIDs
    fn add_known_companions(&mut self) {
        // Main companions - Character UUIDs (from Origins.lsx UUID field)
        self.names.insert("2bb39cf2-4649-4238-8d0c-44f62b5a3dfd".to_string(), "Shadowheart".to_string());
        self.names.insert("35c3caad-5543-4593-be75-e7deba30f062".to_string(), "Gale".to_string());
        self.names.insert("3780c689-d903-41c2-bf64-1e6ec6a8e1e5".to_string(), "Astarion".to_string());
        self.names.insert("efc9d114-0296-4a30-b701-365fc07d44fb".to_string(), "Wyll".to_string());
        self.names.insert("fb3bc4c3-49eb-4944-b714-d0cb357bb635".to_string(), "Lae'zel".to_string());
        self.names.insert("b8b4a974-b045-45f6-9516-b457b8773abd".to_string(), "Karlach".to_string());
        self.names.insert("c1f137c7-a17c-47b0-826a-12e44a8ec45c".to_string(), "Jaheira".to_string());
        self.names.insert("eae09670-869d-4b70-b605-33af4ee80b34".to_string(), "Minthara".to_string());
        self.names.insert("e1b629bc-7340-4fe6-81a4-834a838ff5c5".to_string(), "Minsc".to_string());
        self.names.insert("a36281c5-adcd-4d6e-8e5a-b5650b8f17eb".to_string(), "Halsin".to_string());
        self.names.insert("38357c93-b437-4f03-88d0-a67bd4c0e3e9".to_string(), "Alfira".to_string());
        self.names.insert("5af0f42c-9b32-4c3c-b108-46c44196081b".to_string(), "The Dark Urge".to_string());
        self.names.insert("a4b56492-d5ac-4a84-8e45-5437cd9da7f3".to_string(), "Custom".to_string());

        // Main companions - GlobalTemplate UUIDs (from Origins.lsx GlobalTemplate field)
        // These are what dialog speakerlists reference
        self.names.insert("c7c13742-bacd-460a-8f65-f864fe41f255".to_string(), "Astarion".to_string());
        self.names.insert("58a69333-40bf-8358-1d17-fff240d7fb12".to_string(), "Lae'zel".to_string());
        self.names.insert("ad9af97d-75da-406a-ae13-7071c563f604".to_string(), "Gale".to_string());
        self.names.insert("3ed74f06-3c60-42dc-83f6-f034cb47c679".to_string(), "Shadowheart".to_string());
        self.names.insert("c774d764-4a17-48dc-b470-32ace9ce447d".to_string(), "Wyll".to_string());
        self.names.insert("2c76687d-93a2-477b-8b18-8a14b549304c".to_string(), "Karlach".to_string());
        self.names.insert("91b6b200-7d00-4d62-8dc9-99e8339dfa1a".to_string(), "Jaheira".to_string());
        self.names.insert("25721313-0c15-4935-8176-9f134385451b".to_string(), "Minthara".to_string());
        self.names.insert("0de603c5-42e2-4811-9dad-f652de080eba".to_string(), "Minsc".to_string());
        self.names.insert("7628bc0e-52b8-42a7-856a-13a6fd413323".to_string(), "Halsin".to_string());
        self.names.insert("4a405fba-3000-4c63-97e5-a8001ebb883c".to_string(), "Alfira".to_string());

        // Player character (Tav)
        self.names.insert("e0d1ff71-04a8-4340-ae64-9684d846eb83".to_string(), "Player".to_string());

        // Dark Urge GlobalTemplate (used in dialog speakerlists)
        self.names.insert("e6b3c2c4-e88d-e9e6-ffa1-d49cdfadd411".to_string(), "The Dark Urge".to_string());
    }

    /// Get a hardcoded speaker name by UUID
    pub fn get(&self, uuid: &str) -> Option<&String> {
        self.names.get(uuid)
    }

    /// Get the DisplayName localization handle for a UUID
    pub fn get_display_handle(&self, uuid: &str) -> Option<&String> {
        self.display_name_handles.get(uuid)
    }

    /// Load DisplayName handles from Level Characters in a PAK file
    /// These are character instances placed in levels - what dialogs reference
    pub fn load_characters_from_pak(&mut self, pak_path: &Path) -> Result<usize, String> {
        let file_list = PakOperations::list(pak_path)
            .map_err(|e| format!("Failed to list PAK: {}", e))?;

        // Find all Characters/_merged.lsf files in Levels folders
        let character_files: Vec<_> = file_list
            .iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                lower.contains("/levels/") &&
                lower.contains("/characters/") &&
                lower.ends_with("_merged.lsf")
            })
            .cloned()
            .collect();

        let mut total_count = 0;

        for char_path in character_files {
            if let Ok(count) = self.load_lsf_display_names(pak_path, &char_path) {
                total_count += count;
            }
        }

        Ok(total_count)
    }

    /// Load DisplayName handles from RootTemplates in a PAK file
    pub fn load_display_names_from_pak(&mut self, pak_path: &Path) -> Result<usize, String> {
        let file_list = PakOperations::list(pak_path)
            .map_err(|e| format!("Failed to list PAK: {}", e))?;

        // Find all RootTemplates files
        let template_files: Vec<_> = file_list
            .iter()
            .filter(|p| p.to_lowercase().contains("roottemplates") && p.to_lowercase().ends_with(".lsf"))
            .cloned()
            .collect();

        let mut total_count = 0;

        for template_path in template_files {
            if let Ok(count) = self.load_lsf_display_names(pak_path, &template_path) {
                total_count += count;
            }
        }

        Ok(total_count)
    }

    /// Load DisplayName handles from a single LSF file (RootTemplates or Characters)
    fn load_lsf_display_names(&mut self, pak_path: &Path, internal_path: &str) -> Result<usize, String> {
        // Read the LSF file from the PAK
        let data = PakOperations::read_file_bytes(pak_path, internal_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse LSF to document
        let lsf_doc = parse_lsf_bytes(&data)
            .map_err(|e| format!("LSF parse error: {}", e))?;

        // Convert to LSX XML string
        let lsx_xml = to_lsx(&lsf_doc)
            .map_err(|e| format!("LSF→LSX error: {}", e))?;

        // Extract UUID and DisplayName handle pairs from the XML
        self.extract_display_names_from_lsx(&lsx_xml)
    }

    /// Extract UUID → DisplayName handle pairs from LSX XML content
    fn extract_display_names_from_lsx(&mut self, xml: &str) -> Result<usize, String> {
        let doc = roxmltree::Document::parse(xml)
            .map_err(|e| format!("XML parse error: {}", e))?;

        let mut count = 0;

        // Find all GameObjects nodes
        for node in doc.descendants() {
            if node.tag_name().name() == "node" {
                if let Some(id) = node.attribute("id") {
                    if id == "GameObjects" {
                        // Extract UUID/MapKey and DisplayName handle from this node's children
                        let mut uuid: Option<String> = None;
                        let mut display_handle: Option<String> = None;

                        for child in node.children() {
                            if child.tag_name().name() == "attribute" {
                                let attr_id = child.attribute("id").unwrap_or("");

                                match attr_id {
                                    "MapKey" => {
                                        if let Some(value) = child.attribute("value") {
                                            uuid = Some(value.to_string());
                                        }
                                    }
                                    "DisplayName" => {
                                        // DisplayName is a TranslatedString with a "handle" attribute
                                        if let Some(handle) = child.attribute("handle") {
                                            display_handle = Some(handle.to_string());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Save the mapping if we have both UUID and DisplayName handle
                        if let (Some(uuid), Some(handle)) = (uuid, display_handle) {
                            // Don't override hardcoded names
                            if !self.names.contains_key(&uuid) && !self.display_name_handles.contains_key(&uuid) {
                                self.display_name_handles.insert(uuid, handle);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}
