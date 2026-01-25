//! Parser for converting LSJ documents to Dialog structures
//!
//! Extracts dialog data from the generic LSJ format into typed Dialog structures.

use maclarian::formats::lsj::{LsjDocument, LsjNode, LsjAttribute};
use super::types::{Dialog, SpeakerInfo, DialogNode, NodeConstructor, GameData, FlagGroup, FlagType, Flag, TaggedText, Rule, RuleGroup, TagTextEntry, DialogEditorData};

/// Parse a dialog from an LSJ document
///
/// # Errors
/// Returns an error if the document is missing required regions or has invalid format.
pub fn parse_dialog(doc: &LsjDocument) -> Result<Dialog, DialogParseError> {
    let mut dialog = Dialog::new();

    // Get the dialog region
    let dialog_region = doc.save.regions.get("dialog")
        .or_else(|| doc.save.regions.get("Dialog"))
        .ok_or(DialogParseError::MissingRegion("dialog".to_string()))?;

    // Parse dialog-level attributes
    if let Some(uuid_attr) = dialog_region.attributes.get("UUID") {
        dialog.uuid = get_string_value(uuid_attr);
    }
    if let Some(cat_attr) = dialog_region.attributes.get("category") {
        dialog.category = Some(get_string_value(cat_attr));
    }
    if let Some(timeline_attr) = dialog_region.attributes.get("TimelineId") {
        dialog.timeline_id = Some(get_string_value(timeline_attr));
    }

    // Parse speaker list
    if let Some(speaker_lists) = dialog_region.children.get("speakerlist") {
        for speaker_list_node in speaker_lists {
            if let Some(speakers) = speaker_list_node.children.get("speaker") {
                for speaker_node in speakers {
                    if let Some(speaker_info) = parse_speaker(speaker_node) {
                        dialog.speakers.insert(speaker_info.index, speaker_info);
                    }
                }
            }
        }
    }

    // Parse default addressed speakers
    if let Some(das_list) = dialog_region.children.get("DefaultAddressedSpeakers") {
        for das_node in das_list {
            if let Some(objects) = das_node.children.get("Object") {
                for obj in objects {
                    let map_key = obj.attributes.get("MapKey")
                        .map_or(0, get_int_value);
                    let map_value = obj.attributes.get("MapValue")
                        .map_or(0, get_int_value);
                    dialog.default_addressed_speakers.insert(map_key, map_value);
                }
            }
        }
    }

    // Parse nodes
    if let Some(nodes_list) = dialog_region.children.get("nodes") {
        for nodes_container in nodes_list {
            // Parse root nodes
            if let Some(root_nodes_list) = nodes_container.children.get("RootNodes") {
                for root_node in root_nodes_list {
                    if let Some(rn_attr) = root_node.attributes.get("RootNodes") {
                        let uuid = get_string_value(rn_attr);
                        if !uuid.is_empty() {
                            dialog.root_nodes.push(uuid);
                        }
                    }
                }
            }

            // Parse node definitions
            if let Some(node_list) = nodes_container.children.get("node") {
                for node_def in node_list {
                    if let Some(node) = parse_node(node_def) {
                        let uuid = node.uuid.clone();
                        dialog.node_order.push(uuid.clone());
                        dialog.nodes.insert(uuid, node);
                    }
                }
            }
        }
    }

    // Parse editor data region
    if let Some(editor_region) = doc.save.regions.get("editorData")
        .or_else(|| doc.save.regions.get("EditorData"))
    {
        dialog.editor_data = parse_editor_data(editor_region);
    }

    Ok(dialog)
}

