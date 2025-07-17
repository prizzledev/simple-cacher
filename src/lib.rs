//! # Simple Cacher
//!
//! A high-performance, flexible caching library for Rust with custom matching capabilities
//! and automatic expiration.
//!
//! ## Key Features
//!
//! - **Fast O(1) exact key lookups** using IndexMap
//! - **Custom pattern matching** via the `Matcher<T>` trait
//! - **Automatic expiration** with configurable TTL per entry
//! - **Size-limited caches** with FIFO eviction (oldest entries removed first)
//! - **Lazy cleanup** - expired entries removed on access
//! - **Zero-copy value access** through references
//!
//! ## Quick Start
//!
//! ```rust
//! use simple_cacher::*;
//! use std::time::Duration;
//!
//! // Create a cache with 5-minute TTL
//! let mut cache = SimpleCacher::new(Duration::from_secs(300));
//!
//! // Insert data
//! cache.insert("user:123".to_string(), "Alice".to_string());
//!
//! // Retrieve data
//! match cache.get(&"user:123".to_string()) {
//!     Ok(entry) => println!("Found: {}", entry.value()),
//!     Err(SimpleCacheError::NotFound) => println!("Not found"),
//!     Err(SimpleCacheError::Expired) => println!("Expired"),
//! }
//! ```
//!
//! ## Custom Matching
//!
//! ```rust
//! use simple_cacher::*;
//! use std::time::Duration;
//!
//! let mut cache = SimpleCacher::new(Duration::from_secs(300));
//! cache.insert("user:alice".to_string(), "Alice Johnson".to_string());
//! cache.insert("admin:bob".to_string(), "Bob Admin".to_string());
//!
//! // Find by prefix
//! let user_matcher = PrefixMatcher::new("user:");
//! if let Ok(user) = cache.get_by_matcher(&user_matcher) {
//!     println!("Found user: {}", user.value());
//! }
//! ```

use indexmap::IndexMap;
use std::time::{Duration, Instant};

/// Error types returned by cache operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleCacheError {
    /// The requested key was not found in the cache.
    NotFound,
    /// The entry was found but has expired and was automatically removed.
    Expired,
}

impl std::fmt::Display for SimpleCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleCacheError::NotFound => write!(f, "Cache entry not found"),
            SimpleCacheError::Expired => write!(f, "Cache entry has expired"),
        }
    }
}

impl std::error::Error for SimpleCacheError {}

/// A cached value with metadata about its creation time and expiration.
///
/// This struct wraps the actual cached value along with timing information
/// to support automatic expiration and cache management.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(60));
/// cache.insert("key".to_string(), "value".to_string());
///
/// if let Ok(entry) = cache.get(&"key".to_string()) {
///     println!("Value: {}", entry.value());
///     println!("Age: {:?}", entry.age());
///     println!("Expired: {}", entry.is_expired());
/// }
/// ```
pub struct SimpleCacheObject<U> {
    created_at: Instant,
    value: U,
    max_age: Duration,
}

impl<U> SimpleCacheObject<U> {
    /// Creates a new cache object with the given value and maximum age.
    fn new(value: U, max_age: Duration) -> Self {
        Self {
            created_at: Instant::now(),
            value,
            max_age,
        }
    }

