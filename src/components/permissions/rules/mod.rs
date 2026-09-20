//! Maps to: CC `components/permissions/rules/`.
//!
//! Official permission-rule management component boundaries. Runtime persistence
//! remains outside these components; helpers produce official-shaped updates and
//! render-state so callers can apply/persist them through the permission flow.

pub mod add_permission_rules;
pub mod add_workspace_directory;
pub mod permission_rule_description;
pub mod permission_rule_input;
pub mod permission_rule_list;
pub mod recent_denials_tab;
pub mod remove_workspace_directory;
pub mod workspace_tab;

pub use permission_rule_description::PermissionRuleDescription;
pub use permission_rule_list::{PermissionRuleList, RuleDetails};
