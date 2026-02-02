//! LOCA editing operations
//!
//! Provides functions for modifying LOCA resources:
//! - Add, update, delete entries
//! - Bulk find-and-replace
//! - Merge resources

use super::{LocalizedText, LocaResource};

/// Result of an edit operation
#[derive(Debug, Clone)]
pub struct EditResult {
    /// Number of entries affected
    pub affected: usize,
    /// Details about what changed
    pub details: Vec<String>,
}

impl EditResult {
    /// Create a new edit result
    #[must_use]
    pub fn new(affected: usize) -> Self {
        Self {
            affected,
            details: Vec::new(),
        }
    }

    /// Add a detail message
    pub fn add_detail(&mut self, detail: impl Into<String>) {
        self.details.push(detail.into());
    }
}

/// Result of a bulk replace operation
#[derive(Debug, Clone)]
pub struct ReplaceResult {
    /// Number of entries modified
    pub entries_modified: usize,
    /// Total number of replacements made
    pub replacements: usize,
    /// Keys of modified entries
    pub modified_keys: Vec<String>,
}

// ============================================================================
// LocaResource editing methods
// ============================================================================

impl LocaResource {
    /// Add a new entry to the resource
    ///
    /// If an entry with the same key already exists, it will be updated.
    ///
    /// # Returns
    /// `true` if a new entry was added, `false` if an existing entry was updated
    pub fn add_entry(&mut self, key: impl Into<String>, text: impl Into<String>) -> bool {
        self.add_entry_with_version(key, text, 1)
    }