    /// Returns `true` if this cache entry has expired based on its max age.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_millis(100));
    /// cache.insert("key".to_string(), "value".to_string());
    ///
    /// if let Ok(entry) = cache.get(&"key".to_string()) {
    ///     // Should not be expired immediately
    ///     assert!(!entry.is_expired());
    /// }
    /// ```
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.max_age
    }

    /// Returns a reference to the cached value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("greeting".to_string(), "Hello, World!".to_string());
    ///
    /// if let Ok(entry) = cache.get(&"greeting".to_string()) {
    ///     assert_eq!(entry.value(), "Hello, World!");
    /// }
    /// ```
    pub fn value(&self) -> &U {
        &self.value
    }

    /// Returns a mutable reference to the cached value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("counter".to_string(), 0u32);
    ///
    /// if let Ok(entry) = cache.get_mut(&"counter".to_string()) {
    ///     *entry.value_mut() += 1;
    ///     assert_eq!(*entry.value(), 1);
    /// }
    /// ```
    pub fn value_mut(&mut self) -> &mut U {
        &mut self.value
    }

    /// Consumes the cache object and returns the cached value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("data".to_string(), vec![1, 2, 3]);
    ///
    /// if let Some(entry) = cache.remove(&"data".to_string()) {
    ///     let data = entry.into_value();
    ///     assert_eq!(data, vec![1, 2, 3]);
    /// }
    /// ```
    pub fn into_value(self) -> U {
        self.value
    }

    /// Returns the age of this cache entry (time since creation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("key".to_string(), "value".to_string());
    ///
    /// if let Ok(entry) = cache.get(&"key".to_string()) {
    ///     let age = entry.age();
    ///     assert!(age.as_millis() >= 0);
    /// }
    /// ```
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Returns the instant when this entry was created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::{Duration, Instant};
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// let before = Instant::now();
    /// cache.insert("key".to_string(), "value".to_string());
    /// let after = Instant::now();
    ///
    /// if let Ok(entry) = cache.get(&"key".to_string()) {
    ///     let created = entry.created_at();
    ///     assert!(created >= before && created <= after);
    /// }
    /// ```
    pub fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// Trait for implementing custom matching logic against cache keys.
///
/// This trait allows you to define complex search patterns for finding cached entries
/// beyond simple exact key matching. Implementations can match based on patterns,
/// ranges, regular expressions, or any custom logic.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// // Custom matcher for email domains
/// struct DomainMatcher {
///     domain: String,
/// }
///
/// impl Matcher<String> for DomainMatcher {
///     fn matches(&self, email: &String) -> bool {
///         email.ends_with(&format!("@{}", self.domain))
///     }
/// }
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert("alice@company.com".to_string(), "Alice".to_string());
/// cache.insert("bob@company.com".to_string(), "Bob".to_string());
/// cache.insert("charlie@gmail.com".to_string(), "Charlie".to_string());
///
/// let company_matcher = DomainMatcher { domain: "company.com".to_string() };
/// let company_users = cache.get_all_by_matcher(&company_matcher);
/// assert_eq!(company_users.len(), 2);
/// ```
pub trait Matcher<T> {
    /// Returns `true` if the given key matches this matcher's criteria.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to test against this matcher
    ///
    /// # Returns
    ///
    /// `true` if the key matches, `false` otherwise
    fn matches(&self, key: &T) -> bool;
}

/// A high-performance cache with automatic expiration and custom matching capabilities.
///
/// `SimpleCacher` provides fast O(1) exact key lookups using an IndexMap, along with
/// flexible O(n) pattern matching via the `Matcher` trait. Entries automatically expire
/// based on configurable TTL values, and the cache can be size-limited with FIFO eviction.
///
/// # Type Parameters
///
/// * `T` - The type of keys (must implement `Clone + Eq + Hash`)
/// * `U` - The type of cached values
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
///
/// // Insert a value
/// cache.insert("user:123".to_string(), "Alice Johnson".to_string());
///
/// // Retrieve it
/// match cache.get(&"user:123".to_string()) {
///     Ok(entry) => println!("Found: {}", entry.value()),
///     Err(SimpleCacheError::NotFound) => println!("Not found"),
///     Err(SimpleCacheError::Expired) => println!("Expired and removed"),
/// }
/// ```
///
/// ## Size-Limited Cache
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// // Cache with max 1000 entries
/// let mut cache = SimpleCacher::with_max_size(Duration::from_secs(300), 1000);
///
/// // When full, oldest entries are automatically removed
/// for i in 0..1500 {
///     cache.insert(format!("key_{}", i), format!("value_{}", i));
/// }
///
/// assert_eq!(cache.len(), 1000); // Only newest 1000 entries remain
/// ```
pub struct SimpleCacher<T, U> {
    cache: IndexMap<T, SimpleCacheObject<U>>,
    max_age: Duration,
    max_size: Option<usize>,
}

