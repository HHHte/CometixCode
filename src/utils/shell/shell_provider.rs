//! Shell provider contract.
//!
//! Maps to: CC `utils/shell/shellProvider.ts:1-38`.

use std::path::{Path, PathBuf};

pub const SHELL_TYPES: &[ShellType] = &[ShellType::Bash, ShellType::PowerShell];
pub const DEFAULT_HOOK_SHELL: ShellType = ShellType::Bash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShellType {
    Bash,
    PowerShell,
}

impl ShellType {
    /// The literal CC's `SHELL_TYPES` tuple carries (`'bash' | 'powershell'`).
    pub fn as_str(self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::PowerShell => "powershell",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltExecCommand {
    pub command_string: String,
    pub cwd_file_path: PathBuf,
}

/// Maps to CC `ShellProvider`.
pub trait ShellProvider: Send + Sync {
    fn shell_type(&self) -> ShellType;
    fn shell_path(&self) -> &Path;
    fn detached(&self) -> bool;
    fn build_exec_command(
        &self,
        command: &str,
        id: &str,
        sandbox_tmp_dir: Option<&Path>,
        use_sandbox: bool,
    ) -> std::io::Result<BuiltExecCommand>;
    fn get_spawn_args(&self, command_string: &str) -> Vec<String>;
    fn get_environment_overrides(&self, command: &str) -> Vec<(String, String)>;
}
