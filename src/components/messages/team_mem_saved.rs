//! Maps to: CC `components/messages/teamMemSaved.ts:1-19`.
//!
//! This remains a plain helper rather than an iocraft component, matching the
//! official compiler-hoisting boundary behind the TEAMMEM build feature.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamMemSavedPart {
    pub segment: String,
    pub count: usize,
}

pub fn team_mem_saved_part(team_count: Option<usize>) -> Option<TeamMemSavedPart> {
    let count = team_count.unwrap_or(0);
    if count == 0 {
        return None;
    }
    Some(TeamMemSavedPart {
        segment: format!(
            "{count} team {}",
            if count == 1 { "memory" } else { "memories" }
        ),
        count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_absent_and_counts_pluralize_exactly() {
        assert_eq!(team_mem_saved_part(None), None);
        assert_eq!(team_mem_saved_part(Some(0)), None);
        assert_eq!(
            team_mem_saved_part(Some(1)),
            Some(TeamMemSavedPart {
                segment: "1 team memory".to_string(),
                count: 1,
            })
        );
        assert_eq!(
            team_mem_saved_part(Some(2)).unwrap().segment,
            "2 team memories"
        );
    }
}
