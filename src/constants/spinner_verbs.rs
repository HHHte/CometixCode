//! Maps to: CC `constants/spinnerVerbs.ts`.
//! Built-in spinner verbs plus settings-driven append/replace behavior.

use crate::utils::settings::SettingsJson;

// Spinner verbs for loading messages.
// Keep this list 1:1 with CC `SPINNER_VERBS`.
pub const SPINNER_VERBS: &[&str] = &[
    "Accomplishing",
    "Actioning",
    "Actualizing",
    "Architecting",
    "Baking",
    "Beaming",
    "Beboppin'",
    "Befuddling",
    "Billowing",
    "Blanching",
    "Bloviating",
    "Boogieing",
    "Boondoggling",
    "Booping",
    "Bootstrapping",
    "Brewing",
    "Bunning",
    "Burrowing",
    "Calculating",
    "Canoodling",
    "Caramelizing",
    "Cascading",
    "Catapulting",
    "Cerebrating",
    "Channeling",
    "Channelling",
    "Choreographing",
    "Churning",
    "Clauding",
    "Coalescing",
    "Cogitating",
    "Combobulating",
    "Composing",
    "Computing",
    "Concocting",
    "Considering",
    "Contemplating",
    "Cooking",
    "Crafting",
    "Creating",
    "Crunching",
    "Crystallizing",
    "Cultivating",
    "Deciphering",
    "Deliberating",
    "Determining",
    "Dilly-dallying",
    "Discombobulating",
    "Doing",
    "Doodling",
    "Drizzling",
    "Ebbing",
    "Effecting",
    "Elucidating",
    "Embellishing",
    "Enchanting",
    "Envisioning",
    "Evaporating",
    "Fermenting",
    "Fiddle-faddling",
    "Finagling",
    "Flambéing",
    "Flibbertigibbeting",
    "Flowing",
    "Flummoxing",
    "Fluttering",
    "Forging",
    "Forming",
    "Frolicking",
    "Frosting",
    "Gallivanting",
    "Galloping",
    "Garnishing",
    "Generating",
    "Gesticulating",
    "Germinating",
    "Gitifying",
    "Grooving",
    "Gusting",
    "Harmonizing",
    "Hashing",
    "Hatching",
    "Herding",
    "Honking",
    "Hullaballooing",
    "Hyperspacing",
    "Ideating",
    "Imagining",
    "Improvising",
    "Incubating",
    "Inferring",
    "Infusing",
    "Ionizing",
    "Jitterbugging",
    "Julienning",
    "Kneading",
    "Leavening",
    "Levitating",
    "Lollygagging",
    "Manifesting",
    "Marinating",
    "Meandering",
    "Metamorphosing",
    "Misting",
    "Moonwalking",
    "Moseying",
    "Mulling",
    "Mustering",
    "Musing",
    "Nebulizing",
    "Nesting",
    "Newspapering",
    "Noodling",
    "Nucleating",
    "Orbiting",
    "Orchestrating",
    "Osmosing",
    "Perambulating",
    "Percolating",
    "Perusing",
    "Philosophising",
    "Photosynthesizing",
    "Pollinating",
    "Pondering",
    "Pontificating",
    "Pouncing",
    "Precipitating",
    "Prestidigitating",
    "Processing",
    "Proofing",
    "Propagating",
    "Puttering",
    "Puzzling",
    "Quantumizing",
    "Razzle-dazzling",
    "Razzmatazzing",
    "Recombobulating",
    "Reticulating",
    "Roosting",
    "Ruminating",
    "Sautéing",
    "Scampering",
    "Schlepping",
    "Scurrying",
    "Seasoning",
    "Shenaniganing",
    "Shimmying",
    "Simmering",
    "Skedaddling",
    "Sketching",
    "Slithering",
    "Smooshing",
    "Sock-hopping",
    "Spelunking",
    "Spinning",
    "Sprouting",
    "Stewing",
    "Sublimating",
    "Swirling",
    "Swooping",
    "Symbioting",
    "Synthesizing",
    "Tempering",
    "Thinking",
    "Thundering",
    "Tinkering",
    "Tomfoolering",
    "Topsy-turvying",
    "Transfiguring",
    "Transmuting",
    "Twisting",
    "Undulating",
    "Unfurling",
    "Unravelling",
    "Vibing",
    "Waddling",
    "Wandering",
    "Warping",
    "Whatchamacalliting",
    "Whirlpooling",
    "Whirring",
    "Whisking",
    "Wibbling",
    "Working",
    "Wrangling",
    "Zesting",
    "Zigzagging",
];

/// Maps to CC `getSpinnerVerbs()`.
pub fn get_spinner_verbs(settings: Option<&SettingsJson>) -> Vec<String> {
    let default_verbs = || {
        SPINNER_VERBS
            .iter()
            .map(|verb| (*verb).to_string())
            .collect()
    };

    let Some(config) = settings.and_then(|settings| settings.spinner_verbs.as_ref()) else {
        return default_verbs();
    };

    let configured = config
        .verbs
        .iter()
        .filter(|verb| !verb.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    if config.mode == "replace" {
        if configured.is_empty() {
            default_verbs()
        } else {
            configured
        }
    } else {
        default_verbs().into_iter().chain(configured).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_spinner_verbs_matches_official_append_replace_modes() {
        let mut replace_settings = SettingsJson::default();
        replace_settings.spinner_verbs =
            Some(crate::utils::settings::types::SpinnerVerbsSettings {
                mode: "replace".to_string(),
                verbs: vec!["Reviewing".to_string(), "".to_string()],
            });
        assert_eq!(
            get_spinner_verbs(Some(&replace_settings)),
            vec!["Reviewing"]
        );

        let mut empty_replace_settings = SettingsJson::default();
        empty_replace_settings.spinner_verbs =
            Some(crate::utils::settings::types::SpinnerVerbsSettings {
                mode: "replace".to_string(),
                verbs: Vec::new(),
            });
        assert_eq!(
            get_spinner_verbs(Some(&empty_replace_settings))[0],
            "Accomplishing"
        );

        let mut append_settings = SettingsJson::default();
        append_settings.spinner_verbs = Some(crate::utils::settings::types::SpinnerVerbsSettings {
            mode: "append".to_string(),
            verbs: vec!["Reviewing".to_string()],
        });
        let candidates = get_spinner_verbs(Some(&append_settings));
        assert_eq!(candidates[0], "Accomplishing");
        assert_eq!(candidates.last().map(String::as_str), Some("Reviewing"));
    }
}
