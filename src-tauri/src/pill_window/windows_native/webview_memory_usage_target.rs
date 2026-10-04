use tauri::webview::PlatformWebview;
use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use windows::core::Interface;

/// While the pill is hidden, WebView2 is asked to keep as little in RAM as it can (it may trim
/// caches and move memory to disk). Unlike suspending, scripts keep running, which the hidden
/// page needs in order to hear when to come back; Microsoft's docs suggest this API for
/// exactly that case. Best effort: WebView2 decides how much it frees.
pub fn adjust_pill_webview_memory_target(pill_window: &WebviewWindow, is_pill_hidden: bool) -> Result<(), String> {
    let memory_usage_target_level = if is_pill_hidden {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    pill_window
        .with_webview(move |pill_platform_webview| {
            if let Err(memory_target_error) =
                apply_memory_usage_target_level(&pill_platform_webview, memory_usage_target_level)
            {
                eprintln!("Crest: could not change the webview's memory target: {memory_target_error}");
            }
        })
        .map_err(|error| error.to_string())
}

fn apply_memory_usage_target_level(
    pill_platform_webview: &PlatformWebview,
    memory_usage_target_level: COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL,
) -> windows::core::Result<()> {
    // SAFETY: Tauri runs this closure on the thread that owns the webview, whose COM objects
    // stay alive for the call. ICoreWebView2_19 exists since WebView2 runtime 1.0.1823 (2023);
    // Windows 11 keeps the runtime updated, and on an older one the cast fails harmlessly.
    unsafe {
        let pill_core_webview = pill_platform_webview.controller().CoreWebView2()?;
        pill_core_webview
            .cast::<ICoreWebView2_19>()?
            .SetMemoryUsageTargetLevel(memory_usage_target_level)
    }
}
