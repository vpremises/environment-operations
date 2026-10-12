//! Existing v1 repository policies remain data, with exact-path exceptions.
use serde::{Deserialize, Serialize};

/// Repository-local OSS policy; never grants publication or relicensing rights.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OssPolicy {
    /// Supported policy version.
    pub version: u32,
    /// Exact expected manifest license, including downstream license expressions.
    pub license: String,
    /// Reviewed archive filenames; wildcard exceptions are not accepted.
    #[serde(default)]
    pub allowed_vendor_archives: Vec<String>,
    /// Reviewed source filenames; never a whole-directory exclusion.
    #[serde(default)]
    pub allowed_source_files: Vec<String>,
    /// Additional source suffixes forbidden by an explicit repository policy.
    #[serde(default)]
    pub forbidden_extensions: Vec<String>,
    /// Optional Rust layout and comment policy.
    #[serde(default)]
    pub rust_source: Option<RustSource>,
}

/// Explicit source layout rules; absent rules are not inferred.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RustSource {
    /// Maximum physical lines per Rust source file.
    pub max_lines: usize,
    /// Require English ASCII in source and comments.
    pub english_ascii: bool,
    /// Relative maintained source directories.
    pub directories: Vec<String>,
}

impl OssPolicy {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1 || self.license.trim().is_empty() {
            return Err("policy-invalid");
        }
        for name in self
            .allowed_vendor_archives
            .iter()
            .chain(&self.allowed_source_files)
        {
            if !super::paths::relative(name) || name.contains(['*', '?', '[']) {
                return Err("policy-exception-invalid");
            }
        }
        if self.forbidden_extensions.iter().any(|s| {
            !s.starts_with('.')
                || s.len() > 64
                || s.contains(['/', '\\', '*', '?', '[', ':'])
                || s.len() < 2
        }) {
            return Err("policy-extension-invalid");
        }
        if let Some(source) = &self.rust_source {
            if !(1..=10_000).contains(&source.max_lines)
                || source.directories.is_empty()
                || source
                    .directories
                    .iter()
                    .any(|p| !super::paths::relative(p))
            {
                return Err("source-policy-invalid");
            }
        }
        Ok(())
    }
}
