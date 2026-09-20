//! Maps to: CC `components/LogoV2/Clawd.tsx`.

use crate::utils::env;
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClawdPose {
    #[default]
    Default,
    ArmsUp,
    LookLeft,
    LookRight,
}

#[derive(Clone, Copy)]
struct ClawdSegments {
    r1_l: &'static str,
    r1_e: &'static str,
    r1_r: &'static str,
    r2_l: &'static str,
    r2_r: &'static str,
}

impl ClawdPose {
    fn segments(self) -> ClawdSegments {
        match self {
            Self::Default => ClawdSegments {
                r1_l: " ▐",
                r1_e: "▛███▜",
                r1_r: "▌",
                r2_l: "▝▜",
                r2_r: "▛▘",
            },
            Self::LookLeft => ClawdSegments {
                r1_l: " ▐",
                r1_e: "▟███▟",
                r1_r: "▌",
                r2_l: "▝▜",
                r2_r: "▛▘",
            },
            Self::LookRight => ClawdSegments {
                r1_l: " ▐",
                r1_e: "▙███▙",
                r1_r: "▌",
                r2_l: "▝▜",
                r2_r: "▛▘",
            },
            Self::ArmsUp => ClawdSegments {
                r1_l: "▗▟",
                r1_e: "▛███▜",
                r1_r: "▙▖",
                r2_l: " ▜",
                r2_r: "▛ ",
            },
        }
    }

    fn apple_eyes(self) -> &'static str {
        match self {
            Self::Default | Self::ArmsUp => " ▗   ▖ ",
            Self::LookLeft => " ▘   ▘ ",
            Self::LookRight => " ▝   ▝ ",
        }
    }
}

#[derive(Default, Props)]
pub struct ClawdProps {
    pub pose: ClawdPose,
}

#[component]
pub fn Clawd(props: &ClawdProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();

    if env::get().terminal.as_deref() == Some("Apple_Terminal") {
        let eyes = props.pose.apple_eyes().to_string();
        return element! {
            View(flex_direction: FlexDirection::Column, align_items: AlignItems::CENTER) {
                View(flex_direction: FlexDirection::Row) {
                    Text(content: "▗".to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
                    Text(content: eyes, color: theme.clawd_bg, background_color: theme.clawd_body, wrap: TextWrap::NoWrap)
                    Text(content: "▖".to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
                }
                Text(content: "       ".to_string(), background_color: theme.clawd_body, wrap: TextWrap::NoWrap)
                Text(content: "▘▘ ▝▝".to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
            }
        };
    }

    let segments = props.pose.segments();
    element! {
        View(flex_direction: FlexDirection::Column) {
            View(flex_direction: FlexDirection::Row) {
                Text(content: segments.r1_l.to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
                Text(content: segments.r1_e.to_string(), color: theme.clawd_body, background_color: theme.clawd_bg, wrap: TextWrap::NoWrap)
                Text(content: segments.r1_r.to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
            }
            View(flex_direction: FlexDirection::Row) {
                Text(content: segments.r2_l.to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
                Text(content: "█████".to_string(), color: theme.clawd_body, background_color: theme.clawd_bg, wrap: TextWrap::NoWrap)
                Text(content: segments.r2_r.to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
            }
            Text(content: "  ▘▘ ▝▝  ".to_string(), color: theme.clawd_body, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render_pose(pose: ClawdPose) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                Clawd(pose: pose)
            }
        }
        .render(Some(20))
        .to_string()
    }

    #[test]
    fn clawd_default_pose_matches_official_segments() {
        let text = render_pose(ClawdPose::Default);
        assert!(text.contains("▛███▜"), "canvas=\n{text}");
        assert!(text.contains("▝▜█████▛▘"), "canvas=\n{text}");
        assert!(text.contains("▘▘ ▝▝"), "canvas=\n{text}");
    }

    #[test]
    fn clawd_pose_segments_match_official_pose_table() {
        assert_eq!(ClawdPose::LookLeft.segments().r1_e, "▟███▟");
        assert_eq!(ClawdPose::LookRight.segments().r1_e, "▙███▙");
        assert_eq!(ClawdPose::ArmsUp.segments().r1_l, "▗▟");
        assert_eq!(ClawdPose::ArmsUp.segments().r1_r, "▙▖");
        assert_eq!(ClawdPose::ArmsUp.apple_eyes(), " ▗   ▖ ");
    }
}
