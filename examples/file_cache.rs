// Simple file content caching with directory matching

use simple_cacher::*;
use std::time::Duration;

#[derive(Debug, Clone)]
struct FileContent {
    path: String,
    content: String,
    size: usize,
    last_modified: std::time::SystemTime,
}

impl FileContent {
    fn new(path: String, content: String) -> Self {
        let size = content.len();
        let last_modified = std::time::SystemTime::now();

        Self {
            path,
            content,
            size,
            last_modified,
        }
    }
}

// Custom matcher for finding files by directory
struct DirectoryMatcher {
    directory: String,
}

impl DirectoryMatcher {
    fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }
}

impl Matcher<String> for DirectoryMatcher {
    fn matches(&self, file_path: &String) -> bool {
        file_path.starts_with(&self.directory)
    }
}

// Custom matcher for finding files by extension
struct ExtensionMatcher {
    extension: String,
}

impl ExtensionMatcher {
    fn new(extension: &str) -> Self {
        Self {
            extension: extension.to_string(),
        }
    }
}

impl Matcher<String> for ExtensionMatcher {
    fn matches(&self, file_path: &String) -> bool {
        file_path.ends_with(&format!(".{}", self.extension))
    }
}

fn main() {
    println!("=== File Cache Example ===\n");

    // Create file cache with 5-minute TTL and max 1000 files
    let mut file_cache = SimpleCacher::with_max_size(
        Duration::from_secs(300), // 5 minutes
        1000,                     // max 1000 files
    );

    println!("📁 Caching file contents...\n");

    // Simulate caching various file types
    let files = vec![
        ("/src/main.rs", "fn main() { println!(\"Hello, world!\"); }"),
        ("/src/lib.rs", "pub mod cache; pub mod utils;"),
        ("/config/app.toml", "[database]\nurl = \"localhost:5432\""),
        ("/config/nginx.conf", "server { listen 80; }"),
        (
            "/docs/README.md",
            "# Project Documentation\n\nThis is the readme.",
        ),
        (
            "/docs/CHANGELOG.md",
            "# Changelog\n\n## v1.0.0\n- Initial release",
        ),
        ("/assets/style.css", "body { font-family: Arial; }"),
        ("/assets/script.js", "console.log('App loaded');"),
        (
            "/tests/unit.rs",
            "#[test] fn test_cache() { assert!(true); }",
        ),
        ("/logs/app.log", "2024-07-18 12:00:00 INFO Server started"),
    ];

    for (path, content) in files {
        let file_content = FileContent::new(path.to_string(), content.to_string());
        file_cache.insert(path.to_string(), file_content);
    }

    println!("✅ Cached {} files", file_cache.len());

    // Test exact file retrieval
    println!("\n🔍 Retrieving specific files:");

    match file_cache.get(&"/src/main.rs".to_string()) {
        Ok(file_obj) => {
            let file = file_obj.value();
            println!("✅ Found main.rs: {} bytes", file.size);
            println!("   Content: {}", file.content);
            println!("   Age: {:?}", file_obj.age());
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Find all files in /src directory
    println!("\n📂 Finding all files in /src directory:");
    let src_matcher = DirectoryMatcher::new("/src");
    let src_files = file_cache.get_all_by_matcher(&src_matcher);

    for (path, file_obj) in src_files {
        let file = file_obj.value();
        println!("✅ {}: {} bytes", path, file.size);
    }

    // Find all Rust files
    println!("\n🦀 Finding all .rs files:");
    let rust_matcher = ExtensionMatcher::new("rs");
    let rust_files = file_cache.get_all_by_matcher(&rust_matcher);

    for (path, file_obj) in rust_files {
        let file = file_obj.value();
        println!("✅ {}: {} bytes", path, file.size);
    }

    // Find all config files
    println!("\n⚙️  Finding all config files:");
    let config_matcher = DirectoryMatcher::new("/config");
    let config_files = file_cache.get_all_by_matcher(&config_matcher);

    for (path, file_obj) in config_files {
        let file = file_obj.value();
        println!("✅ {}: {} bytes", path, file.size);
    }

    // Demonstrate cache invalidation (file update simulation)
    println!("\n🔄 Simulating file update:");

    let updated_content = FileContent::new(
        "/src/main.rs".to_string(),
        "fn main() { println!(\"Hello, updated world!\"); }".to_string(),
    );

    file_cache.insert("/src/main.rs".to_string(), updated_content);

    match file_cache.get(&"/src/main.rs".to_string()) {
        Ok(file_obj) => {
            let file = file_obj.value();
            println!("✅ Updated main.rs: {} bytes", file.size);
            println!("   New content: {}", file.content);
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Performance test
    println!("\n🚀 Performance test:");

    let iterations = 10_000;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let path = format!("/test/file_{}.txt", i % 100);
        let _ = file_cache.get(&path);
    }

    let lookup_time = start.elapsed();
    println!("✅ {} file lookups in {:?}", iterations, lookup_time);
    println!("   Average: {:?} per lookup", lookup_time / iterations);

    // Cache statistics
    let stats = file_cache.stats();
    println!("\n📊 Cache Statistics:");
    println!("   Total files: {}", stats.total_entries);
    println!("   Active files: {}", stats.active_entries);
    println!("   Max capacity: {:?}", stats.max_size);

    // Calculate total cached size
    let mut total_size = 0;
    for (_, file_obj) in file_cache.iter_active() {
        total_size += file_obj.value().size;
    }

    println!(
        "   Total cached size: {} bytes ({:.2} KB)",
        total_size,
        total_size as f64 / 1024.0
    );

    println!("\n🎉 File cache example completed!");
}
