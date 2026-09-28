//! Command editor model (SPEC §5.1): built-in packs + the user's pack, merged by id.
//! A user command with a built-in id overrides it; that is how built-ins get edited or
//! switched off without touching `packs/`.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use super::{load_dir, validate, Command, Pack};

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Entry {
    pub command: Command,
    /// Comes from a built-in pack (can't be deleted, only edited/switched off).
    pub builtin: bool,
    /// Built-in overridden by the user pack.
    pub modified: bool,
    /// Why the brain would skip it (empty = valid).
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, TS)]
#[ts(export)]
pub struct Library {
    pub entries: Vec<Entry>,
    pub folders: Vec<Vec<String>>,
}

impl Library {
    pub fn merge(builtin: Vec<Pack>, user: Option<Pack>) -> Self {
        let (mut user_cmds, folders) = user.map(|p| (p.commands, p.folders)).unwrap_or_default();
        let by_id: HashMap<String, usize> = user_cmds
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.clone(), i))
            .collect();
        let mut taken = vec![false; user_cmds.len()];
        let mut entries = Vec::new();
        for c in builtin.into_iter().flat_map(|p| p.commands) {
            let over = by_id.get(&c.id).copied();
            let command = match over {
                Some(i) => {
                    taken[i] = true;
                    user_cmds[i].clone()
                }
                None => c,
            };
            entries.push(entry(command, true, over.is_some()));
        }
        for (i, c) in user_cmds.drain(..).enumerate() {
            if !taken[i] {
                entries.push(entry(c, false, false));
            }
        }
        Self { entries, folders }
    }

    /// What the brain matches: valid, switched-on commands.
    pub fn active(self) -> Vec<Command> {
        self.entries
            .into_iter()
            .filter(|e| e.command.enabled && e.issues.is_empty())
            .map(|e| e.command)
            .collect()
    }
}

fn entry(command: Command, builtin: bool, modified: bool) -> Entry {
    Entry {
        issues: validate(&command),
        command,
        builtin,
        modified,
    }
}

/// Editor save: the user pack keeps own commands and built-ins that differ from the pack.
pub fn user_pack(builtin: &[Command], edited: Vec<Command>, folders: Vec<Vec<String>>) -> Pack {
    let orig: HashMap<&str, &Command> = builtin.iter().map(|c| (c.id.as_str(), c)).collect();
    Pack {
        id: "user".into(),
        name: "Мои команды".into(),
        commands: edited
            .into_iter()
            .filter(|c| orig.get(c.id.as_str()).is_none_or(|o| *o != c))
            .collect(),
        folders,
        ..Pack::default()
    }
}

/// The user's pack as saved, drafts included (the editor must not lose half-made commands).
pub fn read_user(path: &Path) -> Option<Pack> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text)
        .map_err(|e| tracing::warn!("user commands: {e}"))
        .ok()
}

/// Atomic write of the user pack (tmp + rename, a crash never leaves half a file).
pub fn write_user(path: &Path, pack: &Pack) -> crate::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(pack)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Add-on catalog folder (§9): shipped with the app, installed on demand.
const CATALOG: &str = "addons";

/// One catalog card for the «Дополнения» screen.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Addon {
    pub pack: Pack,
    pub installed: bool,
    /// Ships switched on (`packs/*.json`, «Команды по умолчанию»), can't be removed.
    pub default: bool,
}

/// Installed add-on ids (`addons.json`); missing or broken file = nothing installed.
pub fn read_installed(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn write_installed(path: &Path, ids: &[String]) -> crate::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(ids)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// First run (no `addons.json`): every catalog add-on starts installed. They are scoped by
/// `when` and tested clash-free together, and a fresh Jarvis that doesn't know
/// «открой проводник» feels broken.
pub fn install_defaults(path: &Path, dir: &Path) -> crate::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let ids: Vec<String> = load_dir(&dir.join(CATALOG))
        .0
        .into_iter()
        .map(|p| p.id)
        .collect();
    write_installed(path, &ids)
}

/// What the brain and the editor see: default packs + installed add-ons.
pub fn builtin_packs(dir: &Path, installed: &[String]) -> Vec<Pack> {
    let mut packs = load_dir(dir).0;
    packs.extend(
        load_dir(&dir.join(CATALOG))
            .0
            .into_iter()
            .filter(|p| installed.contains(&p.id)),
    );
    packs
}

