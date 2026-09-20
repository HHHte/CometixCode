//! Runtime projection inferred from CC new-agent wizard usage.
//! CC `new-agent-creation/types.ts` is an `@generated-stub` and is not treated
//! as an independent behavior source.

use crate::components::wizard::types::WizardData;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWizardFinal {
    pub agent_type: String,
    pub when_to_use: String,
    pub system_prompt: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

pub fn final_agent(data: &WizardData) -> Option<AgentWizardFinal> {
    serde_json::from_value(data.get("finalAgent")?.clone()).ok()
}

pub fn string(data: &WizardData, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_string)
}

pub fn strings(data: &WizardData, key: &str) -> Option<Vec<String>> {
    data.get(key).and_then(Value::as_array).map(|values| {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    })
}

pub fn update(key: &str, value: Value) -> WizardData {
    let mut data = Map::new();
    data.insert(key.to_string(), value);
    data
}
