//! Maps to: CC `components/messages/teamMemCollapsed.tsx`.

use crate::types::message::CollapsedReadSearchGroup;

pub fn check_has_team_mem_ops(message: &CollapsedReadSearchGroup) -> bool {
    message.team_memory_search_count > 0
        || message.team_memory_read_count > 0
        || message.team_memory_write_count > 0
}

fn push_count(
    parts: &mut Vec<String>,
    active: bool,
    count: usize,
    active_first: &'static str,
    active_next: &'static str,
    past_first: &'static str,
    past_next: &'static str,
    singular: &'static str,
    plural: &'static str,
) {
    if count == 0 {
        return;
    }
    let first = parts.is_empty();
    let verb = if active {
        if first { active_first } else { active_next }
    } else if first {
        past_first
    } else {
        past_next
    };
    parts.push(format!(
        "{verb} {count} {}",
        if count == 1 { singular } else { plural }
    ));
}

pub fn push_team_mem_count_parts(message: &CollapsedReadSearchGroup, parts: &mut Vec<String>) {
    push_count(
        parts,
        message.active,
        message.team_memory_read_count,
        "Recalling",
        "recalling",
        "Recalled",
        "recalled",
        "team memory",
        "team memories",
    );
    if message.team_memory_search_count > 0 {
        let first = parts.is_empty();
        parts.push(
            if message.active {
                if first {
                    "Searching team memories"
                } else {
                    "searching team memories"
                }
            } else if first {
                "Searched team memories"
            } else {
                "searched team memories"
            }
            .to_string(),
        );
    }
    push_count(
        parts,
        message.active,
        message.team_memory_write_count,
        "Writing",
        "writing",
        "Wrote",
        "wrote",
        "team memory",
        "team memories",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_memory_parts_preserve_case_order_and_pluralization() {
        let mut message = CollapsedReadSearchGroup {
            team_memory_read_count: 2,
            team_memory_search_count: 1,
            team_memory_write_count: 1,
            ..Default::default()
        };
        let mut parts = Vec::new();
        push_team_mem_count_parts(&message, &mut parts);
        assert_eq!(
            parts,
            [
                "Recalled 2 team memories",
                "searched team memories",
                "wrote 1 team memory"
            ]
        );
        message.active = true;
        let mut parts = vec!["Read 1 file".to_string()];
        push_team_mem_count_parts(&message, &mut parts);
        assert_eq!(parts[1], "recalling 2 team memories");
    }
}
