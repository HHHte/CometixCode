//! Rust process entry — not a CC source file.
//!
//! Maps to: Node loading `entrypoints/cli.tsx` (`void main()`).
//! CC `main.tsx` lives at [`cometix_code::main`].

fn main() {
    if let Some(code) =
        cometix_code::utils::sandbox::sandbox_adapter::run_linux_seccomp_helper_if_requested()
    {
        cometix_code::utils::cleanup_registry::exit_process(code);
    }
    cometix_code::entrypoints::cli::run();
    cometix_code::utils::cleanup_registry::run_cleanup_functions_sync();
}
