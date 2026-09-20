//! Maps to: CC `types/notebook.ts`.
//!
//! Minimal Jupyter notebook data shapes used by the notebook edit permission
//! preview. The official TypeScript file is type-only; this Rust projection is
//! intentionally serde-friendly so UI code can safely inspect saved `.ipynb`
//! JSON without executing notebook logic.

use serde::{Deserialize, Serialize};

/// Maps to: CC `types/notebook.ts#NotebookCellType`.
pub type NotebookCellType = String;

/// Maps to: CC `types/notebook.ts#NotebookCell.source`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotebookCellSource {
    Text(String),
    Lines(Vec<String>),
}

impl Default for NotebookCellSource {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl NotebookCellSource {
    pub fn joined(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Lines(lines) => lines.join(""),
        }
    }
}

/// Maps to: CC `types/notebook.ts#NotebookCell`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: Option<String>,
    #[serde(default)]
    pub cell_type: NotebookCellType,
    #[serde(default)]
    pub source: NotebookCellSource,
    #[serde(default)]
    pub execution_count: Option<u64>,
    #[serde(default)]
    pub outputs: Vec<serde_json::Value>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Maps to: CC `types/notebook.ts#NotebookContent`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookContent {
    #[serde(default)]
    pub cells: Vec<NotebookCell>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub nbformat: u64,
    #[serde(default)]
    pub nbformat_minor: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notebook_cell_source_joins_array_sources_like_official_types() {
        let notebook: NotebookContent = serde_json::from_value(serde_json::json!({
            "cells": [{
                "id": "cell-a",
                "cell_type": "code",
                "source": ["print(", "'hi')\n"]
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }))
        .unwrap();

        assert_eq!(notebook.cells[0].source.joined(), "print('hi')\n");
    }
}
