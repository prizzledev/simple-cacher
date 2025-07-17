// Basic cache operations: insert, get, expiration handling

use simple_cacher::*;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Basic Cache Usage Example ===\n");

    // Create a cache with 2-second TTL
    let mut cache = SimpleCacher::new(Duration::from_secs(2));

    // Insert some user data
    cache.insert("user:alice".to_string(), "Alice Johnson".to_string());
    cache.insert("user:bob".to_string(), "Bob Smith".to_string());
    cache.insert("user:charlie".to_string(), "Charlie Brown".to_string());

    println!("📦 Inserted 3 users into cache");

    // Successful lookups
    println!("\n🔍 Looking up users:");
    match cache.get(&"user:alice".to_string()) {
        Ok(user) => {
            println!("✅ Found: {} (age: {:?})", user.value(), user.age());
            println!("   Created at: {:?}", user.created_at());
            println!("   Expired: {}", user.is_expired());
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    match cache.get(&"user:bob".to_string()) {
        Ok(user) => println!("✅ Found: {}", user.value()),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Try to get non-existent user
    match cache.get(&"user:nonexistent".to_string()) {
        Ok(user) => println!("✅ Found: {}", user.value()),
        Err(SimpleCacheError::NotFound) => println!("❌ User not found (as expected)"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }

    // Show cache statistics
    let stats = cache.stats();
    println!("\n📊 Cache Stats:");
    println!("   Total entries: {}", stats.total_entries);
    println!("   Active entries: {}", stats.active_entries);
    println!("   Expired entries: {}", stats.expired_entries);
    println!("   Max age: {:?}", stats.max_age);

    // Wait for entries to expire
    println!("\n⏰ Waiting 3 seconds for entries to expire...");
    thread::sleep(Duration::from_secs(3));

    // Try to access expired entries
    println!("\n🔍 Accessing after expiration:");
    match cache.get(&"user:alice".to_string()) {
        Ok(user) => println!("✅ Found: {}", user.value()),
        Err(SimpleCacheError::Expired) => println!("⏰ Alice's data expired and was auto-removed"),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Show updated stats
    let stats_after = cache.stats();
    println!("\n📊 Cache Stats After Expiration:");
    println!("   Total entries: {}", stats_after.total_entries);
    println!("   Active entries: {}", stats_after.active_entries);
    println!("   Expired entries: {}", stats_after.expired_entries);

    // Add new data with custom TTL
    println!("\n🔧 Adding user with custom 10-second TTL:");
    cache.insert_with_ttl(
        "user:diana".to_string(),
        "Diana Prince".to_string(),
        Duration::from_secs(10),
    );

    match cache.get(&"user:diana".to_string()) {
        Ok(user) => println!("✅ Found: {} (will expire in ~10 seconds)", user.value()),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demonstrate mutable access
    println!("\n✏️  Modifying cached data:");
    match cache.get_mut(&"user:diana".to_string()) {
        Ok(user) => {
            let old_value = user.value().clone();
            *user.value_mut() = "Wonder Woman".to_string();
            println!("✅ Updated: '{}' -> '{}'", old_value, user.value());
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Verify the change
    match cache.get(&"user:diana".to_string()) {
        Ok(user) => println!("✅ Verified: {}", user.value()),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n🎉 Basic usage example completed!");
}
