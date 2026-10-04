use tauri::{Runtime, WebviewWindow};

use crate::pill_window::pill_display_choice::ConnectedDisplay;

/// The monitors Windows reports right now, as the display choice needs them. Read fresh on
/// every call, so a monitor plugged in since startup shows up in the settings list.
pub fn read_connected_displays<R: Runtime>(any_window: &WebviewWindow<R>) -> Result<Vec<ConnectedDisplay>, String> {
    let main_display_name = any_window
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .and_then(|main_monitor| main_monitor.name().cloned());
    let connected_monitors = any_window.available_monitors().map_err(|error| error.to_string())?;
    Ok(connected_monitors
        .into_iter()
        .filter_map(|connected_monitor| {
            // Windows names every monitor; one without a name couldn't be chosen or remembered.
            let display_name = connected_monitor.name()?.clone();
            Some(ConnectedDisplay {
                is_main_display: main_display_name.as_ref() == Some(&display_name),
                display_name,
                physical_left: connected_monitor.position().x,
                physical_top: connected_monitor.position().y,
                physical_width: connected_monitor.size().width,
                physical_height: connected_monitor.size().height,
                scale_factor: connected_monitor.scale_factor(),
            })
        })
        .collect())
}
