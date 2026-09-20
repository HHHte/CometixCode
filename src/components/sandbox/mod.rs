//! Maps to: CC `components/sandbox/`.
//! Sandbox command UI components. Runtime sandbox initialization and command
//! wrapping remain in `utils/sandbox/*`; these components render official tabs
//! and dispatch official settings-update intents.

pub mod sandbox_config_tab;
pub mod sandbox_dependencies_tab;
pub mod sandbox_doctor_section;
pub mod sandbox_overrides_tab;
pub mod sandbox_settings;

pub use sandbox_doctor_section::SandboxDoctorSection;
pub use sandbox_settings::SandboxSettings;
