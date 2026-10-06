//! Windows: a named pipe that only the current user may open (verified with
//! `dev-tools/claude_code_hook_probe`, also from inside the Claude app's MSIX container).

mod named_pipe_claude_code_hook_channel;
mod named_pipe_hook_event_client;
mod named_pipe_hook_event_server;
mod windows_user_identity;

pub use named_pipe_claude_code_hook_channel::NamedPipeClaudeCodeHookChannel;