impl<T, U> SimpleCacher<T, U>
where
    T: Clone + Eq + std::hash::Hash,
{
    /// Creates a new cache with the specified maximum age for entries.
    ///
    /// All entries inserted into this cache will expire after the given duration,
    /// unless overridden with `insert_with_ttl`.
    ///
    /// # Arguments
    ///
    /// * `max_age` - Default time-to-live for cache entries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300)); // 5 minutes
    /// cache.insert("key".to_string(), "value".to_string());
    /// ```
    pub fn new(max_age: Duration) -> Self {
        Self {
            cache: IndexMap::new(),
            max_age,
            max_size: None,
        }
    }

    /// Creates a new cache with both maximum age and maximum size constraints.
    ///
    /// When the cache exceeds `max_size` entries, the oldest entries are automatically
    /// removed to make room for new ones (FIFO eviction policy).
    ///
    /// # Arguments
    ///
    /// * `max_age` - Default time-to-live for cache entries
    /// * `max_size` - Maximum number of entries to keep in the cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache: SimpleCacher<String,String> = SimpleCacher::with_max_size(
    ///     Duration::from_secs(300), // 5 minutes TTL
    ///     1000 // max 1000 entries
    /// );
    /// ```
    pub fn with_max_size(max_age: Duration, max_size: usize) -> Self {
        Self {
            cache: IndexMap::new(),
            max_age,
            max_size: Some(max_size),
        }
    }

    /// Retrieves an entry by exact key match in O(1) time.
    ///
    /// If the entry exists but has expired, it will be automatically removed
    /// from the cache and `SimpleCacheError::Expired` will be returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Ok(&SimpleCacheObject<U>)` - The cached entry if found and not expired
    /// * `Err(SimpleCacheError::NotFound)` - The key doesn't exist
    /// * `Err(SimpleCacheError::Expired)` - The entry existed but expired (now removed)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("user:123".to_string(), "Alice".to_string());
    ///
    /// match cache.get(&"user:123".to_string()) {
    ///     Ok(entry) => {
    ///         println!("Found: {}", entry.value());
    ///         println!("Age: {:?}", entry.age());
    ///     }
    ///     Err(SimpleCacheError::NotFound) => println!("User not found"),
    ///     Err(SimpleCacheError::Expired) => println!("User data expired"),
    /// }
    /// ```
    pub fn get(&mut self, key: &T) -> Result<&SimpleCacheObject<U>, SimpleCacheError> {
        // Check if entry exists and if it's expired
        let should_remove = match self.cache.get(key) {
            Some(obj) => obj.is_expired(),
            None => return Err(SimpleCacheError::NotFound),
        };

        if should_remove {
            self.cache.shift_remove(key);
            return Err(SimpleCacheError::Expired);
        }

        // Safe to get immutable reference now
        Ok(self.cache.get(key).unwrap())
    }

    /// Retrieves a mutable reference to an entry by exact key match.
    ///
    /// Similar to `get()`, but returns a mutable reference that allows you to modify
    /// the cached value in place. Expired entries are automatically removed.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Ok(&mut SimpleCacheObject<U>)` - Mutable reference to the cached entry
    /// * `Err(SimpleCacheError::NotFound)` - The key doesn't exist
    /// * `Err(SimpleCacheError::Expired)` - The entry existed but expired (now removed)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(60));
    /// cache.insert("counter".to_string(), 0u32);
    ///
    /// if let Ok(entry) = cache.get_mut(&"counter".to_string()) {
    ///     *entry.value_mut() += 1;
    ///     assert_eq!(*entry.value(), 1);
    /// }
    /// ```
    pub fn get_mut(&mut self, key: &T) -> Result<&mut SimpleCacheObject<U>, SimpleCacheError> {
        // Check if exists and if it's expired first
        let should_remove = match self.cache.get(key) {
            Some(obj) => obj.is_expired(),
            None => return Err(SimpleCacheError::NotFound),
        };

        if should_remove {
            self.cache.shift_remove(key);
            return Err(SimpleCacheError::Expired);
        }

        // Safe to get mutable reference now
        Ok(self.cache.get_mut(key).unwrap())
    }

    /// Finds the first entry matching the given matcher in O(n) time.
    ///
    /// This method iterates through all cache entries and returns the first one
    /// that matches the provided matcher's criteria. Expired entries encountered
    /// during the search are automatically cleaned up.
    ///
    /// # Arguments
    ///
    /// * `matcher` - An implementation of `Matcher<T>` that defines the search criteria
    ///
    /// # Returns
    ///
    /// * `Ok(&SimpleCacheObject<U>)` - The first matching entry found
    /// * `Err(SimpleCacheError::NotFound)` - No matching entries found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("user:alice".to_string(), "Alice".to_string());
    /// cache.insert("user:bob".to_string(), "Bob".to_string());
    /// cache.insert("admin:charlie".to_string(), "Charlie".to_string());
    ///
    /// let user_matcher = PrefixMatcher::new("user:");
    /// if let Ok(user) = cache.get_by_matcher(&user_matcher) {
    ///     println!("Found user: {}", user.value());
    /// }
    /// ```
    pub fn get_by_matcher<M>(
        &mut self,
        matcher: &M,
    ) -> Result<&SimpleCacheObject<U>, SimpleCacheError>
    where
        M: Matcher<T>,
    {
        // First pass: collect expired keys and find match
        let mut expired_keys = Vec::new();
        let mut found_key = None;

        for (key, obj) in &self.cache {
            if obj.is_expired() {
                expired_keys.push(key.clone());
            } else if found_key.is_none() && matcher.matches(key) {
                found_key = Some(key.clone());
            }
        }

        // Clean up expired entries
        for key in expired_keys {
            self.cache.shift_remove(&key);
        }

        // Return the found entry (get fresh reference after cleanup)
        if let Some(key) = found_key {
            self.cache.get(&key).ok_or(SimpleCacheError::NotFound)
        } else {
            Err(SimpleCacheError::NotFound)
        }
    }

    /// Finds all entries matching the given matcher.
    ///
    /// This method returns all cache entries that match the provided matcher's criteria.
    /// The cache is automatically cleaned of expired entries before searching.
    ///
    /// # Arguments
    ///
    /// * `matcher` - An implementation of `Matcher<T>` that defines the search criteria
    ///
    /// # Returns
    ///
    /// A vector of tuples containing references to matching keys and their cached values.
    /// The vector may be empty if no matches are found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("user:alice".to_string(), "Alice".to_string());
    /// cache.insert("user:bob".to_string(), "Bob".to_string());
    /// cache.insert("admin:charlie".to_string(), "Charlie".to_string());
    ///
    /// let user_matcher = PrefixMatcher::new("user:");
    /// let users = cache.get_all_by_matcher(&user_matcher);
    /// println!("Found {} users", users.len());
    /// ```
    pub fn get_all_by_matcher<M>(&mut self, matcher: &M) -> Vec<(&T, &SimpleCacheObject<U>)>
    where
        M: Matcher<T>,
    {
        // Clean up expired entries first
        self.cleanup_expired();

        self.cache
            .iter()
            .filter(|(key, obj)| !obj.is_expired() && matcher.matches(key))
            .collect()
    }

    /// Inserts a new entry into the cache with the default TTL.
    ///
    /// If the cache has a size limit and is at capacity, the oldest entry
    /// will be automatically removed to make room for the new entry (FIFO eviction).
    /// If an entry with the same key already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value
    /// * `value` - The value to cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("user:123".to_string(), "Alice Johnson".to_string());
    /// ```
    pub fn insert(&mut self, key: T, value: U) {
        // Enforce max size by removing oldest entries (FIFO)
        if let Some(max_size) = self.max_size {
            while self.cache.len() >= max_size {
                self.cache.shift_remove_index(0);
            }
        }

        let cache_obj = SimpleCacheObject::new(value, self.max_age);
        self.cache.insert(key, cache_obj);
    }

    /// Inserts a new entry into the cache with a custom TTL.
    ///
    /// This allows you to override the default TTL for specific entries,
    /// useful for caching data with different freshness requirements.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value
    /// * `value` - The value to cache
    /// * `ttl` - Custom time-to-live for this specific entry
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300)); // Default 5 min
    ///
    /// // Cache with custom 1-hour TTL
    /// cache.insert_with_ttl(
    ///     "important_data".to_string(),
    ///     "critical information".to_string(),
    ///     Duration::from_secs(3600)
    /// );
    /// ```
    pub fn insert_with_ttl(&mut self, key: T, value: U, ttl: Duration) {
        if let Some(max_size) = self.max_size {
            while self.cache.len() >= max_size {
                self.cache.shift_remove_index(0);
            }
        }

        let cache_obj = SimpleCacheObject::new(value, ttl);
        self.cache.insert(key, cache_obj);
    }

    /// Removes an entry by key and returns it if it existed.
    ///
    /// This method removes the entry regardless of whether it has expired.
    /// Returns `None` if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to remove
    ///
    /// # Returns
    ///
    /// `Some(SimpleCacheObject<U>)` if the key existed, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("temp_data".to_string(), "temporary".to_string());
    ///
    /// if let Some(removed) = cache.remove(&"temp_data".to_string()) {
    ///     println!("Removed: {}", removed.into_value());
    /// }
    /// ```
    pub fn remove(&mut self, key: &T) -> Option<SimpleCacheObject<U>> {
        self.cache.shift_remove(key)
    }

    /// Checks if a key exists in the cache and is not expired.
    ///
    /// This is a lightweight check that doesn't trigger cleanup of expired entries.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence
    ///
    /// # Returns
    ///
    /// `true` if the key exists and is not expired, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("key".to_string(), "value".to_string());
    ///
    /// assert!(cache.contains_key(&"key".to_string()));
    /// assert!(!cache.contains_key(&"nonexistent".to_string()));
    /// ```
    pub fn contains_key(&self, key: &T) -> bool {
        self.cache
            .get(key)
            .map(|obj| !obj.is_expired())
            .unwrap_or(false)
    }

    /// Manually removes all expired entries from the cache.
    ///
    /// This method performs a full scan of the cache and removes all entries
    /// that have exceeded their TTL. This can be useful for periodic cleanup
    /// to free memory and maintain cache performance.
    ///
    /// # Returns
    ///
    /// The number of expired entries that were removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_millis(100));
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// // Wait for expiration
    /// std::thread::sleep(Duration::from_millis(150));
    ///
    /// let removed = cache.cleanup_expired();
    /// println!("Cleaned up {} expired entries", removed);
    /// ```
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_keys: Vec<T> = self
            .cache
            .iter()
            .filter_map(|(k, v)| {
                if v.is_expired() {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.cache.shift_remove(&key);
        }
        count
    }

    /// Returns the total number of entries in the cache (including expired ones).
    ///
    /// Note that this includes expired entries that haven't been cleaned up yet.
    /// Use `active_len()` to get only non-expired entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns the number of non-expired entries in the cache.
    ///
    /// This method counts only entries that are still valid (not expired).
    /// It does not modify the cache or remove expired entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// assert_eq!(cache.active_len(), 2);
    /// ```
    pub fn active_len(&self) -> usize {
        self.cache
            .iter()
            .filter(|(_, obj)| !obj.is_expired())
            .count()
    }

    /// Returns `true` if the cache contains no entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let cache = SimpleCacher::<String, String>::new(Duration::from_secs(300));
    /// assert!(cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Removes all entries from the cache.
    ///
    /// After calling this method, the cache will be empty and `len()` will return 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("key".to_string(), "value".to_string());
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns comprehensive statistics about the cache state.
    ///
    /// This provides detailed information about cache usage, including total entries,
    /// active (non-expired) entries, expired entries, and configuration settings.
    ///
    /// # Returns
    ///
    /// A `CacheStats` struct containing cache metrics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::with_max_size(Duration::from_secs(300), 1000);
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// let stats = cache.stats();
    /// println!("Active entries: {}", stats.active_entries);
    /// println!("Max size: {:?}", stats.max_size);
    /// ```
    pub fn stats(&self) -> CacheStats {
        let total = self.cache.len();
        let expired = self
            .cache
            .iter()
            .filter(|(_, obj)| obj.is_expired())
            .count();

        CacheStats {
            total_entries: total,
            active_entries: total - expired,
            expired_entries: expired,
            max_size: self.max_size,
            max_age: self.max_age,
        }
    }

    /// Returns an iterator over all non-expired entries in the cache.
    ///
    /// This iterator yields tuples of `(&T, &SimpleCacheObject<U>)` for each
    /// active (non-expired) entry. Expired entries are skipped.
    ///
    /// # Returns
    ///
    /// An iterator over active cache entries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    /// use std::time::Duration;
    ///
    /// let mut cache = SimpleCacher::new(Duration::from_secs(300));
    /// cache.insert("user:1".to_string(), "Alice".to_string());
    /// cache.insert("user:2".to_string(), "Bob".to_string());
    ///
    /// for (key, entry) in cache.iter_active() {
    ///     println!("{}: {} (age: {:?})", key, entry.value(), entry.age());
    /// }
    /// ```
    pub fn iter_active(&self) -> impl Iterator<Item = (&T, &SimpleCacheObject<U>)> {
        self.cache.iter().filter(|(_, obj)| !obj.is_expired())
    }
}

