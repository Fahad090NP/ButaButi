//! Pattern collection for multi-pattern embroidery files
//!
//! Some embroidery formats (like HUS and VP3) support multiple patterns/designs
//! within a single file. This module provides a collection structure to manage
//! multiple patterns with named access.

use crate::core::pattern::EmbPattern;
use std::collections::HashMap;

/// A collection of named embroidery patterns
///
/// Used for formats that support multiple patterns in a single file,
/// such as HUS (Husqvarna Viking) and VP3 (Pfaff) formats.
///
/// # Example
///
/// ```
/// use butabuti::core::collection::EmbPatternCollection;
/// use butabuti::core::pattern::EmbPattern;
///
/// let mut collection = EmbPatternCollection::new();
///
/// // Add patterns to collection
/// let mut pattern1 = EmbPattern::new();
/// pattern1.stitch_abs(10.0, 10.0);
/// collection.add("design1".to_string(), pattern1);
///
/// let mut pattern2 = EmbPattern::new();
/// pattern2.stitch_abs(20.0, 20.0);
/// collection.add("design2".to_string(), pattern2);
///
/// // Access patterns
/// assert_eq!(collection.len(), 2);
/// assert!(collection.get("design1").is_some());
/// assert!(collection.get("design2").is_some());
///
/// // Iterate over patterns
/// for (name, pattern) in collection.iter() {
///     println!("Pattern '{}' has {} stitches", name, pattern.count_stitches());
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct EmbPatternCollection {
    /// Named patterns in the collection
    patterns: HashMap<String, EmbPattern>,
}

