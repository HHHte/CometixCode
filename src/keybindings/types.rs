//! Maps to: CC `keybindings/types.ts` — core keybinding data shapes.

/// Maps to: CC `ParsedKeystroke`. Six modifier booleans plus a normalized
/// key name ("escape", "enter", " ", "a", "f1", ...).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedKeystroke {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    /// cmd/win — distinct from alt/meta; only arrives via kitty keyboard
    /// protocol.
    pub super_key: bool,
}

/// Maps to: CC `Chord` — a multi-key sequence, e.g. `ctrl+x ctrl+k`.
pub type Chord = Vec<ParsedKeystroke>;

/// Maps to: CC `KeybindingAction` (string) with `null` = explicit unbind.
pub type KeybindingAction = Option<String>;

/// Maps to: CC `KeybindingContextName`. Contexts are flat filter tags, not
/// nested scopes; `Global` participates in every resolution.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ContextName {
    Global,
    Chat,
    Autocomplete,
    Settings,
    Confirmation,
    Tabs,
    Transcript,
    HistorySearch,
    Task,
    ThemePicker,
    Scroll,
    Help,
    Attachments,
    Footer,
    MessageSelector,
    DiffDialog,
    ModelPicker,
    Select,
    Plugin,
    MessageActions,
    VimNormal,
    VimInsert,
    VimVisual,
    Custom(String),
}

impl ContextName {
    /// Maps to: CC `KeybindingContextName` string literals.
    pub fn from_official_str(value: &str) -> Self {
        match value {
            "Global" => Self::Global,
            "Chat" => Self::Chat,
            "Autocomplete" => Self::Autocomplete,
            "Settings" => Self::Settings,
            "Confirmation" => Self::Confirmation,
            "Tabs" => Self::Tabs,
            "Transcript" => Self::Transcript,
            "HistorySearch" => Self::HistorySearch,
            "Task" => Self::Task,
            "ThemePicker" => Self::ThemePicker,
            "Scroll" => Self::Scroll,
            "Help" => Self::Help,
            "Attachments" => Self::Attachments,
            "Footer" => Self::Footer,
            "MessageSelector" => Self::MessageSelector,
            "DiffDialog" => Self::DiffDialog,
            "ModelPicker" => Self::ModelPicker,
            "Select" => Self::Select,
            "Plugin" => Self::Plugin,
            "MessageActions" => Self::MessageActions,
            "VimNormal" => Self::VimNormal,
            "VimInsert" => Self::VimInsert,
            "VimVisual" => Self::VimVisual,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Maps to: CC `KeybindingContextName` display/storage string.
    pub fn as_official_str(&self) -> &str {
        match self {
            Self::Global => "Global",
            Self::Chat => "Chat",
            Self::Autocomplete => "Autocomplete",
            Self::Settings => "Settings",
            Self::Confirmation => "Confirmation",
            Self::Tabs => "Tabs",
            Self::Transcript => "Transcript",
            Self::HistorySearch => "HistorySearch",
            Self::Task => "Task",
            Self::ThemePicker => "ThemePicker",
            Self::Scroll => "Scroll",
            Self::Help => "Help",
            Self::Attachments => "Attachments",
            Self::Footer => "Footer",
            Self::MessageSelector => "MessageSelector",
            Self::DiffDialog => "DiffDialog",
            Self::ModelPicker => "ModelPicker",
            Self::Select => "Select",
            Self::Plugin => "Plugin",
            Self::MessageActions => "MessageActions",
            Self::VimNormal => "VimNormal",
            Self::VimInsert => "VimInsert",
            Self::VimVisual => "VimVisual",
            Self::Custom(value) => value.as_str(),
        }
    }
}

/// Maps to: CC `KeybindingBlock` while retaining insertion order for the
/// generated `keybindings.json` template.
#[derive(Clone, Debug, PartialEq)]
pub struct KeybindingBlock {
    pub context: ContextName,
    pub bindings: Vec<(String, KeybindingAction)>,
    /// Rust L1 action projections participate in runtime resolution but are
    /// excluded from CC's generated DEFAULT_BINDINGS template.
    pub include_in_template: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeybindingWarningSeverity {
    #[default]
    Warning,
    Error,
}

/// Maps to: CC `ParsedBinding`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedBinding {
    pub chord: Chord,
    pub action: KeybindingAction,
    pub context: ContextName,
}

/// Maps to: CC resolver `ChordResolveResult` (5 variants).
#[derive(Clone, Debug, PartialEq)]
pub enum ChordResolveResult {
    /// Full chord completed — fire the action.
    Match { action: String },
    /// No binding matched — the event propagates normally.
    None,
    /// Explicitly unbound (action = null) — swallow the event.
    Unbound,
    /// Partial chord — waiting for the next key (1s timeout upstream).
    ChordStarted { pending: Vec<ParsedKeystroke> },
    /// Invalid key during a chord, or Escape pressed mid-chord.
    ChordCancelled,
}
