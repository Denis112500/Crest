//! Windows: reads the `Run` entry itself and checks whether its program still exists. The
//! plugin can't tell "missing" from "disabled in Task Manager", the repair must.

use std::path::Path;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Registry::{RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

use crate::launch_at_login::launch_at_login_repair::RunEntryState;

/// Where Windows keeps the programs it starts at login (the key `tauri-plugin-autostart` writes).
const WINDOWS_RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const EXECUTABLE_EXTENSION: &str = ".exe";

/// Reads the `Run` entry itself, whatever Task Manager's "Startup apps" says about it: the
/// plugin's own check also counts an entry disabled in Task Manager as "off", which the repair
/// must not undo. It also tells whether the program the entry starts still exists.
pub fn read_run_key_entry_state(run_entry_name: &str) -> RunEntryState {
    let Some(run_command) = [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .find_map(|registry_root| read_run_command(registry_root, run_entry_name))
    else {
        return RunEntryState::Missing;
    };
    match extract_executable_path(&run_command) {
        Some(executable_path) if Path::new(&executable_path).is_file() => RunEntryState::StartsExistingProgram,
        _ => RunEntryState::StartsMissingProgram,
    }
}

fn read_run_command(registry_root: HKEY, run_entry_name: &str) -> Option<String> {
    let run_key_path = HSTRING::from(WINDOWS_RUN_KEY_PATH);
    let run_value_name = HSTRING::from(run_entry_name);
    let mut run_command_size_in_bytes = 0_u32;
    // SAFETY: the strings live until each call returns. The first call only reports the size;
    // the second writes at most that many bytes into a buffer at least that large.
    unsafe {
        RegGetValueW(
            registry_root,
            PCWSTR(run_key_path.as_ptr()),
            PCWSTR(run_value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut run_command_size_in_bytes),
        )
        .ok()
        .ok()?;
        let mut run_command_buffer = vec![0_u16; (run_command_size_in_bytes as usize).div_ceil(2)];
        RegGetValueW(
            registry_root,
            PCWSTR(run_key_path.as_ptr()),
            PCWSTR(run_value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(run_command_buffer.as_mut_ptr().cast()),
            Some(&mut run_command_size_in_bytes),
        )
        .ok()
        .ok()?;
        let run_command_length =
            run_command_buffer.iter().position(|&character| character == 0).unwrap_or(run_command_buffer.len());
        Some(String::from_utf16_lossy(&run_command_buffer[..run_command_length]))
    }
}

/// The program part of a `Run` command: `"C:\…\steam.exe" -silent` (quoted, with arguments) or,
/// as `tauri-plugin-autostart` writes it, `C:\…\crest.exe ` (unquoted, with a trailing space for
/// its empty argument list). Everything up to and including ".exe".
fn extract_executable_path(run_command: &str) -> Option<String> {
    let unquoted_run_command = run_command.trim().trim_start_matches('"');
    let executable_extension_start = unquoted_run_command.to_lowercase().find(EXECUTABLE_EXTENSION)?;
    Some(unquoted_run_command[..executable_extension_start + EXECUTABLE_EXTENSION.len()].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_program_in_quoted_and_unquoted_run_commands() {
        assert_eq!(
            extract_executable_path(r"C:\Users\denis\AppData\Local\Crest\crest.exe ").as_deref(),
            Some(r"C:\Users\denis\AppData\Local\Crest\crest.exe")
        );
        assert_eq!(
            extract_executable_path(r#""C:\Program Files (x86)\Steam\steam.exe" -silent"#).as_deref(),
            Some(r"C:\Program Files (x86)\Steam\steam.exe")
        );
        assert_eq!(extract_executable_path("not a program"), None);
    }
}
