//! Maps to: CC `components/PromptInput/PromptInputStashNotice.tsx:1-19`.

use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PromptInputStashNoticeProps {
    pub has_stash: bool,
}

#[component]
pub fn PromptInputStashNotice(
    props: &PromptInputStashNoticeProps,
) -> impl Into<AnyElement<'static>> {
    if !props.has_stash {
        return element! { Fragment }.into_any();
    }
    element! {
        View(padding_left: 2u32) {
            Text(
                content: format!(
                    "{} Stashed (auto-restores after submit)",
                    crate::constants::figures::get().pointer_small
                ),
                dim: true,
                wrap: TextWrap::NoWrap,
            )
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stash_notice_is_null_or_exact_dim_copy() {
        assert!(
            element! { PromptInputStashNotice(has_stash: false) }
                .render(Some(80))
                .to_string()
                .trim()
                .is_empty()
        );
        let canvas = element! { PromptInputStashNotice(has_stash: true) }.render(Some(80));
        assert!(
            canvas
                .to_string()
                .contains("  › Stashed (auto-restores after submit)")
        );
        assert!(canvas.resolved_text_style(2, 0).unwrap().dim);
    }
}
