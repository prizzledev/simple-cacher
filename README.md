# Simple Cacher

[![Crates.io](https://img.shields.io/crates/v/simple-cacher.svg)](https://crates.io/crates/simple-cacher)
[![Documentation](https://docs.rs/simple-cacher/badge.svg)](https://docs.rs/simple-cacher)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/prizzledev/simple-cacher/workflows/CI/badge.svg)](https://github.com/prizzledev/simple-cacher/actions)

A high-performance, flexible caching library for Rust with custom matching capabilities and automatic expiration.

## Features

- **Fast O(1) exact key lookups** using IndexMap
- **Custom pattern matching** via the `Matcher<T>` trait
- **Automatic expiration** with configurable TTL per entry
- **Size-limited caches** with FIFO eviction (oldest entries removed first)
- **Lazy cleanup** - expired entries removed on access
- **Zero-copy value access** through references
- **Thread-safe design** (when used with appropriate synchronization)
- **Comprehensive error handling**

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple_cacher = "0.1.0"

# Optional: Enable regex support
simple_cacher = { version = "0.1.0", features = ["regex_support"] }
```

## Quick Start

```rust
use simple_cacher::*;
use std::time::Duration;

// Create a cache with 5-minute TTL
let mut cache = SimpleCacher::new(Duration::from_secs(300));

// Insert data
cache.insert("user:123".to_string(), "Alice".to_string());

// Retrieve data
match cache.get(&"user:123".to_string()) {
    Ok(entry) => println!("Found: {}", entry.value()),
    Err(SimpleCacheError::NotFound) => println!("Not found"),
    Err(SimpleCacheError::Expired) => println!("Expired"),
}

// Check if expired
if let Ok(entry) = cache.get(&"user:123".to_string()) {
    println!("Entry age: {:?}", entry.age());
    println!("Is expired: {}", entry.is_expired());
}
```

## Custom Matching

The library provides powerful pattern matching capabilities:

```rust
use simple_cacher::*;
use std::time::Duration;

let mut cache = SimpleCacher::new(Duration::from_secs(300));

// Insert data
cache.insert("user:alice".to_string(), "Alice Johnson".to_string());
cache.insert("user:bob".to_string(), "Bob Smith".to_string());
cache.insert("admin:charlie".to_string(), "Charlie Admin".to_string());

// Find by prefix
let user_matcher = PrefixMatcher::new("user:");
let users = cache.get_all_by_matcher(&user_matcher);
println!("Found {} users", users.len());

// Find first match
if let Ok(user) = cache.get_by_matcher(&user_matcher) {
    println!("First user: {}", user.value());
}
```

### Built-in Matchers

- `PrefixMatcher` - Match strings by prefix
- `SuffixMatcher` - Match strings by suffix
- `ContainsMatcher` - Match strings containing substring
- `RangeMatcher<T>` - Match numeric values in range
- `FnMatcher<T, F>` - Custom function-based matching
- `ExactMatcher<T>` - Exact matching (useful in generic code)

### Custom Matchers

Implement the `Matcher<T>` trait for domain-specific matching:

```rust
struct DomainMatcher {
    domain: String,
}

impl Matcher<String> for DomainMatcher {
    fn matches(&self, email: &String) -> bool {
        email.ends_with(&format!("@{}", self.domain))
    }
}

// Usage
let company_matcher = DomainMatcher { domain: "company.com".to_string() };
let company_emails = cache.get_all_by_matcher(&company_matcher);
```

## Size-Limited Caches

```rust
use simple_cacher::*;
use std::time::Duration;

// Cache with max 1000 entries, oldest removed first
let mut cache = SimpleCacher::with_max_size(
    Duration::from_secs(300),
    1000
);

// Fill beyond capacity - oldest entries automatically removed
for i in 0..1500 {
    cache.insert(format!("key_{}", i), format!("value_{}", i));
}

assert_eq!(cache.len(), 1000); // Only newest 1000 entries remain
```

## Per-Entry TTL

```rust
use simple_cacher::*;
use std::time::Duration;

let mut cache = SimpleCacher::new(Duration::from_secs(300)); // Default TTL

// Insert with custom TTL
cache.insert_with_ttl(
    "short_lived".to_string(),
    "data".to_string(),
    Duration::from_secs(60) // 1 minute TTL
);
```

## Error Handling

```rust
use simple_cacher::*;

match cache.get(&"some_key".to_string()) {
    Ok(entry) => {
        // Cache hit
        println!("Value: {}", entry.value());
        println!("Age: {:?}", entry.age());
    }
    Err(SimpleCacheError::NotFound) => {
        // Key doesn't exist
        println!("Cache miss");
    }
    Err(SimpleCacheError::Expired) => {
        // Entry existed but expired (automatically removed)
        println!("Entry expired");
    }
}
```

## Cache Management

```rust
use simple_cacher::*;

let mut cache = SimpleCacher::new(Duration::from_secs(300));

// Manual cleanup
let removed = cache.cleanup_expired();
println!("Removed {} expired entries", removed);

// Cache statistics
let stats = cache.stats();
println!("Total: {}, Active: {}, Expired: {}", 
         stats.total_entries, stats.active_entries, stats.expired_entries);

// Iterate over active entries only
for (key, entry) in cache.iter_active() {
    println!("{}: {} (age: {:?})", key, entry.value(), entry.age());
}

// Clear all entries
cache.clear();
```

## Performance Characteristics

- **Insert**: O(1) average case
- **Exact lookup**: O(1) average case
- **Pattern matching**: O(n) where n is cache size
- **Cleanup**: O(k) where k is number of expired entries
- **Memory**: Minimal overhead, only stores necessary metadata

## Examples

The library includes several examples demonstrating different use cases:

- `basic_usage.rs` - Basic cache operations and expiration
- `regex_matching.rs` - Advanced pattern matching with regex (requires `regex_support` feature)
- `file_cache.rs` - File content caching with directory matching

Run examples with:

```bash
# Basic usage
cargo run --example basic_usage

# Regex matching (requires regex_support feature)
cargo run --example regex_matching --features regex_support

# File caching
cargo run --example file_cache
```

## Optional Features

### Regex Support

Enable regex-based matching by adding the `regex_support` feature:

```toml
[dependencies]
simple_cacher = { version = "0.1.0", features = ["regex_support"] }
```

This provides `RegexMatcher` for complex pattern matching.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

### 0.1.0

- Initial release
- Basic caching with TTL support
- Custom matcher system
- Size-limited caches with FIFO eviction
- Comprehensive test coverage