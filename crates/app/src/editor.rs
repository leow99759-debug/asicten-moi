//! Command editor IPC (SPEC §5.1, §5.3): read the merged library, save the user pack,
//! live match preview, «▶ Тест» and reply preview for an unsaved command.

use jarvis_core::brain::{self, Line, Probe};
use jarvis_core::commands::{self, Command, Library, Pack, Reply};

use crate::brain_worker::Work;
use crate::AppState;

fn builtin(state: &AppState) -> Vec<Pack> {
    state
        .packs
        .get()
        .and_then(|p| p.as_deref())
        .map(|d| commands::load_dir(d).0)
        .unwrap_or_default()
}

#[tauri::command]
pub fn editor_library(state: tauri::State<'_, AppState>) -> Library {
    Library::merge(
        builtin(&state),
        commands::read_user(&state.paths.user_commands()),
    )
}

/// Write commands.json (own commands + changed built-ins) and hot-swap the brain's set.
#[tauri::command]
pub fn editor_save(
    state: tauri::State<'_, AppState>,
    commands: Vec<Command>,
    folders: Vec<Vec<String>>,
) -> Result<Library, String> {
    let packs = builtin(&state);
    let flat: Vec<Command> = packs.iter().flat_map(|p| p.commands.clone()).collect();
    let pack = commands::user_pack(&flat, commands, folders);
    let path = state.paths.user_commands();
    commands::write_user(&path, &pack).map_err(|e| e.to_string())?;
    let lib = Library::merge(packs, Some(pack));
    let active = lib.clone().active();
    state
        .commands
        .store(active.len(), std::sync::atomic::Ordering::Relaxed);
    let _ = state.work.send(Work::Reload(active));
    Ok(lib)
}

/// Which command each sample phrase reaches with the editor's current (unsaved) set.
#[tauri::command]
pub fn editor_probe(commands: Vec<Command>, target: Command, texts: Vec<String>) -> Vec<Probe> {
    brain::probe(&commands, &target, &texts)
}

/// «▶ Тест»: run the edited command without voice; the result arrives as an `outcome` event.
#[tauri::command]
pub fn editor_test(state: tauri::State<'_, AppState>, command: Command, sample: String) {
    let _ = state.work.send(Work::Test(Box::new(command), sample));
}

/// «▶» next to the reply: say it with the current voice settings.
#[tauri::command]
pub fn say_reply(state: tauri::State<'_, AppState>, reply: Reply) {
    if let Some(s) = state.speaker.get().cloned() {
        std::thread::spawn(move || {
            s.stop();
            s.say(&Line {
                clips: reply.clips,
                text: reply.text,
            });
        });
    }
}