/// Statistics about cache state and performance.
///
/// This struct provides detailed metrics about cache usage, including
/// the number of active and expired entries, size limits, and TTL settings.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries in the cache (including expired)
    pub total_entries: usize,
    /// Number of non-expired entries
    pub active_entries: usize,
    /// Number of expired entries (not yet cleaned up)
    pub expired_entries: usize,
    /// Maximum number of entries allowed (None if unlimited)
    pub max_size: Option<usize>,
    /// Default time-to-live for new entries
    pub max_age: Duration,
}

// ========== Built-in Matchers ==========

/// Exact equality matcher for cache keys.
///
/// This matcher performs exact equality comparison, similar to using `get()` directly,
/// but is useful in generic code where you need a `Matcher` implementation.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert("exact_key".to_string(), "value".to_string());
///
/// let matcher = ExactMatcher::new("exact_key".to_string());
/// if let Ok(entry) = cache.get_by_matcher(&matcher) {
///     println!("Found: {}", entry.value());
/// }
/// ```
pub struct ExactMatcher<T> {
    target: T,
}

impl<T> ExactMatcher<T> {
    /// Creates a new exact matcher for the given target value.
    ///
    /// # Arguments
    ///
    /// * `target` - The exact value to match against
    pub fn new(target: T) -> Self {
        Self { target }
    }
}

