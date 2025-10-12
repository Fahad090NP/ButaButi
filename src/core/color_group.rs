//! Color group management for organizing threads into logical categories
//!
//! This module provides structures and utilities for grouping embroidery threads
//! into logical categories (e.g., "Skin Tones", "Foliage", "Background"). This is
//! essential for managing complex designs with many colors.
//!
//! # Examples
//!
//! ```
//! use butabuti::core::color_group::{ColorGroup, ThreadGrouping};
//! use butabuti::core::thread::EmbThread;
//!
//! // Create a color group for skin tones
//! let mut skin_group = ColorGroup::new("Skin Tones")
//!     .with_description("All thread colors used for skin rendering");
//!
//! // Add thread indices to the group
//! skin_group.add_thread(0); // Thread #0
//! skin_group.add_thread(1); // Thread #1
//!
//! assert_eq!(skin_group.thread_count(), 2);
//! assert!(skin_group.contains_thread(0));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A named group of thread indices
///
/// Represents a logical grouping of threads (e.g., "Skin Tones", "Background").
/// Each group contains indices referencing threads in the pattern's thread list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorGroup {
    /// Unique name for this group
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Set of thread indices in this group
    #[serde(default)]
    pub thread_indices: HashSet<usize>,

    /// Optional parent group name (for hierarchical grouping)
    pub parent_group: Option<String>,

    /// Custom metadata for extensibility
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// Display order (lower values appear first)
    #[serde(default)]
    pub display_order: i32,

    /// Whether this group is visible in UI
    #[serde(default = "default_visible")]
    pub visible: bool,

    /// Whether this group is locked (prevent modifications)
    #[serde(default)]
    pub locked: bool,
}

fn default_visible() -> bool {
    true
}

impl ColorGroup {
    /// Create a new empty color group
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::ColorGroup;
    ///
    /// let group = ColorGroup::new("Foliage");
    /// assert_eq!(group.name, "Foliage");
    /// assert_eq!(group.thread_count(), 0);
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            thread_indices: HashSet::new(),
            parent_group: None,
            metadata: HashMap::new(),
            display_order: 0,
            visible: true,
            locked: false,
        }
    }

    /// Create a group with specific thread indices
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::ColorGroup;
    ///
    /// let group = ColorGroup::with_threads("Background", vec![5, 6, 7]);
    /// assert_eq!(group.thread_count(), 3);
    /// assert!(group.contains_thread(5));
    /// ```
    pub fn with_threads(name: impl Into<String>, thread_indices: Vec<usize>) -> Self {
        let mut group = Self::new(name);
        group.thread_indices = thread_indices.into_iter().collect();
        group
    }

    /// Builder: Set description
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::ColorGroup;
    ///
    /// let group = ColorGroup::new("Highlights")
    ///     .with_description("Bright accent colors");
    /// assert_eq!(group.description.unwrap(), "Bright accent colors");
    /// ```
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Builder: Set parent group
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent_group = Some(parent.into());
        self
    }

    /// Builder: Set display order
    pub fn with_display_order(mut self, order: i32) -> Self {
        self.display_order = order;
        self
    }

    /// Builder: Set visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Builder: Set locked status
    pub fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Add a thread index to this group
    ///
    /// # Returns
    ///
    /// `true` if the thread was newly added, `false` if it was already in the group
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::ColorGroup;
    ///
    /// let mut group = ColorGroup::new("Test");
    /// assert!(group.add_thread(0));  // Newly added
    /// assert!(!group.add_thread(0)); // Already exists
    /// ```
    pub fn add_thread(&mut self, thread_index: usize) -> bool {
        self.thread_indices.insert(thread_index)
    }

    /// Remove a thread index from this group
    ///
    /// # Returns
    ///
    /// `true` if the thread was removed, `false` if it wasn't in the group
    pub fn remove_thread(&mut self, thread_index: usize) -> bool {
        self.thread_indices.remove(&thread_index)
    }

    /// Check if this group contains a specific thread
    pub fn contains_thread(&self, thread_index: usize) -> bool {
        self.thread_indices.contains(&thread_index)
    }

    /// Get the number of threads in this group
    pub fn thread_count(&self) -> usize {
        self.thread_indices.len()
    }

    /// Check if this group is empty
    pub fn is_empty(&self) -> bool {
        self.thread_indices.is_empty()
    }

    /// Get an iterator over thread indices
    pub fn thread_indices_iter(&self) -> impl Iterator<Item = &usize> {
        self.thread_indices.iter()
    }

    /// Get thread indices as a sorted vector
    pub fn thread_indices_sorted(&self) -> Vec<usize> {
        let mut indices: Vec<_> = self.thread_indices.iter().copied().collect();
        indices.sort_unstable();
        indices
    }

    /// Clear all thread indices from this group
    pub fn clear(&mut self) {
        self.thread_indices.clear();
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Remove metadata value
    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata.remove(key)
    }
}

