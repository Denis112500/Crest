// The loopback HTTP side (option A): Claude Code's `http` hook POSTs the event here and
// reads the answer from the response. A tiny hand-written HTTP/1.1 reader is enough for a
// probe. It also logs requests that aren't POSTs (a browser's CORS preflight is OPTIONS),
// to see what else on this PC can reach a local port.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::hook_event_fields::summarize_hook_event;
use crate::permission_reply::build_reply_to_hook_event;
use crate::probe_log_line::print_probe_line;
use crate::ListenOptions;

const LARGEST_ACCEPTED_BODY_BYTES: usize = 4 * 1024 * 1024;

pub fn run_http_listener(listen_options: ListenOptions) -> Result<(), String> {
    let tcp_listener = TcpListener::bind(("127.0.0.1", listen_options.http_port))
        .map_err(|error| format!("port {} on 127.0.0.1: {error}", listen_options.http_port))?;
    print_probe_line("http", &format!("listening on http://127.0.0.1:{}/hook", listen_options.http_port));
    for incoming_connection in tcp_listener.incoming().flatten() {
        thread::spawn(move || {
            if let Err(problem) = answer_http_request(incoming_connection, listen_options) {
                print_probe_line("http", &format!("a request failed: {problem}"));
            }
        });
    }
    Ok(())
}

fn answer_http_request(mut tcp_connection: TcpStream, listen_options: ListenOptions) -> std::io::Result<()> {
    let mut request_reader = BufReader::new(tcp_connection.try_clone()?);
    let mut request_line = String::new();
    request_reader.read_line(&mut request_line)?;
    let request_method = request_line.split_whitespace().next().unwrap_or_default().to_string();

    let mut body_length = 0usize;
    let mut notable_headers = Vec::new();
    loop {
        let mut header_line = String::new();
        if request_reader.read_line(&mut header_line)? == 0 || header_line.trim_end().is_empty() {
            break;
        }
        if let Some((header_name, header_value)) = header_line.trim_end().split_once(':') {
            let header_name = header_name.trim().to_ascii_lowercase();
            let header_value = header_value.trim();
            match header_name.as_str() {
                "content-length" => body_length = header_value.parse().unwrap_or(0),
                "host" | "user-agent" | "origin" | "content-type" => {
                    notable_headers.push(format!("{header_name}={header_value}"))
                }
                _ => {}
            }
        }
    }
    if body_length > LARGEST_ACCEPTED_BODY_BYTES {
        return write_http_response(&mut tcp_connection, "413 Payload Too Large", "");
    }
    let mut body_bytes = vec![0u8; body_length];
    request_reader.read_exact(&mut body_bytes)?;

    if request_method != "POST" {
        print_probe_line("http", &format!("{request_method} refused  {}", notable_headers.join("  ")));
        return write_http_response(&mut tcp_connection, "405 Method Not Allowed", "");
    }
    let hook_event_summary = summarize_hook_event(&String::from_utf8_lossy(&body_bytes));
    print_probe_line(
        "http",
        &format!(
            "{}  {}  [{}]",
            hook_event_summary.hook_event_name,
            hook_event_summary.details,
            notable_headers.join("  ")
        ),
    );
    let reply_json = build_reply_to_hook_event(
        &hook_event_summary.hook_event_name,
        listen_options.permission_answer,
        listen_options.answer_delay,
    );
    write_http_response(&mut tcp_connection, "200 OK", &reply_json)?;
    if hook_event_summary.hook_event_name == "PermissionRequest" {
        print_probe_line("http", &format!("answered PermissionRequest: {}", listen_options.permission_answer.label()));
    }
    Ok(())
}

fn write_http_response(tcp_connection: &mut TcpStream, status_text: &str, body_text: &str) -> std::io::Result<()> {
    let response_text = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_text}",
        body_text.len()
    );
    tcp_connection.write_all(response_text.as_bytes())
}
