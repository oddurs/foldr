//! Native operating-system adapters.
//!
//! Add filesystem operations to the relevant adapter without exposing terminal
//! rendering or command-line arguments to the core.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;
