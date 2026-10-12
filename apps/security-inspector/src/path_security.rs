//! Linux link classification keeps traversal inside selected mounted roots.
use std::fs::Metadata;
/// Skip symbolic links as exposed by the Linux mounted filesystem.
pub(crate) fn is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}
