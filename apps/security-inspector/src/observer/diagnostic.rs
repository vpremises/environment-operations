//! Diagnostics disclose stable categories and root IDs without observed filesystem paths.

use crate::model::Diagnostic;

pub(crate) fn create(
    code: impl Into<String>,
    root_id: Option<&str>,
    field: impl Into<String>,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        root_id: root_id.map(ToOwned::to_owned),
        field: field.into(),
        message: message.into(),
    }
}

pub(crate) fn io(code: &str, root_id: &str, field: &str, error: &std::io::Error) -> Diagnostic {
    create(
        code,
        Some(root_id),
        field,
        format!("filesystem operation failed: {}", error.kind()),
    )
}

pub(crate) fn order(left: &Diagnostic, right: &Diagnostic) -> std::cmp::Ordering {
    (&left.root_id, &left.field, &left.code, &left.message).cmp(&(
        &right.root_id,
        &right.field,
        &right.code,
        &right.message,
    ))
}
