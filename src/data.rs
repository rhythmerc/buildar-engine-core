//! The game files an engine runs on, which the player brings from their own copy of the game ([`GameFiles`]): where
//! the runtime looks for them, where it puts them, how it knows them, and what to tell the player. An engine states
//! this as plain data, readable before it is made: the runtime's installer shows it and checks for it.
use std::path::{Path, PathBuf};

/// An engine's game files, as the runtime's installer finds and keeps them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameFiles {
    /// The folder the player puts them in, under the device's downloads (`Download/<drop>`).
    pub drop: &'static str,
    /// Where they are kept, under the app's own folder.
    pub folder: &'static str,
    /// A path that only they have, from the folder they are kept in (`/`-separated, any case): how they are known,
    /// both where the player put them and once kept.
    pub marker: &'static str,
    /// Folders found beside them, brought too: each the name it has there and where it goes under the app's folder.
    pub beside: &'static [(&'static str, &'static str)],
    /// What to bring and how to make it, a line each.
    pub how: &'static [&'static str],
}

impl GameFiles {
    /// Whether `root` holds them (its marker there, in any case).
    pub fn in_folder(&self, root: &Path) -> bool {
        find(root, self.marker).is_some()
    }

    /// Where they are kept under the app's folder `files`.
    pub fn kept_in(&self, files: &Path) -> PathBuf {
        files.join(self.folder)
    }

    /// Where an archive's copy of them starts: the shortest prefix (ending in `/`, or empty) of an entry name that
    /// the marker follows, in any case. `\` counts as `/`.
    pub fn prefix_in<'a>(&self, names: impl Iterator<Item = &'a str>) -> Option<String> {
        let marker: Vec<&str> = parts(self.marker).collect();
        names
            .filter_map(|name| {
                let name = name.replace('\\', "/");
                let parts: Vec<&str> = parts(&name).collect();
                (0..parts.len()).find(|&at| {
                    parts.len() >= at + marker.len()
                        && parts[at..at + marker.len()].iter().zip(&marker).all(|(a, b)| a.eq_ignore_ascii_case(b))
                })
                .map(|at| parts[..at].iter().map(|p| format!("{p}/")).collect::<String>())
            })
            .min_by_key(String::len)
    }
}

fn parts(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|p| !p.is_empty())
}

/// `relative` under `root`, each part matched in any case.
fn find(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut at = root.to_path_buf();
    for part in parts(relative) {
        let exact = at.join(part);
        at = if exact.exists() {
            exact
        } else {
            std::fs::read_dir(&at).ok()?.flatten().map(|e| e.path()).find(|p| {
                p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.eq_ignore_ascii_case(part))
            })?
        };
    }
    Some(at)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILES: GameFiles = GameFiles { drop: "Game", folder: "game", marker: "Data/main.cfg", beside: &[], how: &[] };

    #[test]
    fn known_by_their_marker_in_any_case() {
        let root = std::env::temp_dir().join(format!("engine-core-data-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("DATA")).unwrap();
        assert!(!FILES.in_folder(&root));
        std::fs::write(root.join("DATA/Main.CFG"), b"").unwrap();
        assert!(FILES.in_folder(&root));
        assert_eq!(FILES.kept_in(Path::new("/files")), Path::new("/files/game"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn found_in_an_archive() {
        let names = ["copy/", "copy/game/data/main.cfg", "copy/game/data/other", "copy/extra/x"];
        assert_eq!(FILES.prefix_in(names.into_iter()), Some("copy/game/".into()));
        assert_eq!(FILES.prefix_in(["Data\\main.cfg"].into_iter()), Some(String::new()));
        assert_eq!(FILES.prefix_in(["main.cfg", "Data/other"].into_iter()), None);
        // (a folder for a marker: any entry under it)
        let folder = GameFiles { marker: "Data", ..FILES };
        assert_eq!(folder.prefix_in(["g/x.txt", "g/data/a/b"].into_iter()), Some("g/".into()));
    }
}
