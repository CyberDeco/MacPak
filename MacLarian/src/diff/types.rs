//! Core types for diff and merge operations
//!

use std::fmt;

/// Options for diff operations
#[derive(Debug, Clone, Default)]
pub struct DiffOptions {
    /// Ignore whitespace differences in string values
    pub ignore_whitespace: bool,
    /// Ignore version differences in document header
    pub ignore_version: bool,
    /// Compare nodes by key attribute if available (otherwise by position)
    pub match_by_key: bool,
}

/// Options for merge operations
#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    /// Diff options to use when comparing
    pub diff_options: DiffOptions,
    /// Prefer "ours" when there's a conflict (instead of marking as conflict)
    pub prefer_ours: bool,
    /// Prefer "theirs" when there's a conflict
    pub prefer_theirs: bool,
}

/// Type of change detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Item was added
    Added,
    /// Item was removed
    Removed,
    /// Item was modified
    Modified,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added => write!(f, "+"),
            Self::Removed => write!(f, "-"),
            Self::Modified => write!(f, "~"),
        }
    }
}

/// Path to a node in the document tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodePath {
    /// Region ID
    pub region: String,
    /// Path segments (node IDs from root to target)
    pub segments: Vec<String>,
    /// Node key if available
    pub key: Option<String>,
}

impl NodePath {
    pub fn new(region: &str) -> Self {
        Self {
            region: region.to_string(),
            segments: Vec::new(),
            key: None,
        }
    }

    pub fn push(&mut self, segment: &str) {
        self.segments.push(segment.to_string());
    }

    pub fn with_segment(&self, segment: &str) -> Self {
        let mut new = self.clone();
        new.push(segment);
        new
    }

    pub fn with_key(&self, key: Option<&str>) -> Self {
        let mut new = self.clone();
        new.key = key.map(String::from);
        new
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.region)?;
        for seg in &self.segments {
            write!(f, "/{seg}")?;
        }
        if let Some(key) = &self.key {
            write!(f, "[{key}]")?;
        }
        Ok(())
    }
}

/// A change to an attribute
#[derive(Debug, Clone)]
pub struct AttributeChange {
    /// Attribute ID
    pub id: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Old value (for Modified/Removed)
    pub old_value: Option<String>,
    /// New value (for Modified/Added)
    pub new_value: Option<String>,
    /// Old type name (for type changes)
    pub old_type: Option<String>,
    /// New type name (for type changes)
    pub new_type: Option<String>,
}

impl fmt::Display for AttributeChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.change_type {
            ChangeType::Added => {
                write!(
                    f,
                    "+ @{}: {}",
                    self.id,
                    self.new_value.as_deref().unwrap_or("")
                )
            }
            ChangeType::Removed => {
                write!(
                    f,
                    "- @{}: {}",
                    self.id,
                    self.old_value.as_deref().unwrap_or("")
                )
            }
            ChangeType::Modified => {
                write!(
                    f,
                    "~ @{}: {} -> {}",
                    self.id,
                    self.old_value.as_deref().unwrap_or(""),
                    self.new_value.as_deref().unwrap_or("")
                )
            }
        }
    }
}

/// A change to a node
#[derive(Debug, Clone)]
pub struct NodeChange {
    /// Path to the node
    pub path: NodePath,
    /// Type of change
    pub change_type: ChangeType,
    /// Attribute changes (for Modified nodes)
    pub attribute_changes: Vec<AttributeChange>,
    /// Child node changes (for Modified nodes)
    pub child_changes: Vec<NodeChange>,
}

impl NodeChange {
    pub fn added(path: NodePath) -> Self {
        Self {
            path,
            change_type: ChangeType::Added,
            attribute_changes: Vec::new(),
            child_changes: Vec::new(),
        }
    }

    pub fn removed(path: NodePath) -> Self {
        Self {
            path,
            change_type: ChangeType::Removed,
            attribute_changes: Vec::new(),
            child_changes: Vec::new(),
        }
    }

    pub fn modified(path: NodePath) -> Self {
        Self {
            path,
            change_type: ChangeType::Modified,
            attribute_changes: Vec::new(),
            child_changes: Vec::new(),
        }
    }

    /// Check if this change has any actual modifications
    pub fn is_empty(&self) -> bool {
        self.change_type == ChangeType::Modified
            && self.attribute_changes.is_empty()
            && self.child_changes.is_empty()
    }

    /// Count total changes (recursive)
    pub fn change_count(&self) -> usize {
        match self.change_type {
            ChangeType::Added | ChangeType::Removed => 1,
            ChangeType::Modified => {
                self.attribute_changes.len()
                    + self.child_changes.iter().map(Self::change_count).sum::<usize>()
            }
        }
    }
}

impl fmt::Display for NodeChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", self.change_type, self.path)?;
        for attr in &self.attribute_changes {
            writeln!(f, "    {attr}")?;
        }
        for child in &self.child_changes {
            // Indent child changes
            for line in child.to_string().lines() {
                writeln!(f, "  {line}")?;
            }
        }
        Ok(())
    }
}

