//! This module defines the [IgnoreGlobs]. To set it up from [Cli], a [Config] and its
//! [Default] value, use the [configure_from](IgnoreGlobs::configure_from) method.

use crate::app::Cli;
use crate::config_file::Config;

use clap::error::ErrorKind;
use clap::Error;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

/// The struct holding optimized glob matching structures.
/// Uses HashSets for O(1) extension and exact name lookups,
/// falling back to GlobSet only for complex patterns.
#[derive(Clone, Debug)]
pub struct IgnoreGlobs {
    extensions: HashSet<String>,
    exact_names: HashSet<String>,
    complex_globs: GlobSet,
}

impl IgnoreGlobs {
    /// Returns a value from either [Cli], a [Config] or a [Default] value. The first value
    /// that is not [None] is used. The order of precedence for the value used is:
    /// - [from_cli](IgnoreGlobs::from_cli)
    /// - [from_config](IgnoreGlobs::from_config)
    /// - [Default::default]
    ///
    /// # Errors
    ///
    /// If either of the [Glob::new] or [GlobSetBuilder.build] methods return an [Err].
    pub fn configure_from(cli: &Cli, config: &Config) -> Result<Self, Error> {
        if let Some(value) = Self::from_cli(cli) {
            return value;
        }

        if let Some(value) = Self::from_config(config) {
            return value;
        }

        Ok(Default::default())
    }

    /// Build IgnoreGlobs from an iterator of pattern strings.
    /// 
    /// Classifies each pattern into extensions, exact names, or complex globs
    /// for optimized O(1) or O(k) matching where k << total patterns.
    fn from_patterns<'a, I>(patterns: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut extensions = HashSet::new();
        let mut exact_names = HashSet::new();
        let mut complex_builder = GlobSetBuilder::new();

        for pattern in patterns {
            match Self::classify_pattern(pattern)? {
                PatternType::Extension(ext) => {
                    extensions.insert(ext);
                }
                PatternType::ExactName(name) => {
                    exact_names.insert(name);
                }
                PatternType::Complex(glob) => {
                    complex_builder.add(glob);
                }
            }
        }

        let complex_globs = complex_builder
            .build()
            .map_err(|err| Error::raw(ErrorKind::ValueValidation, err))?;

        Ok(Self {
            extensions,
            exact_names,
            complex_globs,
        })
    }

    /// Get a potential [IgnoreGlobs] from [Cli].
    ///
    /// If the "ignore-glob" argument has been passed, this returns a [Result] in a [Some] with
    /// either the built [IgnoreGlobs] or an [Error], if any error was encountered while creating the
    /// [IgnoreGlobs]. If the argument has not been passed, this returns [None].
    fn from_cli(cli: &Cli) -> Option<Result<Self, Error>> {
        if cli.ignore_glob.is_empty() {
            return None;
        }

        Some(Self::from_patterns(cli.ignore_glob.iter().map(String::as_str)))
    }

    /// Get a potential [IgnoreGlobs] from a [Config].
    ///
    /// If the `Config::ignore-globs` contains an Array of Strings,
    /// each of its values is used to build the [GlobSet]. If the building
    /// succeeds, the [IgnoreGlobs] is returned in the [Result] in a [Some]. If any error is
    /// encountered while building, an [Error] is returned in the Result instead. If the Config does
    /// not contain such a key, this returns [None].
    fn from_config(config: &Config) -> Option<Result<Self, Error>> {
        let globs = config.ignore_globs.as_ref()?;
        Some(Self::from_patterns(globs.iter().map(String::as_str)))
    }

    /// Create a [Glob] from a provided pattern.
    ///
    /// This method is mainly a helper to wrap the handling of potential errors.
    fn create_glob(pattern: &str) -> Result<Glob, Error> {
        Glob::new(pattern).map_err(|err| Error::raw(ErrorKind::ValueValidation, err))
    }



    /// Classify a glob pattern into one of three categories for optimized matching.
    ///
    /// - Extension patterns (*.ext) → extracted extension for O(1) HashSet lookup
    /// - Exact names (no wildcards) → exact string for O(1) HashSet lookup
    /// - Complex patterns → Glob for full regex matching
    fn classify_pattern(pattern: &str) -> Result<PatternType, Error> {
        if pattern.starts_with("*.") && !pattern[2..].contains(['*', '?', '[', ']', '.']) {
            // Simple extension pattern like "*.jpg" -> extract "jpg" (lowercase for case-insensitive)
            // Note: Multi-dot patterns like "*.tar.gz" are excluded (contain '.') because
            // Path::extension() only returns the last component ("gz" not "tar.gz")
            Ok(PatternType::Extension(pattern[2..].to_lowercase()))
        } else if !pattern.contains(['*', '?', '[', ']']) {
            // No glob metacharacters = exact match (preserve case)
            Ok(PatternType::ExactName(pattern.to_string()))
        } else {
            // Complex pattern needs full glob matching
            Self::create_glob(pattern).map(PatternType::Complex)
        }
    }

    /// Optimized glob matching using fast paths for extensions and exact names.
    ///
    /// Performance: O(1) for extensions and exact names, O(k) for complex patterns where k << 147.
    pub fn is_match(&self, name: &OsStr) -> bool {
        let name_str = name.to_string_lossy();
        
        // Fast path 1: Extension check (O(1))
        // Most files have extensions, check this first
        if let Some(ext) = Path::new(name_str.as_ref()).extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if self.extensions.contains(&ext_lower) {
                return true;
            }
        }
        
        // Fast path 2: Exact name check (O(1))
        // Check for exact directory/file name matches
        if self.exact_names.contains(name_str.as_ref()) {
            return true;
        }
        
        // Slow path: Complex patterns (~15 patterns instead of 147)
        self.complex_globs.is_match(name)
    }
}

