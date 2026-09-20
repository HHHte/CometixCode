//! Maps to: CC `keybindings/` — declarative action-routing for keyboard
//! input.
//!
//! Layering (mirrors CC):
//!   - `types` — ParsedKeystroke/Chord/ParsedBinding/ContextName (types.ts)
//!   - `parser` — "ctrl+x ctrl+k" string parsing + display names (parser.ts)
//!   - `matcher` — crossterm KeyEvent → ParsedKeystroke bridge (match.ts)
//!   - `default_bindings` — default context × action table (defaultBindings.ts)
//!   - `reserved_shortcuts` — reserved shortcut metadata (reservedShortcuts.ts)
//!   - `resolver` — pure chord state machine, last-wins overrides (resolver.ts)
//!   - `keybinding_context` — runtime registry (KeybindingContext.tsx)
//!   - `keybinding_provider_setup` — interceptor/setup (KeybindingProviderSetup.tsx)
//!   - `use_keybinding` — per-component action handler hook (useKeybinding.ts)
//!   - `shortcut_format` — configured shortcut display fallback
//!     (shortcutFormat.ts / useShortcutDisplay.ts)

pub mod default_bindings;
pub mod keybinding_context;
pub mod keybinding_provider_setup;
pub mod load_user_bindings;
pub mod matcher;
pub mod parser;
pub mod reserved_shortcuts;
pub mod resolver;
pub mod schema;
pub mod shortcut_format;
pub mod template;
pub mod types;
pub mod use_keybinding;
pub mod validate;
