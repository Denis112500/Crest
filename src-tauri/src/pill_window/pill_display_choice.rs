//! Which monitor the pill uses (the chosen one, else the main one) and the labels for the
//! settings list. Plain logic, unit-tested without real monitors.

use serde::Serialize;

/// What the display choice needs to know about a monitor; filled from Tauri's `Monitor`,
/// kept separate so the rules below can be unit-tested without real monitors.
pub struct ConnectedDisplay {
    pub display_name: String,
    pub physical_left: i32,
    pub physical_top: i32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
    pub is_main_display: bool,
}

/// One entry of the settings window's display list. `display_name: None` is "Main
/// display", which follows whatever Windows calls the main display.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillDisplayOption {
    pub display_name: Option<String>,
    pub label: String,
    pub is_chosen: bool,
}

const MAIN_DISPLAY_OPTION_LABEL: &str = "Main display";
const WINDOWS_DISPLAY_NAME_PREFIX: &str = r"\\.\DISPLAY";

/// The monitor the pill goes on: the chosen one while it's connected, otherwise the main one.
pub fn find_display_for_pill<'displays>(
    connected_displays: &'displays [ConnectedDisplay],
    chosen_display_name: Option<&str>,
) -> Option<&'displays ConnectedDisplay> {
    chosen_display_name
        .and_then(|chosen_name| connected_displays.iter().find(|display| display.display_name == chosen_name))
        .or_else(|| connected_displays.iter().find(|display| display.is_main_display))
}

/// "Main display" first, then every connected monitor. A chosen monitor that's unplugged
/// isn't listed, and "Main display" is marked instead, which is where the pill is then.
pub fn describe_pill_display_options(
    connected_displays: &[ConnectedDisplay],
    chosen_display_name: Option<&str>,
) -> Vec<PillDisplayOption> {
    let is_chosen_display_connected = chosen_display_name
        .is_some_and(|chosen_name| connected_displays.iter().any(|display| display.display_name == chosen_name));
    let main_display_left = connected_displays
        .iter()
        .find(|display| display.is_main_display)
        .map_or(0, |main_display| main_display.physical_left);
    let main_display_option = PillDisplayOption {
        display_name: None,
        label: MAIN_DISPLAY_OPTION_LABEL.to_string(),
        is_chosen: !is_chosen_display_connected,
    };
    let monitor_options = connected_displays.iter().map(|display| PillDisplayOption {
        display_name: Some(display.display_name.clone()),
        label: describe_display(display, main_display_left),
        is_chosen: chosen_display_name == Some(display.display_name.as_str()),
    });
    std::iter::once(main_display_option).chain(monitor_options).collect()
}

/// "Display 2 · 1920 × 1080 · right of main": Windows' own number, the size and where it sits,
/// so the user can tell monitors apart without opening Windows' display settings.
fn describe_display(display: &ConnectedDisplay, main_display_left: i32) -> String {
    let display_title = match display.display_name.strip_prefix(WINDOWS_DISPLAY_NAME_PREFIX) {
        Some(display_number) => format!("Display {display_number}"),
        None => display.display_name.clone(),
    };
    let display_position = if display.is_main_display {
        "main"
    } else if display.physical_left < main_display_left {
        "left of main"
    } else {
        "right of main"
    };
    format!("{display_title} · {} × {} · {display_position}", display.physical_width, display.physical_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The two monitors of the development PC (measured): primary on the left, second on the right.
    fn development_pc_displays() -> Vec<ConnectedDisplay> {
        vec![
            ConnectedDisplay {
                display_name: r"\\.\DISPLAY1".to_string(),
                physical_left: 0,
                physical_top: 0,
                physical_width: 2560,
                physical_height: 1440,
                scale_factor: 1.0,
                is_main_display: true,
            },
            ConnectedDisplay {
                display_name: r"\\.\DISPLAY2".to_string(),
                physical_left: 2560,
                physical_top: 0,
                physical_width: 1920,
                physical_height: 1080,
                scale_factor: 1.0,
                is_main_display: false,
            },
        ]
    }

    #[test]
    fn uses_the_chosen_display_while_it_is_connected() {
        let connected_displays = development_pc_displays();
        let pill_display = find_display_for_pill(&connected_displays, Some(r"\\.\DISPLAY2")).unwrap();
        assert_eq!(pill_display.display_name, r"\\.\DISPLAY2");
    }

    #[test]
    fn falls_back_to_the_main_display_when_the_chosen_one_is_unplugged() {
        let connected_displays = development_pc_displays();
        let pill_display = find_display_for_pill(&connected_displays, Some(r"\\.\DISPLAY3")).unwrap();
        assert_eq!(pill_display.display_name, r"\\.\DISPLAY1");
        let display_options = describe_pill_display_options(&connected_displays, Some(r"\\.\DISPLAY3"));
        assert!(display_options[0].is_chosen);
        assert!(display_options.iter().skip(1).all(|display_option| !display_option.is_chosen));
    }

    #[test]
    fn labels_each_display_with_number_size_and_position() {
        let display_options = describe_pill_display_options(&development_pc_displays(), Some(r"\\.\DISPLAY2"));
        let display_labels: Vec<&str> = display_options.iter().map(|display_option| display_option.label.as_str()).collect();
        assert_eq!(
            display_labels,
            ["Main display", "Display 1 · 2560 × 1440 · main", "Display 2 · 1920 × 1080 · right of main"]
        );
        assert!(display_options[2].is_chosen);
    }
}
