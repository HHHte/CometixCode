//! Maps to: CC `commands/plugin/plugin.tsx`.

/// Typed JSX props returned by `call`; routing belongs to PluginSettings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSettingsData {
    pub args: Option<String>,
}

/// Maps to: CC `commands/plugin/plugin.tsx:5-11#call`.
pub fn call(args: Option<&str>) -> PluginSettingsData {
    PluginSettingsData {
        args: args.map(str::to_owned),
    }
}
