use super::*;

pub(super) fn environment_selection_error_message(err: AnecdoctErr) -> String {
    match err {
        AnecdoctErr::InvalidRequest(message) => message,
        err => err.to_string(),
    }
}
