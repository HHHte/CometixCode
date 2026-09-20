//! Maps to: CC `components/CustomSelect/option-map.ts`.
//!
//! CC's OptionMap is a Map<value, item> whose items form a doubly linked
//! list (previous/next) with first/last anchors. The Rust port stores items
//! in a Vec and links by index — identical traversal semantics without
//! self-referential pointers. Values are Strings, matching the established
//! `SelectOptionData.value` shape.

use std::collections::HashMap;

/// Maps to: CC `OptionMapItem` (label/description stay on the caller's
/// option data; navigation only needs value + links).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionMapItem {
    pub value: String,
    pub index: usize,
    pub previous: Option<usize>,
    pub next: Option<usize>,
}

/// Maps to: CC `OptionMap`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptionMap {
    items: Vec<OptionMapItem>,
    index_by_value: HashMap<String, usize>,
}

impl OptionMap {
    pub fn new(values: impl IntoIterator<Item = String>) -> Self {
        let mut items: Vec<OptionMapItem> = Vec::new();
        let mut index_by_value = HashMap::new();
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                items[index - 1].next = Some(index);
            }
            index_by_value.insert(value.clone(), index);
            items.push(OptionMapItem {
                value,
                index,
                previous: index.checked_sub(1),
                next: None,
            });
        }
        Self {
            items,
            index_by_value,
        }
    }

    pub fn get(&self, value: &str) -> Option<&OptionMapItem> {
        self.index_by_value
            .get(value)
            .and_then(|index| self.items.get(*index))
    }

    pub fn item_at(&self, index: usize) -> Option<&OptionMapItem> {
        self.items.get(index)
    }

    /// Maps to: CC `OptionMap.first`.
    pub fn first(&self) -> Option<&OptionMapItem> {
        self.items.first()
    }

    /// Maps to: CC `OptionMap.last`.
    pub fn last(&self) -> Option<&OptionMapItem> {
        self.items.last()
    }

    /// Maps to: CC `Map.size`.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(values: &[&str]) -> OptionMap {
        OptionMap::new(values.iter().map(|value| value.to_string()))
    }

    #[test]
    fn option_map_links_items_like_official_linked_list() {
        let options = map(&["a", "b", "c"]);
        assert_eq!(options.size(), 3);
        assert_eq!(options.first().map(|item| item.value.as_str()), Some("a"));
        assert_eq!(options.last().map(|item| item.value.as_str()), Some("c"));

        let b = options.get("b").expect("b exists");
        assert_eq!(b.index, 1);
        assert_eq!(b.previous, Some(0));
        assert_eq!(b.next, Some(2));
        assert_eq!(options.get("a").and_then(|item| item.previous), None);
        assert_eq!(options.get("c").and_then(|item| item.next), None);
    }

    #[test]
    fn option_map_empty_has_no_anchors() {
        let options = map(&[]);
        assert!(options.is_empty());
        assert_eq!(options.first(), None);
        assert_eq!(options.last(), None);
    }
}