impl<T> Matcher<T> for ExactMatcher<T>
where
    T: PartialEq,
{
    fn matches(&self, key: &T) -> bool {
        key == &self.target
    }
}

/// String prefix matcher for finding keys that start with a specific string.
///
/// This matcher is useful for finding groups of related cache entries that
/// follow a naming convention with common prefixes.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert("user:alice".to_string(), "Alice Johnson".to_string());
/// cache.insert("user:bob".to_string(), "Bob Smith".to_string());
/// cache.insert("admin:charlie".to_string(), "Charlie Admin".to_string());
///
/// let user_matcher = PrefixMatcher::new("user:");
/// let users = cache.get_all_by_matcher(&user_matcher);
/// assert_eq!(users.len(), 2); // Found alice and bob
/// ```
pub struct PrefixMatcher {
    prefix: String,
}

impl PrefixMatcher {
    /// Creates a new prefix matcher.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix string to match against
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Matcher<String> for PrefixMatcher {
    fn matches(&self, key: &String) -> bool {
        key.starts_with(&self.prefix)
    }
}

impl Matcher<&str> for PrefixMatcher {
    fn matches(&self, key: &&str) -> bool {
        key.starts_with(&self.prefix)
    }
}

/// String suffix matcher for finding keys that end with a specific string.
///
/// This matcher is useful for finding cache entries based on file extensions,
/// domain names, or other suffix-based patterns.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert("document.pdf".to_string(), "PDF content".to_string());
/// cache.insert("image.jpg".to_string(), "JPEG data".to_string());
/// cache.insert("script.js".to_string(), "JavaScript code".to_string());
///
/// let pdf_matcher = SuffixMatcher::new(".pdf");
/// let pdfs = cache.get_all_by_matcher(&pdf_matcher);
/// assert_eq!(pdfs.len(), 1);
/// ```
pub struct SuffixMatcher {
    suffix: String,
}

impl SuffixMatcher {
    /// Creates a new suffix matcher.
    ///
    /// # Arguments
    ///
    /// * `suffix` - The suffix string to match against
    pub fn new(suffix: impl Into<String>) -> Self {
        Self {
            suffix: suffix.into(),
        }
    }
}

impl Matcher<String> for SuffixMatcher {
    fn matches(&self, key: &String) -> bool {
        key.ends_with(&self.suffix)
    }
}

impl Matcher<&str> for SuffixMatcher {
    fn matches(&self, key: &&str) -> bool {
        key.ends_with(&self.suffix)
    }
}

/// String substring matcher for finding keys that contain a specific string.
///
/// This matcher searches for cache entries where the key contains the specified
/// substring anywhere within it.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert("user_profile_123".to_string(), "Profile data".to_string());
/// cache.insert("user_settings_456".to_string(), "Settings data".to_string());
/// cache.insert("admin_config".to_string(), "Config data".to_string());
///
/// let profile_matcher = ContainsMatcher::new("profile");
/// let profiles = cache.get_all_by_matcher(&profile_matcher);
/// assert_eq!(profiles.len(), 1);
/// ```
pub struct ContainsMatcher {
    substring: String,
}

impl ContainsMatcher {
    /// Creates a new substring matcher.
    ///
    /// # Arguments
    ///
    /// * `substring` - The substring to search for within keys
    pub fn new(substring: impl Into<String>) -> Self {
        Self {
            substring: substring.into(),
        }
    }
}

impl Matcher<String> for ContainsMatcher {
    fn matches(&self, key: &String) -> bool {
        key.contains(&self.substring)
    }
}

impl Matcher<&str> for ContainsMatcher {
    fn matches(&self, key: &&str) -> bool {
        key.contains(&self.substring)
    }
}

/// Numeric range matcher for finding keys within a specified range.
///
/// This matcher is useful for numeric keys like IDs, scores, timestamps,
/// or any other ordered numeric data.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert(85, "Good score".to_string());
/// cache.insert(92, "Excellent score".to_string());
/// cache.insert(67, "Average score".to_string());
/// cache.insert(45, "Poor score".to_string());
///
/// let high_score_matcher = RangeMatcher::new(80, 100);
/// let high_scores = cache.get_all_by_matcher(&high_score_matcher);
/// assert_eq!(high_scores.len(), 2); // 85 and 92
/// ```
pub struct RangeMatcher<T> {
    min: T,
    max: T,
    inclusive: bool,
}

impl<T> RangeMatcher<T> {
    /// Creates a new inclusive range matcher.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value (inclusive)
    /// * `max` - Maximum value (inclusive)
    pub fn new(min: T, max: T) -> Self {
        Self {
            min,
            max,
            inclusive: true,
        }
    }