/// Pattern classification types for optimized matching
enum PatternType {
    Extension(String),
    ExactName(String),
    Complex(Glob),
}

/// The default value of `IgnoreGlobs` contains patterns for common build directories
/// and large files that are typically not useful for LLM-assisted coding.
impl Default for IgnoreGlobs {
    fn default() -> Self {
        // Comprehensive ignore patterns based on scrypt/code2term.rs
        let patterns = [
            // Version control directories
            ".git",
            ".svn", 
            ".hg",
            ".bzr",
            
            // Build and dependency directories
            "node_modules",
            "target",           // Rust
            "dist",
            "build",
            "vendor",           // Go, PHP, etc.
            "out",
            ".next",            // Next.js
            ".nuxt",            // Nuxt.js
            ".output",          // Various build tools
            "_build",           // Documentation builds
            "site",             // Documentation sites
            
            // Python
            "__pycache__",
            "*.pyc",
            "*.pyo",
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            ".hypothesis",
            "venv",
            ".venv",
            "env",
            ".env",
            "*.egg-info",
            
            // IDE and editor files
            ".idea",
            ".vscode",
            "*.swp",
            "*.swo",
            "*~",
            ".DS_Store",
            "Thumbs.db",
            
            // Package managers
            ".yarn",
            ".pnp.*",
            ".npm",
            
            // Coverage and test reports
            "coverage",
            ".coverage",
            "*.cover",
            ".nyc_output",
            "*.lcov",
            
            // Compiled object files
            "*.o",
            "*.so",
            "*.dll",
            "*.exe",
            "*.bin",
            "*.class",          // Java
            
            // Logs and databases
            "*.log",
            "*.sqlite",
            "*.db",
            
            // Lock files (often large and not needed for reading)
            "*.lock",
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",       // Can be large in workspaces
            "poetry.lock",
            "Pipfile.lock",
            
            // Archives and compressed files
            "*.zip",
            "*.tar",
            "*.tar.gz",
            "*.tar.bz2",
            "*.tar.xz",
            "*.rar",
            "*.7z",
            "*.gz",
            "*.bz2",
            "*.xz",
            
            // Large binary/data files
            "*.iso",
            "*.dmg",
            "*.pkg",
            "*.deb",
            "*.rpm",
            "*.msi",
            "*.exe",
            "*.app",
            
            // Media files
            // Images
            "*.jpg",
            "*.jpeg",
            "*.png",
            "*.gif",
            "*.bmp",
            "*.ico",
            "*.svg",
            "*.webp",
            "*.tiff",
            "*.tif",
            "*.psd",
            "*.ai",
            "*.eps",
            
            // Videos
            "*.mp4",
            "*.mov",
            "*.avi",
            "*.mkv",
            "*.webm",
            "*.flv",
            "*.wmv",
            "*.mpg",
            "*.mpeg",
            "*.m4v",
            "*.3gp",
            
            // Audio
            "*.mp3",
            "*.wav",
            "*.ogg",
            "*.flac",
            "*.aac",
            "*.wma",
            "*.m4a",
            "*.opus",
            
            // Documents
            "*.pdf",
            "*.docx",
            "*.doc",
            "*.xlsx",
            "*.xls",
            "*.pptx",
            "*.ppt",
            "*.odt",
            "*.ods",
            "*.odp",
            
            // Data files
            "*.pkl",            // Python pickle
            "*.npy",            // NumPy
            "*.npz",            
            "*.parquet",        // Apache Parquet
            "*.hdf5",           // HDF5
            "*.h5",
            "*.mat",            // MATLAB
            "*.feather",        // Feather format
            "*.msgpack",        // MessagePack
            
            // Other common excludes
            ".cache",
            ".parcel-cache",
            ".turbo",
            ".vercel",
            ".netlify",
            ".serverless",
            ".terraform",
            "*.min.js",
            "*.min.css",
            "*.map",            // Source maps
            ".sass-cache",
            ".gradle",
            ".m2",              // Maven
            ".stack-work",      // Haskell Stack
            ".cabal-sandbox",   // Haskell Cabal
            "bower_components", // Bower
            "jspm_packages",    // JSPM
            ".pnp",             // Yarn PnP
            "*.pid",
            "*.seed",
            "*.pid.lock",
        ];
        
        // Classify patterns for optimized matching
        let mut extensions = HashSet::new();
        let mut exact_names = HashSet::new();
        let mut complex_builder = GlobSetBuilder::new();
        
        for pattern in patterns {
            match Self::classify_pattern(pattern) {
                Ok(PatternType::Extension(ext)) => {
                    extensions.insert(ext);
                }
                Ok(PatternType::ExactName(name)) => {
                    exact_names.insert(name);
                }
                Ok(PatternType::Complex(glob)) => {
                    complex_builder.add(glob);
                }
                Err(_) => {} // Skip invalid patterns (should not happen with hardcoded patterns)
            }
        }
        
        // Build complex globs GlobSet, use empty set if build fails
        let complex_globs = complex_builder.build().unwrap_or_else(|_| GlobSet::empty());
        
        Self {
            extensions,
            exact_names,
            complex_globs,
        }
    }
}

