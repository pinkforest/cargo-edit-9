use std::fmt::Display;

pub use cargo_edit_9::CargoResult;
pub use cargo_edit_9::CliResult;
pub use cargo_edit_9::Context;
pub use cargo_edit_9::Error;

/// User requested to downgrade a crate
pub(crate) fn version_downgrade_err(current: impl Display, requested: impl Display) -> Error {
    anyhow::format_err!("Cannot downgrade from {} to {}", current, requested)
}
