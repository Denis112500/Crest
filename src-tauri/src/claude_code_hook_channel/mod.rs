//! The local channel between Claude Code's hooks and Crest. The OS-specific part sits behind a
//! trait, like the other platform code.

mod claude_code_hook_channel_trait;
mod claude_code_hook_process;
mod length_prefixed_message;

#[cfg(target_os = "windows")]
mod windows_named_pipe;

pub use claude_code_hook_channel_trait::{ClaudeCodeHookChannel, HookEventHandler, RunningHookEventServer};
pub use claude_code_hook_process::run_as_claude_code_hook;

#[cfg(target_os = "windows")]
pub use windows_named_pipe::NamedPipeClaudeCodeHookChannel as CurrentPlatformClaudeCodeHookChannel;