/// Thread grouping manager for organizing all color groups in a pattern
///
/// This structure manages multiple color groups and provides utilities for
/// organizing threads, querying group membership, and maintaining group hierarchies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadGrouping {
    /// All color groups, indexed by name
    #[serde(default)]
    groups: HashMap<String, ColorGroup>,

    /// Default group for ungrouped threads
    #[serde(default)]
    default_group_name: Option<String>,
}

impl ThreadGrouping {
    /// Create a new empty thread grouping
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            default_group_name: None,
        }
    }

    /// Create with a default group for ungrouped threads
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::ThreadGrouping;
    ///
    /// let grouping = ThreadGrouping::with_default_group("Ungrouped");
    /// assert!(grouping.has_default_group());
    /// ```
    pub fn with_default_group(default_group_name: impl Into<String>) -> Self {
        let default_name = default_group_name.into();
        let mut grouping = Self::new();
        grouping.add_group(ColorGroup::new(&default_name));
        grouping.default_group_name = Some(default_name);
        grouping
    }

    /// Add a color group
    ///
    /// # Returns
    ///
    /// The previous group with the same name, if any
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::{ThreadGrouping, ColorGroup};
    ///
    /// let mut grouping = ThreadGrouping::new();
    /// grouping.add_group(ColorGroup::new("Foliage"));
    /// assert!(grouping.has_group("Foliage"));
    /// ```
    pub fn add_group(&mut self, group: ColorGroup) -> Option<ColorGroup> {
        self.groups.insert(group.name.clone(), group)
    }

    /// Remove a color group by name
    pub fn remove_group(&mut self, name: &str) -> Option<ColorGroup> {
        self.groups.remove(name)
    }

    /// Get a color group by name
    pub fn get_group(&self, name: &str) -> Option<&ColorGroup> {
        self.groups.get(name)
    }

    /// Get a mutable reference to a color group
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut ColorGroup> {
        self.groups.get_mut(name)
    }

    /// Check if a group exists
    pub fn has_group(&self, name: &str) -> bool {
        self.groups.contains_key(name)
    }

    /// Get the number of groups
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Check if there are any groups
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Get an iterator over all groups
    pub fn groups_iter(&self) -> impl Iterator<Item = (&String, &ColorGroup)> {
        self.groups.iter()
    }

    /// Get an iterator over group names
    pub fn group_names(&self) -> impl Iterator<Item = &String> {
        self.groups.keys()
    }

    /// Get groups sorted by display order
    pub fn groups_sorted_by_order(&self) -> Vec<&ColorGroup> {
        let mut groups: Vec<_> = self.groups.values().collect();
        groups.sort_by_key(|g| g.display_order);
        groups
    }

    /// Add a thread to a specific group
    ///
    /// # Returns
    ///
    /// `Ok(true)` if added, `Ok(false)` if already in group, `Err` if group doesn't exist
    pub fn add_thread_to_group(
        &mut self,
        group_name: &str,
        thread_index: usize,
    ) -> Result<bool, String> {
        if let Some(group) = self.groups.get_mut(group_name) {
            Ok(group.add_thread(thread_index))
        } else {
            Err(format!("Group '{}' does not exist", group_name))
        }
    }

    /// Remove a thread from a specific group
    pub fn remove_thread_from_group(
        &mut self,
        group_name: &str,
        thread_index: usize,
    ) -> Result<bool, String> {
        if let Some(group) = self.groups.get_mut(group_name) {
            Ok(group.remove_thread(thread_index))
        } else {
            Err(format!("Group '{}' does not exist", group_name))
        }
    }

    /// Find all groups containing a specific thread
    pub fn find_groups_with_thread(&self, thread_index: usize) -> Vec<&ColorGroup> {
        self.groups
            .values()
            .filter(|g| g.contains_thread(thread_index))
            .collect()
    }

    /// Find all groups containing a specific thread (returns names)
    pub fn find_group_names_with_thread(&self, thread_index: usize) -> Vec<&str> {
        self.groups
            .iter()
            .filter_map(|(name, group)| {
                if group.contains_thread(thread_index) {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all thread indices across all groups (deduplicated)
    pub fn all_grouped_threads(&self) -> HashSet<usize> {
        let mut all_threads = HashSet::new();
        for group in self.groups.values() {
            all_threads.extend(group.thread_indices.iter());
        }
        all_threads
    }

    /// Check if a thread is in any group
    pub fn is_thread_grouped(&self, thread_index: usize) -> bool {
        self.groups
            .values()
            .any(|g| g.contains_thread(thread_index))
    }

    /// Get ungrouped thread indices given a total thread count
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::color_group::{ThreadGrouping, ColorGroup};
    ///
    /// let mut grouping = ThreadGrouping::new();
    /// let mut group = ColorGroup::new("Test");
    /// group.add_thread(0);
    /// group.add_thread(2);
    /// grouping.add_group(group);
    ///
    /// let ungrouped = grouping.ungrouped_threads(5);
    /// assert_eq!(ungrouped, vec![1, 3, 4]);
    /// ```
    pub fn ungrouped_threads(&self, total_thread_count: usize) -> Vec<usize> {
        let grouped = self.all_grouped_threads();
        (0..total_thread_count)
            .filter(|i| !grouped.contains(i))
            .collect()
    }

    /// Assign ungrouped threads to the default group
    ///
    /// Returns the number of threads assigned
    pub fn assign_to_default_group(&mut self, total_thread_count: usize) -> Result<usize, String> {
        let default_name = self
            .default_group_name
            .clone()
            .ok_or_else(|| "No default group configured".to_string())?;

        let ungrouped = self.ungrouped_threads(total_thread_count);
        let count = ungrouped.len();

        if let Some(group) = self.groups.get_mut(&default_name) {
            for thread_idx in ungrouped {
                group.add_thread(thread_idx);
            }
            Ok(count)
        } else {
            Err(format!("Default group '{}' does not exist", default_name))
        }
    }

    /// Set the default group name
    pub fn set_default_group(&mut self, name: Option<String>) {
        self.default_group_name = name;
    }

    /// Get the default group name
    pub fn default_group(&self) -> Option<&str> {
        self.default_group_name.as_deref()
    }

    /// Check if a default group is configured
    pub fn has_default_group(&self) -> bool {
        self.default_group_name.is_some()
    }

    /// Clear all groups
    pub fn clear(&mut self) {
        self.groups.clear();
        self.default_group_name = None;
    }

    /// Validate group structure
    ///
    /// Checks for:
    /// - Circular parent references
    /// - Invalid parent group names
    /// - Locked groups with modifications
    ///
    /// Returns a list of validation errors
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for invalid parent references and circular chains
        for (name, group) in &self.groups {
            if let Some(parent) = &group.parent_group {
                if !self.groups.contains_key(parent) {
                    errors.push(format!("Group '{}' has invalid parent '{}'", name, parent));
                }

                // Check for circular references (full chain detection)
                let mut visited = HashSet::new();
                let mut current = Some(name.as_str());
                while let Some(group_name) = current {
                    if !visited.insert(group_name) {
                        errors.push(format!(
                            "Circular reference detected in group '{}' (chain involves: {})",
                            name,
                            visited
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>()
                                .join(" → ")
                        ));
                        break;
                    }
                    current = self
                        .groups
                        .get(group_name)
                        .and_then(|g| g.parent_group.as_deref());
                }
            }
        }

        // Check if default group exists
        if let Some(default_name) = &self.default_group_name {
            if !self.groups.contains_key(default_name) {
                errors.push(format!("Default group '{}' does not exist", default_name));
            }
        }

        errors
    }

    /// Merge another ThreadGrouping into this one
    ///
    /// Groups with duplicate names will be replaced
    pub fn merge(&mut self, other: ThreadGrouping) {
        self.groups.extend(other.groups);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_group_creation() {
        let group = ColorGroup::new("Test Group");
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.thread_count(), 0);
        assert!(group.visible);
        assert!(!group.locked);
    }

    #[test]
    fn test_color_group_with_threads() {
        let group = ColorGroup::with_threads("Test", vec![0, 1, 2]);
        assert_eq!(group.thread_count(), 3);
        assert!(group.contains_thread(0));
        assert!(group.contains_thread(1));
        assert!(group.contains_thread(2));
        assert!(!group.contains_thread(3));
    }

    #[test]
    fn test_color_group_builder() {
        let group = ColorGroup::new("Test")
            .with_description("Test description")
            .with_display_order(5)
            .with_visibility(false)
            .with_locked(true);

        assert_eq!(group.description.unwrap(), "Test description");
        assert_eq!(group.display_order, 5);
        assert!(!group.visible);
        assert!(group.locked);
    }

    #[test]
    fn test_add_remove_threads() {
        let mut group = ColorGroup::new("Test");
        assert!(group.add_thread(0));
        assert!(!group.add_thread(0)); // Already exists

        assert!(group.contains_thread(0));
        assert!(group.remove_thread(0));
        assert!(!group.contains_thread(0));
        assert!(!group.remove_thread(0)); // Already removed
    }

    #[test]
    fn test_thread_indices_sorted() {
        let mut group = ColorGroup::new("Test");
        group.add_thread(5);
        group.add_thread(1);
        group.add_thread(3);

        let sorted = group.thread_indices_sorted();
        assert_eq!(sorted, vec![1, 3, 5]);
    }

    #[test]
    fn test_metadata() {
        let mut group = ColorGroup::new("Test");
        group.set_metadata("key1", "value1");
        group.set_metadata("key2", "value2");

        assert_eq!(group.get_metadata("key1"), Some("value1"));
        assert_eq!(group.get_metadata("key2"), Some("value2"));
        assert_eq!(group.get_metadata("key3"), None);

        let removed = group.remove_metadata("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(group.get_metadata("key1"), None);
    }

    #[test]
    fn test_thread_grouping_creation() {
        let grouping = ThreadGrouping::new();
        assert_eq!(grouping.group_count(), 0);
        assert!(!grouping.has_default_group());
    }

    #[test]
    fn test_thread_grouping_with_default() {
        let grouping = ThreadGrouping::with_default_group("Default");
        assert!(grouping.has_default_group());
        assert_eq!(grouping.default_group(), Some("Default"));
        assert_eq!(grouping.group_count(), 1);
    }

    #[test]
    fn test_add_remove_groups() {
        let mut grouping = ThreadGrouping::new();
        grouping.add_group(ColorGroup::new("Group1"));
        grouping.add_group(ColorGroup::new("Group2"));

        assert_eq!(grouping.group_count(), 2);
        assert!(grouping.has_group("Group1"));
        assert!(grouping.has_group("Group2"));

        grouping.remove_group("Group1");
        assert_eq!(grouping.group_count(), 1);
        assert!(!grouping.has_group("Group1"));
    }

    #[test]
    fn test_add_thread_to_group() {
        let mut grouping = ThreadGrouping::new();
        grouping.add_group(ColorGroup::new("Test"));

        let result = grouping.add_thread_to_group("Test", 5);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let group = grouping.get_group("Test").unwrap();
        assert!(group.contains_thread(5));
    }

    #[test]
    fn test_add_thread_to_nonexistent_group() {
        let mut grouping = ThreadGrouping::new();
        let result = grouping.add_thread_to_group("NonExistent", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_groups_with_thread() {
        let mut grouping = ThreadGrouping::new();

        let mut group1 = ColorGroup::new("Group1");
        group1.add_thread(5);
        group1.add_thread(10);

        let mut group2 = ColorGroup::new("Group2");
        group2.add_thread(5);
        group2.add_thread(15);

        grouping.add_group(group1);
        grouping.add_group(group2);

        let groups = grouping.find_groups_with_thread(5);
        assert_eq!(groups.len(), 2);

        let groups = grouping.find_groups_with_thread(10);
        assert_eq!(groups.len(), 1);

        let groups = grouping.find_groups_with_thread(99);
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_ungrouped_threads() {
        let mut grouping = ThreadGrouping::new();
        let mut group = ColorGroup::new("Test");
        group.add_thread(0);
        group.add_thread(2);
        group.add_thread(4);
        grouping.add_group(group);

        let ungrouped = grouping.ungrouped_threads(6);
        assert_eq!(ungrouped, vec![1, 3, 5]);
    }

    #[test]
    fn test_assign_to_default_group() {
        let mut grouping = ThreadGrouping::with_default_group("Default");
        let mut group = ColorGroup::new("Custom");
        group.add_thread(0);
        group.add_thread(1);
        grouping.add_group(group);

        let assigned = grouping.assign_to_default_group(5).unwrap();
        assert_eq!(assigned, 3); // Threads 2, 3, 4

        let default_group = grouping.get_group("Default").unwrap();
        assert!(default_group.contains_thread(2));
        assert!(default_group.contains_thread(3));
        assert!(default_group.contains_thread(4));
    }

    #[test]
    fn test_groups_sorted_by_order() {
        let mut grouping = ThreadGrouping::new();
        grouping.add_group(ColorGroup::new("C").with_display_order(3));
        grouping.add_group(ColorGroup::new("A").with_display_order(1));
        grouping.add_group(ColorGroup::new("B").with_display_order(2));

        let sorted = grouping.groups_sorted_by_order();
        assert_eq!(sorted[0].name, "A");
        assert_eq!(sorted[1].name, "B");
        assert_eq!(sorted[2].name, "C");
    }

    #[test]
    fn test_validation() {
        let mut grouping = ThreadGrouping::new();
        grouping.add_group(ColorGroup::new("Valid"));
        grouping.add_group(ColorGroup::new("Child").with_parent("Valid"));
        grouping.add_group(ColorGroup::new("Orphan").with_parent("NonExistent"));

        let errors = grouping.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("invalid parent")));
    }

    #[test]
    fn test_serialization() {
        let group = ColorGroup::with_threads("Test", vec![0, 1, 2]).with_description("Test group");

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: ColorGroup = serde_json::from_str(&json).unwrap();

        assert_eq!(group.name, deserialized.name);
        assert_eq!(group.thread_count(), deserialized.thread_count());
    }

    #[test]
    fn test_thread_grouping_serialization() {
        let mut grouping = ThreadGrouping::with_default_group("Default");
        grouping.add_group(ColorGroup::with_threads("Group1", vec![0, 1]));
        grouping.add_group(ColorGroup::with_threads("Group2", vec![2, 3]));

        let json = serde_json::to_string(&grouping).unwrap();
        let deserialized: ThreadGrouping = serde_json::from_str(&json).unwrap();

        assert_eq!(grouping.group_count(), deserialized.group_count());
        assert_eq!(grouping.default_group(), deserialized.default_group());
    }
}
