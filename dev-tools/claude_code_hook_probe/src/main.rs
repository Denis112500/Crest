// Dev tool, not part of Crest: stands in for Crest to find out how Claude Code's hooks can
// reach it locally, and whether an Allow/Deny answer gets back. Two modes:
//
//   listen   prints every hook event that arrives, over a named pipe that only the current
//            Windows user may open and over http://127.0.0.1:<port>; answers PermissionRequest
//            events as told (after a delay, to see whether Claude Code's own dialog waits).
//   forward  what a `command` hook runs: reads the event from stdin, passes it to the
//            listener over the pipe and prints the answer for Claude Code. Without a running
//            listener it exits at once without output (= "no decision").
//
//   cd dev-tools/claude_code_hook_probe
//   cargo run --release -- listen [--permission-answer none|allow|deny] [--answer-delay-seconds 5] [--http-port 47615]
//
// The hooks are written into one test folder by dev-tools/write_hook_probe_session_settings.ps1.
mod hook_event_fields;
mod hook_event_forwarder;
mod permission_reply;
mod probe_http_listener;
mod probe_log_line;
mod probe_pipe_listener;
mod user_only_pipe_security;

use std::thread;
use std::time::Duration;

use crate::permission_reply::PermissionAnswer;

const DEFAULT_HTTP_PORT: u16 = 47615;

/// How the listener answers; shared by the pipe and the HTTP side.
#[derive(Clone, Copy)]
pub struct ListenOptions {
    pub permission_answer: PermissionAnswer,
    pub answer_delay: Duration,
    pub http_port: u16,
}

fn main() {
    let command_line_arguments: Vec<String> = std::env::args().skip(1).collect();
    match command_line_arguments.first().map(String::as_str) {
        Some("forward") => hook_event_forwarder::forward_hook_event_to_listener(),
        Some("listen") => match read_listen_options(&command_line_arguments[1..]) {
            Ok(listen_options) => run_both_listeners(listen_options),
            Err(problem) => exit_with_usage(&problem),
        },
        _ => exit_with_usage("expected \"listen\" or \"forward\""),
    }
}

fn run_both_listeners(listen_options: ListenOptions) {
    println!(
        "Answering PermissionRequest with \"{}\" after {} s. Ctrl+C to stop.",
        listen_options.permission_answer.label(),
        listen_options.answer_delay.as_secs()
    );
    let http_listener_thread = thread::spawn(move || {
        if let Err(problem) = probe_http_listener::run_http_listener(listen_options) {
            eprintln!("HTTP listener stopped: {problem}");
        }
    });
    if let Err(problem) = probe_pipe_listener::run_pipe_listener(listen_options) {
        eprintln!("Pipe listener stopped: {problem}");
    }
    let _ = http_listener_thread.join();
}

fn read_listen_options(listen_arguments: &[String]) -> Result<ListenOptions, String> {
    let mut listen_options = ListenOptions {
        permission_answer: PermissionAnswer::NoDecision,
        answer_delay: Duration::ZERO,
        http_port: DEFAULT_HTTP_PORT,
    };
    let mut remaining_arguments = listen_arguments.iter();
    while let Some(option_name) = remaining_arguments.next() {
        let option_value = remaining_arguments.next().ok_or(format!("{option_name} needs a value"))?;
        match option_name.as_str() {
            "--permission-answer" => {
                listen_options.permission_answer = PermissionAnswer::from_label(option_value)
                    .ok_or(format!("unknown answer \"{option_value}\" (none, allow, deny)"))?;
            }
            "--answer-delay-seconds" => {
                let delay_seconds: u64 = option_value.parse().map_err(|_| format!("not a number: {option_value}"))?;
                listen_options.answer_delay = Duration::from_secs(delay_seconds);
            }
            "--http-port" => {
                listen_options.http_port = option_value.parse().map_err(|_| format!("not a port: {option_value}"))?;
            }
            _ => return Err(format!("unknown option {option_name}")),
        }
    }
    Ok(listen_options)
}

fn exit_with_usage(problem: &str) -> ! {
    eprintln!("{problem}");
    eprintln!("usage: claude_code_hook_probe listen [--permission-answer none|allow|deny] [--answer-delay-seconds N] [--http-port N]");
    eprintln!("       claude_code_hook_probe forward   (run by a Claude Code command hook)");
    std::process::exit(1);
}
