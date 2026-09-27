//! «Дополнения» IPC (SPEC §9): the local pack catalog, one-click install/uninstall.

use jarvis_core::commands::{self, Addon, Library};

use crate::editor::{builtin, reload};
use crate::AppState;

#[tauri::command]
pub fn addons_list(state: tauri::State<'_, AppState>) -> Vec<Addon> {
    let installed = commands::read_installed(&state.paths.addons());
    state
        .packs
        .get()
        .and_then(|p| p.as_deref())
        .map(|d| commands::addons(d, &installed))
        .unwrap_or_default()
}

/// Install or remove an add-on; the brain picks it up at once. Returns active commands.
#[tauri::command]
pub fn addon_set(state: tauri::State<'_, AppState>, id: String, on: bool) -> Result<usize, String> {
    let file = state.paths.addons();
    let mut ids = commands::read_installed(&file);
    if !on {
        // the user's edits of its commands leave with it
        let user_file = state.paths.user_commands();
        let pack = builtin(&state).into_iter().find(|p| p.id == id);
        if let (Some(pack), Some(mut user)) = (pack, commands::read_user(&user_file)) {
            if commands::drop_overrides(&mut user, &pack) {
                commands::write_user(&user_file, &user).map_err(|e| e.to_string())?;
            }
        }
    }
    ids.retain(|i| *i != id);
    if on {
        ids.push(id);
    }
    commands::write_installed(&file, &ids).map_err(|e| e.to_string())?;
    let lib = Library::merge(
        builtin(&state),
        commands::read_user(&state.paths.user_commands()),
    );
    Ok(reload(&state, lib))
}
