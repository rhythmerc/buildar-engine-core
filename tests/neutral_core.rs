//! The core stays neutral: no one mode's words in it. Each mode's words belong in its own module.
use std::path::Path;

/// Words that belong to the skate mode, not to every kind of game.
const SKATE_WORDS: &[&str] = &["skater", "skaters", "board", "boards", "deck", "grind", "grinds", "grinding", "rail", "rails", "trick", "tricks", "park", "parks"];

/// The core's sources: everything under `src/` but the modes' modules.
const MODES: &[&str] = &["skate.rs", "skate"];

/// Whole words only, so "keyboard" or "onboard" don't count.
fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_ascii_lowercase)
}

#[test]
fn the_core_has_no_skate_words() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if MODES.contains(&name.as_str()) || !name.ends_with(".rs") {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap();
        for (line, l) in text.lines().enumerate() {
            for word in words(l) {
                if SKATE_WORDS.contains(&word.as_str()) {
                    found.push(format!("src/{name}:{}: {word}", line + 1));
                }
            }
        }
    }
    assert!(found.is_empty(), "skate words in the core (move them to src/skate.rs):\n{}", found.join("\n"));
}

#[test]
fn whole_words_only() {
    assert_eq!(words("A keyboard, on-board").collect::<Vec<_>>(), ["a", "keyboard", "on", "board"]);
}