impl EmbPatternCollection {
    /// Create a new empty pattern collection
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    ///
    /// let collection = EmbPatternCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Create a collection with specified capacity
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    ///
    /// let collection = EmbPatternCollection::with_capacity(10);
    /// assert!(collection.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            patterns: HashMap::with_capacity(capacity),
        }
    }

    /// Add a pattern to the collection with a given name
    ///
    /// If a pattern with the same name already exists, it will be replaced
    /// and the old pattern will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// let pattern = EmbPattern::new();
    ///
    /// let old = collection.add("design1".to_string(), pattern);
    /// assert!(old.is_none()); // First time adding
    ///
    /// let pattern2 = EmbPattern::new();
    /// let old = collection.add("design1".to_string(), pattern2);
    /// assert!(old.is_some()); // Replaced existing pattern
    /// ```
    pub fn add(&mut self, name: String, pattern: EmbPattern) -> Option<EmbPattern> {
        self.patterns.insert(name, pattern)
    }

    /// Get a reference to a pattern by name
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// assert!(collection.get("design1").is_some());
    /// assert!(collection.get("nonexistent").is_none());
    /// ```
    pub fn get(&self, name: &str) -> Option<&EmbPattern> {
        self.patterns.get(name)
    }

    /// Get a mutable reference to a pattern by name
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// if let Some(pattern) = collection.get_mut("design1") {
    ///     pattern.stitch_abs(10.0, 10.0);
    /// }
    ///
    /// assert_eq!(collection.get("design1").unwrap().count_stitches(), 1);
    /// ```
    pub fn get_mut(&mut self, name: &str) -> Option<&mut EmbPattern> {
        self.patterns.get_mut(name)
    }

    /// Remove a pattern from the collection by name
    ///
    /// Returns the removed pattern if it existed.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// assert_eq!(collection.len(), 1);
    /// let removed = collection.remove("design1");
    /// assert!(removed.is_some());
    /// assert_eq!(collection.len(), 0);
    /// ```
    pub fn remove(&mut self, name: &str) -> Option<EmbPattern> {
        self.patterns.remove(name)
    }

    /// Check if the collection contains a pattern with the given name
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// assert!(collection.contains("design1"));
    /// assert!(!collection.contains("design2"));
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.patterns.contains_key(name)
    }

    /// Get an iterator over the pattern names and references
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    /// collection.add("design2".to_string(), EmbPattern::new());
    ///
    /// let mut count = 0;
    /// for (name, pattern) in collection.iter() {
    ///     println!("Pattern: {}", name);
    ///     count += 1;
    /// }
    /// assert_eq!(count, 2);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&String, &EmbPattern)> {
        self.patterns.iter()
    }

    /// Get a mutable iterator over the pattern names and references
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// for (name, pattern) in collection.iter_mut() {
    ///     pattern.stitch_abs(10.0, 10.0);
    /// }
    ///
    /// assert_eq!(collection.get("design1").unwrap().count_stitches(), 1);
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut EmbPattern)> {
        self.patterns.iter_mut()
    }

    /// Get an iterator over pattern names
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    /// collection.add("design2".to_string(), EmbPattern::new());
    ///
    /// let names: Vec<_> = collection.names().collect();
    /// assert_eq!(names.len(), 2);
    /// ```
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.patterns.keys()
    }

    /// Get an iterator over pattern references
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// for pattern in collection.patterns() {
    ///     println!("Pattern has {} stitches", pattern.count_stitches());
    /// }
    /// ```
    pub fn patterns(&self) -> impl Iterator<Item = &EmbPattern> {
        self.patterns.values()
    }

    /// Get the number of patterns in the collection
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// assert_eq!(collection.len(), 0);
    ///
    /// collection.add("design1".to_string(), EmbPattern::new());
    /// assert_eq!(collection.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if the collection is empty
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    ///
    /// let collection = EmbPatternCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Clear all patterns from the collection
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection = EmbPatternCollection::new();
    /// collection.add("design1".to_string(), EmbPattern::new());
    ///
    /// assert_eq!(collection.len(), 1);
    /// collection.clear();
    /// assert_eq!(collection.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.patterns.clear();
    }

    /// Merge another collection into this one
    ///
    /// Patterns from the other collection will be added to this one.
    /// If there are name conflicts, patterns from the other collection
    /// will replace those in this collection.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::collection::EmbPatternCollection;
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut collection1 = EmbPatternCollection::new();
    /// collection1.add("design1".to_string(), EmbPattern::new());
    ///
    /// let mut collection2 = EmbPatternCollection::new();
    /// collection2.add("design2".to_string(), EmbPattern::new());
    ///
    /// collection1.merge(collection2);
    /// assert_eq!(collection1.len(), 2);
    /// ```
    pub fn merge(&mut self, other: Self) {
        self.patterns.extend(other.patterns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collection_is_empty() {
        let collection = EmbPatternCollection::new();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_add_pattern() {
        let mut collection = EmbPatternCollection::new();
        let pattern = EmbPattern::new();

        let old = collection.add("design1".to_string(), pattern);
        assert!(old.is_none());
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn test_add_replaces_existing() {
        let mut collection = EmbPatternCollection::new();
        let mut pattern1 = EmbPattern::new();
        pattern1.stitch_abs(10.0, 10.0);

        collection.add("design1".to_string(), pattern1);

        let mut pattern2 = EmbPattern::new();
        pattern2.stitch_abs(20.0, 20.0);
        pattern2.stitch_abs(30.0, 30.0);

        let old = collection.add("design1".to_string(), pattern2);
        assert!(old.is_some());
        assert_eq!(old.unwrap().count_stitches(), 1);
        assert_eq!(collection.get("design1").unwrap().count_stitches(), 2);
    }

    #[test]
    fn test_get_pattern() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());

        assert!(collection.get("design1").is_some());
        assert!(collection.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_mut_pattern() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());

        if let Some(pattern) = collection.get_mut("design1") {
            pattern.stitch_abs(10.0, 10.0);
        }

        assert_eq!(collection.get("design1").unwrap().count_stitches(), 1);
    }

    #[test]
    fn test_remove_pattern() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());

        assert_eq!(collection.len(), 1);
        let removed = collection.remove("design1");
        assert!(removed.is_some());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_contains() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());

        assert!(collection.contains("design1"));
        assert!(!collection.contains("design2"));
    }

    #[test]
    fn test_iter() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());
        collection.add("design2".to_string(), EmbPattern::new());

        let mut count = 0;
        for (_name, _pattern) in collection.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_iter_mut() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());
        collection.add("design2".to_string(), EmbPattern::new());

        for (_name, pattern) in collection.iter_mut() {
            pattern.stitch_abs(10.0, 10.0);
        }

        for (_name, pattern) in collection.iter() {
            assert_eq!(pattern.count_stitches(), 1);
        }
    }

    #[test]
    fn test_names() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());
        collection.add("design2".to_string(), EmbPattern::new());

        let names: Vec<_> = collection.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"design1".to_string()));
        assert!(names.contains(&&"design2".to_string()));
    }

    #[test]
    fn test_patterns() {
        let mut collection = EmbPatternCollection::new();
        let mut pattern1 = EmbPattern::new();
        pattern1.stitch_abs(10.0, 10.0);
        let mut pattern2 = EmbPattern::new();
        pattern2.stitch_abs(20.0, 20.0);
        pattern2.stitch_abs(30.0, 30.0);

        collection.add("design1".to_string(), pattern1);
        collection.add("design2".to_string(), pattern2);

        let total_stitches: usize = collection.patterns().map(|p| p.count_stitches()).sum();
        assert_eq!(total_stitches, 3); // 1 + 2
    }

    #[test]
    fn test_clear() {
        let mut collection = EmbPatternCollection::new();
        collection.add("design1".to_string(), EmbPattern::new());
        collection.add("design2".to_string(), EmbPattern::new());

        assert_eq!(collection.len(), 2);
        collection.clear();
        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut collection1 = EmbPatternCollection::new();
        collection1.add("design1".to_string(), EmbPattern::new());

        let mut collection2 = EmbPatternCollection::new();
        collection2.add("design2".to_string(), EmbPattern::new());
        collection2.add("design3".to_string(), EmbPattern::new());

        collection1.merge(collection2);
        assert_eq!(collection1.len(), 3);
        assert!(collection1.contains("design1"));
        assert!(collection1.contains("design2"));
        assert!(collection1.contains("design3"));
    }

    #[test]
    fn test_merge_with_conflicts() {
        let mut collection1 = EmbPatternCollection::new();
        let mut pattern1 = EmbPattern::new();
        pattern1.stitch_abs(10.0, 10.0);
        collection1.add("design1".to_string(), pattern1);

        let mut collection2 = EmbPatternCollection::new();
        let mut pattern2 = EmbPattern::new();
        pattern2.stitch_abs(20.0, 20.0);
        pattern2.stitch_abs(30.0, 30.0);
        collection2.add("design1".to_string(), pattern2);

        collection1.merge(collection2);
        assert_eq!(collection1.len(), 1);
        // Should have the pattern from collection2 (2 stitches)
        assert_eq!(collection1.get("design1").unwrap().count_stitches(), 2);
    }

    #[test]
    fn test_with_capacity() {
        let collection = EmbPatternCollection::with_capacity(10);
        assert!(collection.is_empty());
    }
}
