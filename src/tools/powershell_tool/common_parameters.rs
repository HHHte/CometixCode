//! Maps to: CC `tools/PowerShellTool/commonParameters.ts`.
//!
//! PowerShell common parameters, available on every cmdlet via
//! `[CmdletBinding()]`. Shared between path validation (merged into the
//! per-cmdlet known-param sets) and read-only validation (merged into the
//! `safeFlags` check).
//!
//! Stored lowercase with a leading dash — callers lowercase their input.

/// Maps to: CC `commonParameters.ts:12#COMMON_SWITCHES`.
pub const COMMON_SWITCHES: [&str; 2] = ["-verbose", "-debug"];

/// Maps to: CC `commonParameters.ts:14-25#COMMON_VALUE_PARAMS`.
pub const COMMON_VALUE_PARAMS: [&str; 10] = [
    "-erroraction",
    "-warningaction",
    "-informationaction",
    "-progressaction",
    "-errorvariable",
    "-warningvariable",
    "-informationvariable",
    "-outvariable",
    "-outbuffer",
    "-pipelinevariable",
];

/// Maps to: CC `commonParameters.ts:27-30#COMMON_PARAMETERS`.
pub fn is_common_parameter(param_lower: &str) -> bool {
    COMMON_SWITCHES.contains(&param_lower) || COMMON_VALUE_PARAMS.contains(&param_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_parameters_are_the_union_of_switches_and_value_params() {
        assert!(is_common_parameter("-verbose"));
        assert!(is_common_parameter("-erroraction"));
        assert!(is_common_parameter("-pipelinevariable"));
        assert!(!is_common_parameter("-path"));
        assert!(!is_common_parameter("-whatif"));
    }
}
