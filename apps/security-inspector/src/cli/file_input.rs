//! All CLI documents use the library's directory-anchored, bounded input reader.
use std::path::Path;

pub(super) fn read_utf8(path: &Path, maximum: u64) -> Result<String, &'static str> {
    String::from_utf8(vpremises::read_local_document(path, maximum)?)
        .map_err(|_| "file must contain UTF-8")
}