/// Parse a speaker from LSJ node
fn parse_speaker(node: &LsjNode) -> Option<SpeakerInfo> {
    let index = node.attributes.get("index")
        .map_or(0, |a| get_string_value(a).parse::<i32>().unwrap_or(0));

    let speaker_mapping_id = node.attributes.get("SpeakerMappingId")
        .map(get_string_value)
        .unwrap_or_default();

    let speaker_list: Vec<String> = node.attributes.get("list")
        .map(|a| {
            get_string_value(a)
                .split(';')
                .filter(|s| !s.is_empty())
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    let is_peanut = node.attributes.get("IsPeanutSpeaker")
        .is_some_and(get_bool_value);

    Some(SpeakerInfo {
        index,
        speaker_mapping_id,
        speaker_list,
        is_peanut_speaker: is_peanut,
    })
}

/// Parse a dialog node from LSJ
fn parse_node(node: &LsjNode) -> Option<DialogNode> {
    let uuid = node.attributes.get("UUID")
        .map(get_string_value)?;

    let constructor = node.attributes.get("constructor")
        .map(|a| NodeConstructor::from_str(&get_string_value(a)))
        .unwrap_or_default();

    let mut dialog_node = DialogNode::new(uuid, constructor);

    // Basic attributes
    if let Some(attr) = node.attributes.get("speaker") {
        dialog_node.speaker = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("SourceNode") {
        dialog_node.source_node = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("ShowOnce") {
        dialog_node.show_once = get_bool_value(attr);
    }
    if let Some(attr) = node.attributes.get("endnode") {
        dialog_node.end_node = get_bool_value(attr);
    }
    if let Some(attr) = node.attributes.get("jumptarget") {
        dialog_node.jump_target = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("jumptargetpoint") {
        dialog_node.jump_target_point = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("transitionmode") {
        dialog_node.transition_mode = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("waittime") {
        dialog_node.wait_time = Some(get_float_value(attr));
    }
    if let Some(attr) = node.attributes.get("optional") {
        dialog_node.optional = get_bool_value(attr);
    }
    if let Some(attr) = node.attributes.get("PopLevel") {
        dialog_node.pop_level = Some(get_int_value(attr));
    }

    // Roll attributes
    if let Some(attr) = node.attributes.get("Ability") {
        dialog_node.ability = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("Skill") {
        dialog_node.skill = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("DifficultyClassID") {
        dialog_node.difficulty_class_id = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("DifficultyMod") {
        dialog_node.difficulty_mod = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("LevelOverride") {
        dialog_node.level_override = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("Advantage") {
        dialog_node.advantage = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("RollType") {
        dialog_node.roll_type = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("RollTargetSpeaker") {
        dialog_node.roll_target_speaker = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("Success") {
        dialog_node.success = Some(get_bool_value(attr));
    }
    if let Some(attr) = node.attributes.get("ExcludeCompanionsOptionalBonuses") {
        dialog_node.exclude_companions_optional_bonuses = get_bool_value(attr);
    }
    if let Some(attr) = node.attributes.get("ExcludeSpeakerOptionalBonuses") {
        dialog_node.exclude_speaker_optional_bonuses = get_bool_value(attr);
    }
    if let Some(attr) = node.attributes.get("PersuasionTargetSpeakerIndex") {
        dialog_node.persuasion_target_speaker_index = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("StatName") {
        dialog_node.stat_name = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("StatsAttribute") {
        dialog_node.stats_attribute = Some(get_string_value(attr));
    }

    // Group attributes
    if let Some(attr) = node.attributes.get("GroupID") {
        dialog_node.group_id = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("GroupIndex") {
        dialog_node.group_index = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("Root") {
        dialog_node.root = get_bool_value(attr);
    }

    // Approval
    if let Some(attr) = node.attributes.get("ApprovalRatingID") {
        dialog_node.approval_rating_id = Some(get_string_value(attr));
    }

    // Validated flags
    if let Some(vf_list) = node.children.get("ValidatedFlags") {
        for vf in vf_list {
            if let Some(attr) = vf.attributes.get("ValidatedHasValue") {
                dialog_node.validated_has_value = get_bool_value(attr);
            }
        }
    }

    // Parse children
    if let Some(children_list) = node.children.get("children") {
        for children_container in children_list {
            if let Some(child_nodes) = children_container.children.get("child") {
                for child in child_nodes {
                    if let Some(uuid_attr) = child.attributes.get("UUID") {
                        dialog_node.children.push(get_string_value(uuid_attr));
                    }
                }
            }
        }
    }

    // Parse tags
    if let Some(tags_list) = node.children.get("Tags") {
        for tags_container in tags_list {
            if let Some(tag_nodes) = tags_container.children.get("Tag") {
                for tag_node in tag_nodes {
                    if let Some(tag_attr) = tag_node.attributes.get("Tag") {
                        dialog_node.tags.push(get_string_value(tag_attr));
                    }
                }
            }
        }
    }

    // Parse check flags
    if let Some(cf_list) = node.children.get("checkflags") {
        for cf_container in cf_list {
            if let Some(flag_groups) = cf_container.children.get("flaggroup") {
                for fg in flag_groups {
                    if let Some(flag_group) = parse_flag_group(fg) {
                        dialog_node.check_flags.push(flag_group);
                    }
                }
            }
        }
    }

    // Parse set flags
    if let Some(sf_list) = node.children.get("setflags") {
        for sf_container in sf_list {
            if let Some(flag_groups) = sf_container.children.get("flaggroup") {
                for fg in flag_groups {
                    if let Some(flag_group) = parse_flag_group(fg) {
                        dialog_node.set_flags.push(flag_group);
                    }
                }
            }
        }
    }

    // Parse tagged texts
    if let Some(tt_list) = node.children.get("TaggedTexts") {
        for tt_container in tt_list {
            if let Some(tagged_texts) = tt_container.children.get("TaggedText") {
                for tt in tagged_texts {
                    if let Some(tagged_text) = parse_tagged_text(tt) {
                        dialog_node.tagged_texts.push(tagged_text);
                    }
                }
            }
        }
    }

    // Parse editor data
    if let Some(ed_list) = node.children.get("editorData") {
        for ed_container in ed_list {
            if let Some(data_list) = ed_container.children.get("data") {
                for data in data_list {
                    let key = data.attributes.get("key")
                        .map(get_string_value)
                        .unwrap_or_default();
                    let val = data.attributes.get("val")
                        .map(get_string_value)
                        .unwrap_or_default();
                    if !key.is_empty() {
                        dialog_node.editor_data.insert(key, val);
                    }
                }
            }
        }
    }

    // Parse game data
    if let Some(gd_list) = node.children.get("GameData") {
        for gd in gd_list {
            let mut game_data = GameData::default();

            if let Some(ai_list) = gd.children.get("AiPersonalities") {
                for ai_container in ai_list {
                    if let Some(ai_nodes) = ai_container.children.get("AiPersonality") {
                        for ai in ai_nodes {
                            if let Some(attr) = ai.attributes.get("AiPersonality") {
                                game_data.ai_personalities.push(get_string_value(attr));
                            }
                        }
                    }
                }
            }

            dialog_node.game_data = Some(game_data);
        }
    }

    Some(dialog_node)
}

/// Parse a flag group
fn parse_flag_group(node: &LsjNode) -> Option<FlagGroup> {
    let flag_type = node.attributes.get("type")
        .map(|a| FlagType::from_str(&get_string_value(a)))
        .unwrap_or_default();

    let mut flags = Vec::new();

    if let Some(flag_list) = node.children.get("flag") {
        for flag_node in flag_list {
            let uuid = flag_node.attributes.get("UUID")
                .map(get_string_value)
                .unwrap_or_default();
            let value = flag_node.attributes.get("value")
                .is_some_and(get_bool_value);
            let param_val = flag_node.attributes.get("paramval")
                .map(get_int_value);
            let name = flag_node.attributes.get("name")
                .map(get_string_value);

            flags.push(Flag {
                uuid,
                value,
                param_val,
                name,
            });
        }
    }

    Some(FlagGroup { flag_type, flags })
}

/// Parse tagged text structure
fn parse_tagged_text(node: &LsjNode) -> Option<TaggedText> {
    let has_tag_rule = node.attributes.get("HasTagRule")
        .is_some_and(get_bool_value);

    let mut rule_groups = Vec::new();
    let mut tag_texts = Vec::new();

    // Parse rule groups
    if let Some(rg_list) = node.children.get("RuleGroup") {
        for rg in rg_list {
            let tag_combine_op = rg.attributes.get("TagCombineOp")
                .map_or(0, get_int_value);

            let mut rules = Vec::new();

            if let Some(rules_list) = rg.children.get("Rules") {
                for rules_container in rules_list {
                    if let Some(rule_nodes) = rules_container.children.get("Rule") {
                        for rule_node in rule_nodes {
                            let has_child_rules = rule_node.attributes.get("HasChildRules")
                                .is_some_and(get_bool_value);
                            let rule_combine_op = rule_node.attributes.get("TagCombineOp")
                                .map_or(0, get_int_value);
                            let speaker = rule_node.attributes.get("speaker")
                                .map(get_int_value);

                            let mut rule_tags = Vec::new();
                            if let Some(tags_list) = rule_node.children.get("Tags") {
                                for tags_container in tags_list {
                                    if let Some(tag_nodes) = tags_container.children.get("Tag") {
                                        for tag in tag_nodes {
                                            if let Some(obj_attr) = tag.attributes.get("Object") {
                                                rule_tags.push(get_string_value(obj_attr));
                                            }
                                        }
                                    }
                                }
                            }

                            rules.push(Rule {
                                has_child_rules,
                                tag_combine_op: rule_combine_op,
                                tags: rule_tags,
                                speaker,
                            });
                        }
                    }
                }
            }

            rule_groups.push(RuleGroup {
                tag_combine_op,
                rules,
            });
        }
    }

    // Parse tag texts
    if let Some(tt_list) = node.children.get("TagTexts") {
        for tt_container in tt_list {
            if let Some(text_nodes) = tt_container.children.get("TagText") {
                for text_node in text_nodes {
                    let line_id = text_node.attributes.get("LineId")
                        .map(get_string_value);
                    let stub = text_node.attributes.get("stub")
                        .is_some_and(get_bool_value);

                    // Get the translated string
                    let (handle, value, version) = if let Some(tt_attr) = text_node.attributes.get("TagText") {
                        match tt_attr {
                            LsjAttribute::TranslatedString { handle, value, version, .. } => {
                                (handle.clone(), value.clone(), *version)
                            }
                            LsjAttribute::TranslatedFSString { handle, value, .. } => {
                                (handle.clone(), value.clone(), None)
                            }
                            LsjAttribute::Simple { value, .. } => {
                                (value.as_str().unwrap_or("").to_string(), None, None)
                            }
                        }
                    } else {
                        (String::new(), None, None)
                    };

                    tag_texts.push(TagTextEntry {
                        line_id,
                        handle,
                        value,
                        version,
                        stub,
                    });
                }
            }
        }
    }

    Some(TaggedText {
        has_tag_rule,
        rule_groups,
        tag_texts,
    })
}

/// Parse editor data region
fn parse_editor_data(node: &LsjNode) -> DialogEditorData {
    let mut editor_data = DialogEditorData::default();

    if let Some(attr) = node.attributes.get("HowToTrigger") {
        editor_data.how_to_trigger = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("synopsis") {
        editor_data.synopsis = Some(get_string_value(attr));
    }
    if let Some(attr) = node.attributes.get("nextNodeId") {
        editor_data.next_node_id = Some(get_int_value(attr));
    }
    if let Some(attr) = node.attributes.get("needLayout") {
        editor_data.needs_layout = get_bool_value(attr);
    }

    // Parse default attitudes
    if let Some(da_list) = node.children.get("defaultAttitudes") {
        for da_container in da_list {
            if let Some(data_list) = da_container.children.get("data") {
                for data in data_list {
                    let key = data.attributes.get("key").map(get_string_value).unwrap_or_default();
                    let val = data.attributes.get("val").map(get_string_value).unwrap_or_default();
                    if !key.is_empty() {
                        editor_data.default_attitudes.insert(key, val);
                    }
                }
            }
        }
    }

    // Parse default emotions
    if let Some(de_list) = node.children.get("defaultEmotions") {
        for de_container in de_list {
            if let Some(data_list) = de_container.children.get("data") {
                for data in data_list {
                    let key = data.attributes.get("key").map(get_string_value).unwrap_or_default();
                    let val = data.attributes.get("val").map(get_string_value).unwrap_or_default();
                    if !key.is_empty() {
                        editor_data.default_emotions.insert(key, val);
                    }
                }
            }
        }
    }

    // Parse isPeanut
    if let Some(ip_list) = node.children.get("isPeanuts") {
        for ip_container in ip_list {
            if let Some(data_list) = ip_container.children.get("data") {
                for data in data_list {
                    let key = data.attributes.get("key").map(get_string_value).unwrap_or_default();
                    let val = data.attributes.get("val").map(get_string_value).unwrap_or_default();
                    if !key.is_empty() {
                        editor_data.is_peanut.insert(key, val);
                    }
                }
            }
        }
    }

    editor_data
}

// Helper functions to extract values from LSJ attributes

fn get_string_value(attr: &LsjAttribute) -> String {
    match attr {
        LsjAttribute::Simple { value, .. } => {
            value.as_str().map(std::string::ToString::to_string).unwrap_or_default()
        }
        LsjAttribute::TranslatedString { value, handle, .. } => {
            value.clone().unwrap_or_else(|| handle.clone())
        }
        LsjAttribute::TranslatedFSString { value, handle, .. } => {
            value.clone().unwrap_or_else(|| handle.clone())
        }
    }
}

fn get_int_value(attr: &LsjAttribute) -> i32 {
    match attr {
        LsjAttribute::Simple { value, .. } => {
            value.as_i64().map(|v| v as i32)
                .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn get_float_value(attr: &LsjAttribute) -> f32 {
    match attr {
        LsjAttribute::Simple { value, .. } => {
            value.as_f64().map(|v| v as f32)
                .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
                .unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

fn get_bool_value(attr: &LsjAttribute) -> bool {
    match attr {
        LsjAttribute::Simple { value, .. } => {
            value.as_bool()
                .or_else(|| value.as_str().map(|s| s.eq_ignore_ascii_case("true") || s == "1"))
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// Error type for dialog parsing
#[derive(Debug)]
pub enum DialogParseError {
    MissingRegion(String),
    InvalidFormat(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for DialogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DialogParseError::MissingRegion(r) => write!(f, "Missing region: {r}"),
            DialogParseError::InvalidFormat(s) => write!(f, "Invalid format: {s}"),
            DialogParseError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for DialogParseError {}

impl From<std::io::Error> for DialogParseError {
    fn from(err: std::io::Error) -> Self {
        DialogParseError::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_constructor_from_str() {
        assert_eq!(NodeConstructor::from_str("TagAnswer"), NodeConstructor::TagAnswer);
        assert_eq!(NodeConstructor::from_str("TagQuestion"), NodeConstructor::TagQuestion);
        assert_eq!(NodeConstructor::from_str("ActiveRoll"), NodeConstructor::ActiveRoll);
        assert_eq!(NodeConstructor::from_str("Unknown"), NodeConstructor::Other("Unknown".to_string()));
    }

    #[test]
    fn test_flag_type_from_str() {
        assert_eq!(FlagType::from_str("Local"), FlagType::Local);
        assert_eq!(FlagType::from_str("Global"), FlagType::Global);
        assert_eq!(FlagType::from_str("CustomType"), FlagType::Other("CustomType".to_string()));
    }
}