#[cfg(test)]
mod test {
    use clap::Parser;

    use super::IgnoreGlobs;

    use crate::app::Cli;
    use crate::config_file::Config;

    // The following tests are implemented using match expressions instead of the assert_eq macro,
    // because clap::Error does not implement PartialEq.
    //
    // Further no tests for actually returned GlobSets are implemented, because GlobSet does not
    // even implement PartialEq and thus can not be easily compared.

    #[test]
    fn test_configuration_from_none() {
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert!(matches!(
            IgnoreGlobs::configure_from(&cli, &Config::with_none()),
            Ok(..)
        ));
    }

    #[test]
    fn test_configuration_from_args() {
        let argv = ["lsd", "--ignore-glob", ".git"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert!(matches!(
            IgnoreGlobs::configure_from(&cli, &Config::with_none()),
            Ok(..)
        ));
    }

    #[test]
    fn test_configuration_from_config() {
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        let mut c = Config::with_none();
        c.ignore_globs = Some(vec![".git".into()]);
        assert!(matches!(IgnoreGlobs::configure_from(&cli, &c), Ok(..)));
    }

    #[test]
    fn test_from_cli_none() {
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert!(IgnoreGlobs::from_cli(&cli).is_none());
    }

    #[test]
    fn test_from_config_none() {
        assert!(IgnoreGlobs::from_config(&Config::with_none()).is_none());
    }

    #[test]
    fn test_pattern_classification() {
        use std::ffi::OsStr;
        
        let globs = IgnoreGlobs::default();
        
        // Test extension matching (should hit fast path)
        assert!(globs.is_match(OsStr::new("test.jpg")));
        assert!(globs.is_match(OsStr::new("file.PNG"))); // Case insensitive
        assert!(globs.is_match(OsStr::new("archive.tar.gz")));
        
        // Test exact name matching (should hit fast path)
        assert!(globs.is_match(OsStr::new(".git")));
        assert!(globs.is_match(OsStr::new("node_modules")));
        assert!(globs.is_match(OsStr::new("target")));
        
        // Test files that should NOT match
        assert!(!globs.is_match(OsStr::new("README.md")));
        assert!(!globs.is_match(OsStr::new("src")));
        assert!(!globs.is_match(OsStr::new("main.rs")));
    }

    #[test]
    #[ignore] // Run with: cargo test test_performance_comparison -- --ignored --nocapture
    fn test_performance_comparison() {
        use std::ffi::OsStr;
        use std::time::Instant;
        
        let globs = IgnoreGlobs::default();
        
        // Test files that hit different code paths
        let test_files = vec![
            OsStr::new("file.jpg"),      // Extension fast path
            OsStr::new("image.PNG"),      // Extension fast path (case insensitive)
            OsStr::new(".git"),           // Exact name fast path
            OsStr::new("node_modules"),   // Exact name fast path
            OsStr::new("test.rs"),        // No match (goes through all paths)
            OsStr::new("README.md"),      // No match
        ];
        
        let iterations = 100_000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            for file in &test_files {
                let _ = globs.is_match(file);
            }
        }
        
        let duration = start.elapsed();
        let total_ops = iterations * test_files.len() as u128;
        let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
        
        println!("\nPerformance Results:");
        println!("  Total operations: {}", total_ops);
        println!("  Total time: {:?}", duration);
        println!("  Operations/sec: {:.0}", ops_per_sec);
        println!("  Avg time per operation: {:.2?}", duration / total_ops as u32);
        
        // With the optimization, we expect > 5M ops/sec on modern hardware
        // Old implementation with 147 glob patterns would be much slower
        assert!(ops_per_sec > 1_000_000.0, 
            "Performance too slow: {} ops/sec", ops_per_sec);
    }
}
