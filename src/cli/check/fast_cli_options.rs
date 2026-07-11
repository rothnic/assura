//! Small parser helpers kept outside the latency-sensitive CLI entrypoint.

use std::ffi::OsString;

pub(super) fn reject_unknown_option(value: &OsString) -> Result<(), String> {
    let Some(value) = value.to_str() else {
        return Ok(());
    };
    if value.starts_with('-') {
        return Err(format!("unexpected argument {value:?}"));
    }
    Ok(())
}