    /// Creates a new exclusive range matcher.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value (exclusive)
    /// * `max` - Maximum value (exclusive)
    pub fn exclusive(min: T, max: T) -> Self {
        Self {
            min,
            max,
            inclusive: false,
        }
    }
}

impl<T> Matcher<T> for RangeMatcher<T>
where
    T: PartialOrd,
{
    fn matches(&self, key: &T) -> bool {
        if self.inclusive {
            key >= &self.min && key <= &self.max
        } else {
            key > &self.min && key < &self.max
        }
    }
}

/// Function-based matcher for maximum flexibility in matching logic.
///
/// This matcher allows you to provide a custom function that determines
/// whether a key matches. This is the most flexible matcher and can implement
/// any matching logic you need.
///
/// # Examples
///
/// ```rust
/// use simple_cacher::*;
/// use std::time::Duration;
///
/// let mut cache = SimpleCacher::new(Duration::from_secs(300));
/// cache.insert(2, "Even number".to_string());
/// cache.insert(3, "Odd number".to_string());
/// cache.insert(4, "Even number".to_string());
/// cache.insert(5, "Odd number".to_string());
///
/// // Find even numbers
/// let even_matcher = FnMatcher::new(|&key: &i32| key % 2 == 0);
/// let even_numbers = cache.get_all_by_matcher(&even_matcher);
/// assert_eq!(even_numbers.len(), 2); // 2 and 4
/// ```
pub struct FnMatcher<T, F>
where
    F: Fn(&T) -> bool,
{
    matcher_fn: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> FnMatcher<T, F>
where
    F: Fn(&T) -> bool,
{
    /// Creates a new function-based matcher.
    ///
    /// # Arguments
    ///
    /// * `matcher_fn` - A function that takes a key reference and returns `true` if it matches
    ///
    /// # Examples
    ///
    /// ```rust
    /// use simple_cacher::*;
    ///
    /// // Match strings longer than 5 characters
    /// let long_string_matcher = FnMatcher::new(|s: &String| s.len() > 5);
    /// ```
    pub fn new(matcher_fn: F) -> Self {
        Self {
            matcher_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Matcher<T> for FnMatcher<T, F>
where
    F: Fn(&T) -> bool,
{
    fn matches(&self, key: &T) -> bool {
        (self.matcher_fn)(key)
    }
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic_cache_operations() {
        let mut cache = SimpleCacher::new(Duration::from_secs(1));

        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()).unwrap().value(), "value1");

        // Test not found
        assert!(matches!(
            cache.get(&"nonexistent".to_string()),
            Err(SimpleCacheError::NotFound)
        ));
    }

    #[test]
    fn test_expiration() {
        let mut cache = SimpleCacher::new(Duration::from_millis(100));

        cache.insert("key1".to_string(), "value1".to_string());
        assert!(cache.get(&"key1".to_string()).is_ok());

        thread::sleep(Duration::from_millis(150));
        assert!(matches!(
            cache.get(&"key1".to_string()),
            Err(SimpleCacheError::Expired)
        ));
    }

    #[test]
    fn test_max_size() {
        let mut cache = SimpleCacher::with_max_size(Duration::from_secs(10), 2);

        cache.insert(1, "value1");
        cache.insert(2, "value2");
        cache.insert(3, "value3"); // Should evict key 1

        assert!(matches!(cache.get(&1), Err(SimpleCacheError::NotFound)));
        assert!(cache.get(&2).is_ok());
        assert!(cache.get(&3).is_ok());
    }

    #[test]
    fn test_prefix_matcher() {
        let mut cache = SimpleCacher::new(Duration::from_secs(10));

        cache.insert("prefix_key1".to_string(), "value1");
        cache.insert("prefix_key2".to_string(), "value2");
        cache.insert("other_key".to_string(), "value3");

        let matcher = PrefixMatcher::new("prefix_");
        let result = cache.get_by_matcher(&matcher);
        assert!(result.is_ok());
        assert!(result.unwrap().value().starts_with("value"));
    }

    #[test]
    fn test_range_matcher() {
        let mut cache = SimpleCacher::new(Duration::from_secs(10));

        cache.insert(1, "value1");
        cache.insert(5, "value5");
        cache.insert(10, "value10");
        cache.insert(15, "value15");

        let matcher = RangeMatcher::new(3, 12);
        let result = cache.get_by_matcher(&matcher);
        assert!(result.is_ok());

        // Should find either key 5 or 10
        let found_value = result.unwrap().value();
        assert!(found_value.to_string() == "value5" || found_value.to_string() == "value10");
    }

    #[test]
    fn test_function_matcher() {
        let mut cache = SimpleCacher::new(Duration::from_secs(10));

        cache.insert(2, "even");
        cache.insert(3, "odd");
        cache.insert(4, "even");
        cache.insert(5, "odd");

        let even_matcher = FnMatcher::new(|&key: &i32| key % 2 == 0);
        let result = cache.get_by_matcher(&even_matcher);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value().to_string(), "even");
    }
}
