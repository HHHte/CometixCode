//! Maps to: CC `utils/sinks.ts`.

/// Maps to: CC `utils/sinks.ts:13-16` `initSinks`.
pub fn init_sinks() -> std::io::Result<()> {
    crate::utils::error_log_sink::initialize_error_log_sink()?;
    // CC: initializeAnalyticsSink(). Telemetry backends remain excluded under
    // PORTING.md; this source location does not enable a substitute endpoint.
    Ok(())
}
