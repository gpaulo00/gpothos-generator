use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

/// Container for manually defined resolvers found during scanning
#[derive(Debug, Default)]
pub struct ManualResolvers {
    pub queries: HashSet<String>,
    pub mutations: HashSet<String>,
}

impl ManualResolvers {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if a query with the given name exists
    pub fn contains_query(&self, name: &str) -> bool {
        self.queries.contains(name)
    }
    
    /// Check if a mutation with the given name exists
    pub fn contains_mutation(&self, name: &str) -> bool {
        self.mutations.contains(name)
    }
}

/// Scan specified directories for manually defined queries and mutations
pub fn scan_for_manual_resolvers(
    scan_dirs: &[String],
    verbose: bool,
) -> Result<ManualResolvers> {
    let mut resolvers = ManualResolvers::new();
    
    if scan_dirs.is_empty() {
        if verbose {
            println!("ℹ️  No scan directories specified, skipping manual resolver detection");
        }
        return Ok(resolvers);
    }
    
    // Compile regex patterns once for performance
    // Matches: builder.queryField("name", ...) with flexible whitespace
    let query_re = Regex::new(
        r#"builder\s*\.\s*queryField\s*\(\s*["'`]([^"'`]+)["'`]"#
    )?;
    
    // Matches: builder.mutationField("name", ...) with flexible whitespace
    let mutation_re = Regex::new(
        r#"builder\s*\.\s*mutationField\s*\(\s*["'`]([^"'`]+)["'`]"#
    )?;
    
    for dir in scan_dirs {
        if !std::path::Path::new(dir).exists() {
            if verbose {
                println!("⚠️  Directory not found: {}", dir);
            }
            continue;
        }
        
        if verbose {
            println!("🔍 Scanning directory: {}", dir);
        }
        
        let mut files_scanned = 0;
        
        // Walk through directory recursively
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Only process .ts files
                e.path().extension().and_then(|s| s.to_str()) == Some("ts")
            })
        {
            let path = entry.path();
            
            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };
            
            // Quick check: skip files that don't contain "builder"
            if !content.contains("builder") {
                continue;
            }
            
            // Search for query fields
            for cap in query_re.captures_iter(&content) {
                let query_name = cap[1].to_string();
                if verbose {
                    println!("  ✓ Found manual query: {} in {:?}", query_name, path);
                }
                resolvers.queries.insert(query_name);
            }
            
            // Search for mutation fields
            for cap in mutation_re.captures_iter(&content) {
                let mutation_name = cap[1].to_string();
                if verbose {
                    println!("  ✓ Found manual mutation: {} in {:?}", mutation_name, path);
                }
                resolvers.mutations.insert(mutation_name);
            }
            
            files_scanned += 1;
        }
        
        if verbose {
            println!("  📁 Scanned {} TypeScript files", files_scanned);
        }
    }
    
    if verbose {
        println!(
            "✅ Found {} manual queries and {} manual mutations",
            resolvers.queries.len(),
            resolvers.mutations.len()
        );
    }
    
    Ok(resolvers)
}
