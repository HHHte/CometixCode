//! Maps to: CC `commands/skills/skills.tsx:1-11`.

use crate::components::skills::skills_menu::{SkillMenuCommand, SkillsMenu, SkillsMenuDone};
use crate::skills::load_skills_dir::{get_dynamic_skills, get_skill_dir_commands};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct SkillsCommandProps<'a> {
    pub on_done: HandlerMut<'a, SkillsMenuDone>,
}

/// Thin command boundary: command loading stays outside `SkillsMenu`.
#[component]
pub fn SkillsCommand<'a>(
    props: &mut SkillsCommandProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let commands = hooks.use_state(|| {
        // CC hands `SkillsMenu` `context.options.commands`, i.e. the
        // `getCommands()` snapshot — which merges dynamic skills at
        // `commands.ts:479-516`. `get_skill_dir_commands` is the static half
        // only (that merge moved to its CC owner), so the dynamic half is
        // appended here under the same first-wins name dedup CC applies.
        let mut skills = get_skill_dir_commands(&crate::bootstrap::state::get_original_cwd());
        for skill in get_dynamic_skills() {
            if !skills.iter().any(|entry| entry.name == skill.name) {
                skills.push(skill);
            }
        }
        skills
            .iter()
            .map(SkillMenuCommand::from)
            .collect::<Vec<_>>()
    });
    let mut pending_done = hooks.use_state(|| Option::<SkillsMenuDone>::None);
    let done = { pending_done.read().clone() };
    if let Some(done) = done {
        pending_done.set(None);
        (props.on_done)(done);
    }
    let mut pending_done_for_menu = pending_done;
    element! {
        SkillsMenu(
            commands: commands.read().clone(),
            on_exit: move |done| pending_done_for_menu.set(Some(done)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_boundary_renders_menu() {
        let text = element! { SkillsCommand() }.render(Some(100)).to_string();
        assert!(text.contains("Skills"));
        assert!(text.contains("Esc to close"));
    }
}
