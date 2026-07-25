pub mod auth;
pub mod ffi;
pub mod request_io;
pub mod server;
pub mod socket_path;

pub use limux_core::*;

pub const DEFAULT_HOST_LOG_FILE_NAME: &str = "limux-host.current.log";

pub fn current_build_info() -> BuildInfo {
    BuildInfo::from_compile_env(
        option_env!("LIMUX_BUILD_SHA"),
        option_env!("LIMUX_BUILD_DIRTY"),
        option_env!("LIMUX_BUILD_PROFILE"),
    )
}