/// The whole catalog: defaults first, then add-ons.
pub fn addons(dir: &Path, installed: &[String]) -> Vec<Addon> {
    let defaults = load_dir(dir).0.into_iter().map(|pack| Addon {
        pack,
        installed: true,
        default: true,
    });
    let catalog = load_dir(&dir.join(CATALOG))
        .0
        .into_iter()
        .map(|pack| Addon {
            installed: installed.contains(&pack.id),
            pack,
            default: false,
        });
    defaults.chain(catalog).collect()
}

/// Uninstall: the user's edits of the pack's commands go too, otherwise they would
/// resurface as the user's own commands.
pub fn drop_overrides(user: &mut Pack, pack: &Pack) -> bool {
    let before = user.commands.len();
    user.commands
        .retain(|c| !pack.commands.iter().any(|p| p.id == c.id));
    user.commands.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(id: &str, phrase: &str) -> Command {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": id, "phrases": [phrase], "reply": {"clips": ["ok"]}
        }))
        .expect("cmd")
    }

    fn pack(cmds: Vec<Command>) -> Pack {
        Pack {
            id: "p".into(),
            name: "P".into(),
            commands: cmds,
            ..Pack::default()
        }
    }

    #[test]
    fn user_overrides_builtin_in_place_and_appends_own() {
        let mut over = cmd("b", "бэ");
        over.enabled = false;
        let draft = cmd("draft", "");
        let lib = Library::merge(
            vec![pack(vec![cmd("a", "а"), cmd("b", "б")])],
            Some(pack(vec![cmd("mine", "моя"), over, draft])),
        );
        let ids: Vec<_> = lib.entries.iter().map(|e| e.command.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "mine", "draft"]);
        assert!(lib.entries[1].builtin && lib.entries[1].modified);
        assert!(!lib.entries[2].builtin);
        assert!(!lib.entries[3].issues.is_empty());
        let active: Vec<_> = lib.active().into_iter().map(|c| c.id).collect();
        assert_eq!(active, ["a", "mine"]);
    }

    #[test]
    fn user_pack_keeps_only_changes() {
        let builtin = vec![cmd("a", "а"), cmd("b", "б")];
        let mut b2 = cmd("b", "б");
        b2.folder = vec!["Моё".into()];
        let p = user_pack(
            &builtin,
            vec![cmd("a", "а"), b2, cmd("new", "н")],
            vec![vec!["Пустая".into()]],
        );
        let ids: Vec<_> = p.commands.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["b", "new"]);
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("commands.json");
        write_user(&path, &p).expect("write");
        let back = read_user(&path).expect("read");
        assert_eq!(back, p);
    }

    #[test]
    fn catalog_install_and_uninstall() {
        let dir = tempfile::tempdir().expect("tmp");
        let write = |path: std::path::PathBuf, id: &str, cmd_id: &str| {
            let mut p = pack(vec![cmd(cmd_id, cmd_id)]);
            p.id = id.into();
            std::fs::write(path, serde_json::to_string(&p).expect("json")).expect("w");
            p
        };
        std::fs::create_dir(dir.path().join(CATALOG)).expect("mkdir");
        write(dir.path().join("basic.json"), "basic", "b.one");
        let spotify = write(
            dir.path().join(CATALOG).join("spotify.json"),
            "spotify",
            "s.play",
        );
        write(
            dir.path().join(CATALOG).join("steam.json"),
            "steam",
            "st.open",
        );

        let ids = |packs: Vec<Pack>| packs.into_iter().map(|p| p.id).collect::<Vec<_>>();
        assert_eq!(ids(builtin_packs(dir.path(), &[])), ["basic"]);
        let on = vec!["spotify".to_string()];
        assert_eq!(ids(builtin_packs(dir.path(), &on)), ["basic", "spotify"]);
        let list = addons(dir.path(), &on);
        let flags: Vec<_> = list
            .iter()
            .map(|a| (a.pack.id.as_str(), a.installed, a.default))
            .collect();
        assert_eq!(
            flags,
            [
                ("basic", true, true),
                ("spotify", true, false),
                ("steam", false, false)
            ]
        );

        let file = dir.path().join("addons.json");
        assert!(read_installed(&file).is_empty());
        install_defaults(&file, dir.path()).expect("defaults");
        assert_eq!(read_installed(&file), ["spotify", "steam"]);
        std::fs::remove_file(&file).expect("rm");
        write_installed(&file, &on).expect("write");
        assert_eq!(read_installed(&file), on);

        let mut off = cmd("s.play", "пауза");
        off.enabled = false;
        let mut user = pack(vec![off, cmd("mine", "моя")]);
        assert!(drop_overrides(&mut user, &spotify));
        assert_eq!(user.commands.len(), 1);
        assert!(!drop_overrides(&mut user, &spotify));
    }
}
