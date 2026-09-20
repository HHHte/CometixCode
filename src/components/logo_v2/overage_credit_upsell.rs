//! Maps to: CC `components/LogoV2/OverageCreditUpsell.tsx`.
//!
//! Backend eligibility and cache refresh remain outside this component. The
//! Rust port consumes an already-formatted grant amount and preserves the
//! official display strings, show gate, and homescreen feed config shape.

use super::feed::{FeedConfig, FeedCustomContent, FeedCustomLineTone, FeedLine};
use crate::utils::truncate::truncate;
use iocraft::prelude::*;

pub const OVERAGE_CREDIT_MAX_IMPRESSIONS: u32 = 3;
const FEED_SUBTITLE: &str = "On us. Works on third-party apps · /extra-usage";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverageCreditGrantInfo {
    pub available: bool,
    pub granted: bool,
    /// Already-formatted amount, matching official `formatGrantAmount(info)`.
    pub amount: Option<String>,
}

#[derive(Default, Props)]
pub struct OverageCreditUpsellProps {
    pub amount: Option<String>,
    pub max_width: Option<usize>,
    pub two_line: bool,
}

pub fn is_eligible_for_overage_credit_grant(info: Option<&OverageCreditGrantInfo>) -> bool {
    let Some(info) = info else {
        return false;
    };
    info.available
        && !info.granted
        && info
            .amount
            .as_ref()
            .is_some_and(|amount| !amount.is_empty())
}

pub fn should_show_overage_credit_upsell(
    info: Option<&OverageCreditGrantInfo>,
    has_visited_extra_usage: bool,
    overage_credit_upsell_seen_count: u32,
) -> bool {
    is_eligible_for_overage_credit_grant(info)
        && !has_visited_extra_usage
        && overage_credit_upsell_seen_count < OVERAGE_CREDIT_MAX_IMPRESSIONS
}

pub fn get_usage_text(amount: &str) -> String {
    format!("{amount} in extra usage for third-party apps · /extra-usage")
}

pub fn get_feed_title(amount: &str) -> String {
    format!("{amount} in extra usage")
}

pub fn create_overage_credit_feed(amount: Option<&str>) -> FeedConfig {
    let title = amount
        .map(get_feed_title)
        .unwrap_or_else(|| "extra usage credit".to_string());
    FeedConfig {
        title: title.clone(),
        lines: Vec::new(),
        footer: None,
        empty_message: None,
        custom_content: Some(FeedCustomContent {
            lines: vec![FeedLine::text(FEED_SUBTITLE)],
            width: title.chars().count().max(FEED_SUBTITLE.chars().count()),
            line_tones: vec![FeedCustomLineTone::Dim],
            margin_y_first_line: false,
        }),
    }
}

#[component]
pub fn OverageCreditUpsell(
    props: &OverageCreditUpsellProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let amount = props.amount.clone().filter(|amount| !amount.is_empty());

    element! {
        Fragment {
            #(amount.map(|amount| {
                if props.two_line {
                    let title = get_feed_title(&amount);
                    let title = props.max_width.map(|width| truncate(&title, width, false)).unwrap_or(title);
                    let subtitle = props.max_width.map(|width| truncate(FEED_SUBTITLE, width, false)).unwrap_or_else(|| FEED_SUBTITLE.to_string());
                    element! {
                        View(flex_direction: FlexDirection::Column) {
                            Text(content: title, color: theme.claude, wrap: TextWrap::NoWrap)
                            Text(content: subtitle, color: theme.inactive, wrap: TextWrap::NoWrap)
                        }
                    }.into_any()
                } else {
                    let text = get_usage_text(&amount);
                    let display = props.max_width.map(|width| truncate(&text, width, false)).unwrap_or(text);
                    let highlight_len = get_feed_title(&amount).chars().count().min(display.chars().count());
                    let highlight = display.chars().take(highlight_len).collect::<String>();
                    let rest = display.chars().skip(highlight_len).collect::<String>();
                    element! {
                        View(flex_direction: FlexDirection::Row) {
                            Text(content: highlight, color: theme.claude, wrap: TextWrap::NoWrap)
                            Text(content: rest, color: theme.inactive, wrap: TextWrap::NoWrap)
                        }
                    }.into_any()
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render_overage(amount: Option<&str>, max_width: Option<usize>, two_line: bool) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                OverageCreditUpsell(
                    amount: amount.map(str::to_string),
                    max_width: max_width,
                    two_line: two_line,
                )
            }
        }
        .render(Some(120))
        .to_string()
    }

    #[test]
    fn overage_credit_eligibility_and_show_gate_match_official_rules() {
        let info = OverageCreditGrantInfo {
            available: true,
            granted: false,
            amount: Some("$10".to_string()),
        };
        assert!(is_eligible_for_overage_credit_grant(Some(&info)));
        assert!(should_show_overage_credit_upsell(Some(&info), false, 0));
        assert!(!should_show_overage_credit_upsell(Some(&info), true, 0));
        assert!(!should_show_overage_credit_upsell(Some(&info), false, 3));
        assert!(!is_eligible_for_overage_credit_grant(Some(
            &OverageCreditGrantInfo {
                granted: true,
                ..info.clone()
            }
        )));
        assert!(!is_eligible_for_overage_credit_grant(None));
    }

    #[test]
    fn overage_credit_upsell_renders_official_one_line_copy() {
        let text = render_overage(Some("$10"), None, false);
        assert!(
            text.contains("$10 in extra usage for third-party apps · /extra-usage"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn overage_credit_upsell_renders_official_two_line_feed_copy() {
        let text = render_overage(Some("$10"), None, true);
        assert!(text.contains("$10 in extra usage"), "canvas=\n{text}");
        assert!(
            text.contains("On us. Works on third-party apps · /extra-usage"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn overage_credit_feed_matches_official_homescreen_feed_shape() {
        let feed = create_overage_credit_feed(Some("$10"));
        assert_eq!(feed.title, "$10 in extra usage");
        let custom = feed.custom_content.expect("custom feed content");
        assert_eq!(custom.lines[0].text, FEED_SUBTITLE);
        assert_eq!(custom.line_tones[0], FeedCustomLineTone::Dim);
        assert!(!custom.margin_y_first_line);
        assert_eq!(custom.width, FEED_SUBTITLE.chars().count());

        let fallback = create_overage_credit_feed(None);
        assert_eq!(fallback.title, "extra usage credit");
    }
}
