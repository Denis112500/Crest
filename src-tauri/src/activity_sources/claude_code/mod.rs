//! The Claude Code activity: what a running Claude Code session is doing, from its hook events.

mod claude_code_activity_source;
mod claude_code_hook_event;
mod claude_code_hook_registration;
mod claude_code_integration_switch;
mod claude_code_interrupt_watch;
mod claude_code_session_tracker;
mod claude_code_settings_file;
mod claude_code_status_message;
mod claude_code_tool_summary;
mod claude_code_transcript_interrupt;
mod transcript_tail_reader;

pub use claude_code_hook_registration::{build_crest_hook_command, build_crest_hooks};
pub use claude_code_integration_switch::{turn_claude_code_integration_off, turn_claude_code_integration_on};
pub use claude_code_settings_file::claude_code_settings_file_path;