    /// Add a new entry with a specific version
    ///
    /// If an entry with the same key already exists, it will be updated.
    ///
    /// # Returns
    /// `true` if a new entry was added, `false` if an existing entry was updated
    pub fn add_entry_with_version(
        &mut self,
        key: impl Into<String>,
        text: impl Into<String>,
        version: u16,
    ) -> bool {
        let key = key.into();
        let text = text.into();

        // Check if entry already exists
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.text = text;
            entry.version = version;
            false
        } else {
            self.entries.push(LocalizedText { key, version, text });
            true
        }
    }

    /// Update an existing entry's text
    ///
    /// # Returns
    /// `true` if the entry was found and updated, `false` if not found
    pub fn update_entry(&mut self, key: &str, new_text: impl Into<String>) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.text = new_text.into();
            true
        } else {
            false
        }
    }

    /// Update an existing entry's text and increment its version
    ///
    /// # Returns
    /// `true` if the entry was found and updated, `false` if not found
    pub fn update_entry_bump_version(&mut self, key: &str, new_text: impl Into<String>) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.text = new_text.into();
            entry.version = entry.version.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Delete an entry by key
    ///
    /// # Returns
    /// The removed entry if found, `None` otherwise
    pub fn delete_entry(&mut self, key: &str) -> Option<LocalizedText> {
        if let Some(pos) = self.entries.iter().position(|e| e.key == key) {
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    /// Get an entry by key
    #[must_use]
    pub fn get_entry(&self, key: &str) -> Option<&LocalizedText> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// Get a mutable reference to an entry by key
    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut LocalizedText> {
        self.entries.iter_mut().find(|e| e.key == key)
    }

    /// Check if an entry with the given key exists
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.key == key)
    }

    /// Get the number of entries
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the resource is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Bulk replace text in all entries
    ///
    /// Replaces all occurrences of `find` with `replace` in all entry texts.
    ///
    /// # Arguments
    /// * `find` - Text to search for
    /// * `replace` - Text to replace with
    /// * `case_sensitive` - Whether to match case
    ///
    /// # Returns
    /// Result with statistics about the replacement
    pub fn replace_all(
        &mut self,
        find: &str,
        replace: &str,
        case_sensitive: bool,
    ) -> ReplaceResult {
        let mut entries_modified = 0;
        let mut replacements = 0;
        let mut modified_keys = Vec::new();

        for entry in &mut self.entries {
            let (new_text, count) = if case_sensitive {
                replace_counting(&entry.text, find, replace)
            } else {
                replace_case_insensitive_counting(&entry.text, find, replace)
            };

            if count > 0 {
                entry.text = new_text;
                entries_modified += 1;
                replacements += count;
                modified_keys.push(entry.key.clone());
            }
        }

        ReplaceResult {
            entries_modified,
            replacements,
            modified_keys,
        }
    }

    /// Replace text in entries matching a key pattern
    ///
    /// # Arguments
    /// * `key_pattern` - Substring to match in entry keys
    /// * `find` - Text to search for in entry text
    /// * `replace` - Text to replace with
    /// * `case_sensitive` - Whether to match case for text replacement
    ///
    /// # Returns
    /// Result with statistics about the replacement
    pub fn replace_in_matching(
        &mut self,
        key_pattern: &str,
        find: &str,
        replace: &str,
        case_sensitive: bool,
    ) -> ReplaceResult {
        let mut entries_modified = 0;
        let mut replacements = 0;
        let mut modified_keys = Vec::new();

        for entry in &mut self.entries {
            // Check if key matches pattern
            if !entry.key.contains(key_pattern) {
                continue;
            }

            let (new_text, count) = if case_sensitive {
                replace_counting(&entry.text, find, replace)
            } else {
                replace_case_insensitive_counting(&entry.text, find, replace)
            };

            if count > 0 {
                entry.text = new_text;
                entries_modified += 1;
                replacements += count;
                modified_keys.push(entry.key.clone());
            }
        }

        ReplaceResult {
            entries_modified,
            replacements,
            modified_keys,
        }
    }

    /// Merge entries from another resource
    ///
    /// # Arguments
    /// * `other` - Resource to merge from
    /// * `overwrite` - If true, existing entries will be overwritten
    ///
    /// # Returns
    /// Edit result with statistics
    pub fn merge(&mut self, other: &LocaResource, overwrite: bool) -> EditResult {
        let mut result = EditResult::new(0);

        for entry in &other.entries {
            if let Some(existing) = self.entries.iter_mut().find(|e| e.key == entry.key) {
                if overwrite {
                    existing.text = entry.text.clone();
                    existing.version = entry.version;
                    result.affected += 1;
                    result.add_detail(format!("Updated: {}", entry.key));
                }
            } else {
                self.entries.push(entry.clone());
                result.affected += 1;
                result.add_detail(format!("Added: {}", entry.key));
            }
        }

        result
    }

    /// Sort entries by key
    pub fn sort_by_key(&mut self) {
        self.entries.sort_by(|a, b| a.key.cmp(&b.key));
    }

    /// Remove duplicate entries (keeps first occurrence)
    ///
    /// # Returns
    /// Number of duplicates removed
    pub fn remove_duplicates(&mut self) -> usize {
        let original_len = self.entries.len();
        let mut seen = std::collections::HashSet::new();
        self.entries.retain(|e| seen.insert(e.key.clone()));
        original_len - self.entries.len()
    }

    /// Find entries containing text
    ///
    /// # Arguments
    /// * `text` - Text to search for
    /// * `case_sensitive` - Whether to match case
    ///
    /// # Returns
    /// References to matching entries
    #[must_use]
    pub fn find_by_text(&self, text: &str, case_sensitive: bool) -> Vec<&LocalizedText> {
        if case_sensitive {
            self.entries.iter().filter(|e| e.text.contains(text)).collect()
        } else {
            let text_lower = text.to_lowercase();
            self.entries
                .iter()
                .filter(|e| e.text.to_lowercase().contains(&text_lower))
                .collect()
        }
    }

    /// Find entries with keys matching a pattern
    ///
    /// # Arguments
    /// * `pattern` - Substring to match in keys
    ///
    /// # Returns
    /// References to matching entries
    #[must_use]
    pub fn find_by_key_pattern(&self, pattern: &str) -> Vec<&LocalizedText> {
        self.entries
            .iter()
            .filter(|e| e.key.contains(pattern))
            .collect()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Replace all occurrences and count them
fn replace_counting(text: &str, find: &str, replace: &str) -> (String, usize) {
    let count = text.matches(find).count();
    let new_text = text.replace(find, replace);
    (new_text, count)
}

/// Case-insensitive replace with counting
fn replace_case_insensitive_counting(text: &str, find: &str, replace: &str) -> (String, usize) {
    let find_lower = find.to_lowercase();
    let text_lower = text.to_lowercase();

    // Count occurrences
    let count = text_lower.matches(&find_lower).count();

    if count == 0 {
        return (text.to_string(), 0);
    }

    // Build result preserving non-matching case
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&find_lower) {
        result.push_str(&text[last_end..start]);
        result.push_str(replace);
        last_end = start + find.len();
    }
    result.push_str(&text[last_end..]);

    (result, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut resource = LocaResource::new();
        assert!(resource.add_entry("key1", "text1"));
        assert_eq!(resource.len(), 1);

        // Adding same key should update
        assert!(!resource.add_entry("key1", "updated"));
        assert_eq!(resource.len(), 1);
        assert_eq!(resource.get_entry("key1").unwrap().text, "updated");
    }

    #[test]
    fn test_delete_entry() {
        let mut resource = LocaResource::new();
        resource.add_entry("key1", "text1");
        resource.add_entry("key2", "text2");

        let removed = resource.delete_entry("key1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().key, "key1");
        assert_eq!(resource.len(), 1);
        assert!(!resource.contains_key("key1"));
    }

    #[test]
    fn test_replace_all() {
        let mut resource = LocaResource::new();
        resource.add_entry("key1", "Hello world");
        resource.add_entry("key2", "Hello there");
        resource.add_entry("key3", "Goodbye");

        let result = resource.replace_all("Hello", "Hi", true);
        assert_eq!(result.entries_modified, 2);
        assert_eq!(result.replacements, 2);
        assert_eq!(resource.get_entry("key1").unwrap().text, "Hi world");
        assert_eq!(resource.get_entry("key2").unwrap().text, "Hi there");
    }

    #[test]
    fn test_replace_case_insensitive() {
        let mut resource = LocaResource::new();
        resource.add_entry("key1", "HELLO world");
        resource.add_entry("key2", "hello there");

        let result = resource.replace_all("hello", "hi", false);
        assert_eq!(result.entries_modified, 2);
        assert_eq!(resource.get_entry("key1").unwrap().text, "hi world");
        assert_eq!(resource.get_entry("key2").unwrap().text, "hi there");
    }
}
