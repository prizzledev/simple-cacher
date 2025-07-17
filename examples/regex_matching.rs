// Advanced pattern matching using regex (requires "regex_support" feature)
// Run with: cargo run --example regex_matching --features regex_support

#[cfg(feature = "regex_support")]
use regex::Regex;
use simple_cacher::*;
use std::time::Duration;

#[cfg(feature = "regex_support")]
struct RegexMatcher {
    regex: Regex,
    description: String,
}

#[cfg(feature = "regex_support")]
impl RegexMatcher {
    fn new(pattern: &str, description: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: Regex::new(pattern)?,
            description: description.to_string(),
        })
    }

    fn email_pattern() -> Result<Self, regex::Error> {
        Self::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
            "Valid email addresses",
        )
    }

    fn phone_pattern() -> Result<Self, regex::Error> {
        Self::new(
            r"^\+?1?[-.\s]?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}$",
            "US phone numbers",
        )
    }

    fn ip_address_pattern() -> Result<Self, regex::Error> {
        Self::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$", "IPv4 addresses")
    }

    fn version_pattern() -> Result<Self, regex::Error> {
        Self::new(
            r"^v?(\d+)\.(\d+)\.(\d+)(-[a-zA-Z0-9]+)?(\+[a-zA-Z0-9]+)?$",
            "Semantic version numbers",
        )
    }

    fn url_pattern() -> Result<Self, regex::Error> {
        Self::new(r"^https?://[^\s/$.?#].[^\s]*$", "HTTP/HTTPS URLs")
    }

    fn uuid_pattern() -> Result<Self, regex::Error> {
        Self::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
            "UUID format",
        )
    }
}

#[cfg(feature = "regex_support")]
impl Matcher<String> for RegexMatcher {
    fn matches(&self, key: &String) -> bool {
        self.regex.is_match(key)
    }
}