/// A change to a region
#[derive(Debug, Clone)]
pub struct RegionChange {
    /// Region ID
    pub id: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Node changes within this region
    pub node_changes: Vec<NodeChange>,
}

impl RegionChange {
    /// Check if this region change has any actual modifications
    pub fn is_empty(&self) -> bool {
        self.change_type == ChangeType::Modified && self.node_changes.is_empty()
    }

    /// Count total changes in this region
    pub fn change_count(&self) -> usize {
        match self.change_type {
            ChangeType::Added | ChangeType::Removed => 1,
            ChangeType::Modified => {
                self.node_changes
                    .iter()
                    .map(NodeChange::change_count)
                    .sum()
            }
        }
    }
}

impl fmt::Display for RegionChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.change_type {
            ChangeType::Added => writeln!(f, "+ region: {}", self.id)?,
            ChangeType::Removed => writeln!(f, "- region: {}", self.id)?,
            ChangeType::Modified => writeln!(f, "~ region: {}", self.id)?,
        }
        for node in &self.node_changes {
            write!(f, "{node}")?;
        }
        Ok(())
    }
}

/// High-level change summary
#[derive(Debug, Clone)]
pub enum Change {
    /// Version changed
    Version {
        old: String,
        new: String,
    },
    /// Region change
    Region(RegionChange),
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version { old, new } => write!(f, "~ version: {old} -> {new}"),
            Self::Region(r) => write!(f, "{r}"),
        }
    }
}

/// Result of comparing two documents
#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    /// All detected changes
    pub changes: Vec<Change>,
}

impl DiffResult {
    /// Check if there are no differences
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Count total changes
    pub fn change_count(&self) -> usize {
        self.changes
            .iter()
            .map(|c| match c {
                Change::Version { .. } => 1,
                Change::Region(r) => r.change_count(),
            })
            .sum()
    }

    /// Count regions with changes
    pub fn regions_changed(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, Change::Region(_)))
            .count()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let count = self.change_count();
        let regions = self.regions_changed();
        if count == 0 {
            "No differences".to_string()
        } else {
            format!("{count} change(s) in {regions} region(s)")
        }
    }
}

impl fmt::Display for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.changes.is_empty() {
            writeln!(f, "No differences")?;
        } else {
            for change in &self.changes {
                writeln!(f, "{change}")?;
            }
            writeln!(f, "\n{}", self.summary())?;
        }
        Ok(())
    }
}

/// Type of merge conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Both sides modified the same attribute differently
    AttributeConflict,
    /// Both sides modified the same node's children differently
    ChildConflict,
    /// One side deleted, other side modified
    DeleteModifyConflict,
    /// Both sides added different content at the same location
    AddAddConflict,
}

impl fmt::Display for ConflictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttributeConflict => write!(f, "attribute conflict"),
            Self::ChildConflict => write!(f, "child conflict"),
            Self::DeleteModifyConflict => write!(f, "delete/modify conflict"),
            Self::AddAddConflict => write!(f, "add/add conflict"),
        }
    }
}

/// A merge conflict
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Path to the conflicting element
    pub path: NodePath,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Description of the conflict
    pub description: String,
    /// Value from "ours" (if applicable)
    pub ours: Option<String>,
    /// Value from "theirs" (if applicable)
    pub theirs: Option<String>,
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CONFLICT ({}) at {}", self.conflict_type, self.path)?;
        if !self.description.is_empty() {
            write!(f, ": {}", self.description)?;
        }
        if let (Some(ours), Some(theirs)) = (&self.ours, &self.theirs) {
            write!(f, "\n  ours:   {ours}\n  theirs: {theirs}")?;
        }
        Ok(())
    }
}

/// Result of a three-way merge
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// The merged document (may have conflict markers if conflicts exist)
    pub merged: crate::formats::lsx::LsxDocument,
    /// List of conflicts encountered
    pub conflicts: Vec<Conflict>,
    /// Changes successfully merged from "ours"
    pub ours_applied: usize,
    /// Changes successfully merged from "theirs"
    pub theirs_applied: usize,
}

impl MergeResult {
    /// Check if there are any conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Write the merged document to a file
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write<P: AsRef<std::path::Path>>(&self, path: P) -> crate::Result<()> {
        crate::formats::lsx::write_lsx(&self.merged, path)
    }

    /// Get a summary of the merge
    pub fn summary(&self) -> String {
        if self.conflicts.is_empty() {
            format!(
                "Merge successful: {} from ours, {} from theirs",
                self.ours_applied, self.theirs_applied
            )
        } else {
            format!(
                "Merge with {} conflict(s): {} from ours, {} from theirs",
                self.conflicts.len(),
                self.ours_applied,
                self.theirs_applied
            )
        }
    }
}

impl fmt::Display for MergeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.summary())?;
        if !self.conflicts.is_empty() {
            writeln!(f, "\nConflicts:")?;
            for conflict in &self.conflicts {
                writeln!(f, "  {conflict}")?;
            }
        }
        Ok(())
    }
}
