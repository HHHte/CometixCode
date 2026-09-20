//! Maps to: CC `components/PrBadge.tsx`.

use iocraft::prelude::*;

/// Maps to: CC `utils/ghPrStatus.ts#PrReviewState` values consumed by PrBadge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrReviewState {
    Approved,
    ChangesRequested,
    Pending,
    Merged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrStatusColor {
    Success,
    Error,
    Warning,
    Merged,
}

#[derive(Default, Props)]
pub struct PrBadgeProps {
    pub number: u64,
    pub url: String,
    pub review_state: Option<PrReviewState>,
    pub bold: bool,
}

/// Maps to: CC `PrBadge.tsx#getPrStatusColor`.
pub fn get_pr_status_color(state: Option<PrReviewState>) -> Option<PrStatusColor> {
    match state {
        Some(PrReviewState::Approved) => Some(PrStatusColor::Success),
        Some(PrReviewState::ChangesRequested) => Some(PrStatusColor::Error),
        Some(PrReviewState::Pending) => Some(PrStatusColor::Warning),
        Some(PrReviewState::Merged) => Some(PrStatusColor::Merged),
        None => None,
    }
}

fn status_color(theme: &crate::utils::theme::Theme, color: PrStatusColor) -> Color {
    match color {
        PrStatusColor::Success => theme.success,
        PrStatusColor::Error => theme.error,
        PrStatusColor::Warning => theme.warning,
        PrStatusColor::Merged => theme.merged,
    }
}

/// Maps to: CC `components/PrBadge.tsx#PrBadge`.
#[component]
pub fn PrBadge(props: &PrBadgeProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let status = get_pr_status_color(props.review_state);
    let label = format!("#{}", props.number);
    let dim = status.is_none() && !props.bold;
    let color = status.map(|status| status_color(&theme, status));
    let weight = if props.bold {
        Weight::Bold
    } else {
        Weight::Normal
    };

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(content: "PR".to_string(), dim: !props.bold, wrap: TextWrap::NoWrap)
            Text(content: " ".to_string(), wrap: TextWrap::NoWrap)
            Text(
                content: label,
                href: Some(props.url.clone()),
                color: color,
                dim: dim,
                underline: true,
                weight: weight,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn pr_badge_status_colors_match_official_mapping() {
        assert_eq!(
            get_pr_status_color(Some(PrReviewState::Approved)),
            Some(PrStatusColor::Success)
        );
        assert_eq!(
            get_pr_status_color(Some(PrReviewState::ChangesRequested)),
            Some(PrStatusColor::Error)
        );
        assert_eq!(
            get_pr_status_color(Some(PrReviewState::Pending)),
            Some(PrStatusColor::Warning)
        );
        assert_eq!(
            get_pr_status_color(Some(PrReviewState::Merged)),
            Some(PrStatusColor::Merged)
        );
        assert_eq!(get_pr_status_color(None), None);
    }

    #[test]
    fn pr_badge_renders_label_and_hyperlink() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                PrBadge(number: 42u64, url: "https://github.com/org/repo/pull/42".to_string())
            }
        }
        .render(Some(80));
        let text = canvas.to_string();

        assert!(text.contains("PR #42"), "canvas=\n{text}");
        assert_eq!(
            canvas.hyperlink_at(3, 0).as_deref(),
            Some("https://github.com/org/repo/pull/42")
        );
    }
}