#[cfg(feature = "regex_support")]
fn main() {
    println!("=== Regex Pattern Matching Example ===\n");

    // Create cache for various data types
    let mut data_cache = SimpleCacher::new(Duration::from_secs(1800)); // 30 minutes

    println!("🔧 Setting up test data with various patterns...\n");

    // Add email addresses
    let emails = vec![
        "alice@company.com",
        "bob.smith@example.org",
        "charlie+tag@gmail.com",
        "diana@test-domain.co.uk",
        "eve123@domain.info",
        "invalid-email-format",
        "another@invalid@email.com",
    ];

    for email in emails {
        data_cache.insert(email.to_string(), format!("Email: {}", email));
    }

    // Add phone numbers
    let phones = vec![
        "+1-555-123-4567",
        "(555) 987-6543",
        "555.456.7890",
        "1 555 333 2222",
        "+15551234567",
        "not-a-phone-number",
        "555-abc-defg",
    ];

    for phone in phones {
        data_cache.insert(phone.to_string(), format!("Phone: {}", phone));
    }

    // Add IP addresses
    let ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.0.1",
        "8.8.8.8",
        "127.0.0.1",
        "999.999.999.999", // Invalid
        "192.168.1",       // Incomplete
    ];

    for ip in ips {
        data_cache.insert(ip.to_string(), format!("IP: {}", ip));
    }

    // Add version numbers
    let versions = vec![
        "v1.0.0",
        "2.1.3",
        "v3.0.0-beta",
        "1.2.3-rc.1+build.456",
        "v0.9.8-alpha.2",
        "not-a-version",
        "1.2", // Incomplete version
    ];

    for version in versions {
        data_cache.insert(version.to_string(), format!("Version: {}", version));
    }

    // Add URLs
    let urls = vec![
        "https://www.example.com",
        "http://test.org/path?query=value",
        "https://api.service.com/v1/endpoint",
        "http://localhost:8080",
        "ftp://not-http.com", // Wrong protocol
        "not-a-url-at-all",
    ];

    for url in urls {
        data_cache.insert(url.to_string(), format!("URL: {}", url));
    }

    // Add UUIDs
    let uuids = vec![
        "550e8400-e29b-41d4-a716-446655440000",
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "6ba7b811-9dad-11d1-80b4-00c04fd430c8",
        "not-a-uuid-format",
        "550e8400-e29b-41d4-a716", // Incomplete
    ];

    for uuid in uuids {
        data_cache.insert(uuid.to_string(), format!("UUID: {}", uuid));
    }

    println!("✅ Added test data to cache");

    let stats = data_cache.stats();
    println!("   Total entries: {}", stats.total_entries);

    // Test regex patterns
    println!("\n🔍 Testing regex pattern matching:\n");

    // Test email pattern
    match RegexMatcher::email_pattern() {
        Ok(email_matcher) => {
            println!("📧 Finding valid email addresses:");
            let emails = data_cache.get_all_by_matcher(&email_matcher);

            for (key, _) in emails {
                println!("   ✅ Valid email: {}", key);
            }

            if let Ok(first_email) = data_cache.get_by_matcher(&email_matcher) {
                println!("   First valid email found: {}", first_email.value());
            }
        }
        Err(e) => println!("❌ Email regex error: {}", e),
    }

    // Test phone pattern
    match RegexMatcher::phone_pattern() {
        Ok(phone_matcher) => {
            println!("\n📞 Finding valid phone numbers:");
            let phones = data_cache.get_all_by_matcher(&phone_matcher);

            for (key, _) in phones {
                println!("   ✅ Valid phone: {}", key);
            }
        }
        Err(e) => println!("❌ Phone regex error: {}", e),
    }

    // Test IP address pattern
    match RegexMatcher::ip_address_pattern() {
        Ok(ip_matcher) => {
            println!("\n🌐 Finding valid IP addresses:");
            let ips = data_cache.get_all_by_matcher(&ip_matcher);

            for (key, _) in ips {
                println!("   ✅ Valid IP: {}", key);
            }
        }
        Err(e) => println!("❌ IP regex error: {}", e),
    }

    // Test version pattern
    match RegexMatcher::version_pattern() {
        Ok(version_matcher) => {
            println!("\n🏷️  Finding valid version numbers:");
            let versions = data_cache.get_all_by_matcher(&version_matcher);

            for (key, _) in versions {
                println!("   ✅ Valid version: {}", key);
            }
        }
        Err(e) => println!("❌ Version regex error: {}", e),
    }

    // Test URL pattern
    match RegexMatcher::url_pattern() {
        Ok(url_matcher) => {
            println!("\n🔗 Finding valid URLs:");
            let urls = data_cache.get_all_by_matcher(&url_matcher);

            for (key, _) in urls {
                println!("   ✅ Valid URL: {}", key);
            }
        }
        Err(e) => println!("❌ URL regex error: {}", e),
    }

    // Test UUID pattern
    match RegexMatcher::uuid_pattern() {
        Ok(uuid_matcher) => {
            println!("\n🔑 Finding valid UUIDs:");
            let uuids = data_cache.get_all_by_matcher(&uuid_matcher);

            for (key, _) in uuids {
                println!("   ✅ Valid UUID: {}", key);
            }
        }
        Err(e) => println!("❌ UUID regex error: {}", e),
    }

    // Custom regex patterns
    println!("\n🎯 Testing custom regex patterns:");

    // Find entries that look like configuration keys (lowercase with underscores/dots)
    match RegexMatcher::new(r"^[a-z][a-z0-9_]*\.?[a-z0-9_]*$", "Config key format") {
        Ok(config_matcher) => {
            // Add some config-like entries
            data_cache.insert(
                "database_host".to_string(),
                "Database Host Config".to_string(),
            );
            data_cache.insert("api.key".to_string(), "API Key Config".to_string());
            data_cache.insert("log_level".to_string(), "Log Level Config".to_string());
            data_cache.insert("server.port".to_string(), "Server Port Config".to_string());

            println!("\n⚙️  Finding configuration-style keys:");
            let configs = data_cache.get_all_by_matcher(&config_matcher);

            for (key, value_obj) in configs {
                println!("   ✅ Config key: {} -> {}", key, value_obj.value());
            }
        }
        Err(e) => println!("❌ Config regex error: {}", e),
    }

    // Performance test with regex
    println!("\n🚀 Performance test with regex matching:");

    if let Ok(email_matcher) = RegexMatcher::email_pattern() {
        let iterations = 1000;
        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let _ = data_cache.get_by_matcher(&email_matcher);
        }

        let regex_time = start.elapsed();
        println!(
            "   Regex matching ({} iterations): {:?}",
            iterations, regex_time
        );
        println!("   Average per match: {:?}", regex_time / iterations);
    }

    // Compare with simple string matching
    let prefix_matcher = PrefixMatcher::new("alice");
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = data_cache.get_by_matcher(&prefix_matcher);
    }

    let prefix_time = start.elapsed();
    println!(
        "   Prefix matching ({} iterations): {:?}",
        iterations, prefix_time
    );

    // Complex regex for parsing structured data
    println!("\n🔬 Advanced regex example - parsing structured log entries:");

    // Add some log entries
    let log_entries = vec![
        "[2024-01-15 10:30:45] INFO: User alice logged in from 192.168.1.100",
        "[2024-01-15 10:31:02] ERROR: Database connection failed for user bob",
        "[2024-01-15 10:31:15] WARN: High memory usage detected: 85%",
        "[2024-01-15 10:31:30] DEBUG: Cache hit rate: 94.5%",
        "Invalid log format without timestamp",
    ];

    for (i, log) in log_entries.iter().enumerate() {
        data_cache.insert(format!("log_{}", i), log.to_string());
    }

    // Match log entries with timestamp, level, and message
    match RegexMatcher::new(
        r"^\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\] (INFO|ERROR|WARN|DEBUG): .+$",
        "Structured log entries",
    ) {
        Ok(log_matcher) => {
            println!("   Finding properly formatted log entries:");
            let logs = data_cache.get_all_by_matcher(&log_matcher);

            for (key, value_obj) in logs {
                println!("   ✅ {} -> {}", key, &value_obj.value()[..60]);
            }
        }
        Err(e) => println!("❌ Log regex error: {}", e),
    }

    println!("\n🎉 Regex matching example completed!");

    // Final statistics
    let final_stats = data_cache.stats();
    println!("\n📊 Final Cache Statistics:");
    println!("   Total entries: {}", final_stats.total_entries);
    println!("   Active entries: {}", final_stats.active_entries);
}

#[cfg(not(feature = "regex_support"))]
fn main() {
    println!("=== Regex Pattern Matching Example ===\n");
    println!("❌ This example requires the 'regex_support' feature to be enabled.");
    println!("\nTo run this example:");
    println!("   cargo run --example regex_matching --features regex_support");
    println!("\nOr add to your Cargo.toml:");
    println!("   [features]");
    println!("   default = [\"regex_support\"]");
    println!("   regex_support = [\"regex\"]");
    println!("\n   [dependencies]");
    println!("   regex = \"1.10\"");
}
