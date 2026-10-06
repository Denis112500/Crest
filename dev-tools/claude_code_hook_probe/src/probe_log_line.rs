// One timestamped console line per event, so pipe and HTTP arrivals can be compared.

use windows::Win32::System::SystemInformation::GetLocalTime;

pub fn print_probe_line(transport_label: &str, line_text: &str) {
    let local_time = unsafe { GetLocalTime() };
    println!(
        "{:02}:{:02}:{:02}.{:03}  {transport_label:<4}  {line_text}",
        local_time.wHour, local_time.wMinute, local_time.wSecond, local_time.wMilliseconds
    );
}
