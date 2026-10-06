// A security descriptor that lets only the current Windows user open the pipe. The default
// one also gives "Everyone" read access; for a channel that can approve Claude Code's tool
// calls, nobody else may connect. Testing it here also shows whether a session inside the
// Claude app's MSIX container still counts as the same user.

use std::ffi::c_void;

use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

pub struct UserOnlyPipeSecurity {
    pub current_user_sid: String,
    security_attributes: SECURITY_ATTRIBUTES,
}

impl UserOnlyPipeSecurity {
    pub fn for_current_user() -> Result<Self, String> {
        let current_user_sid = read_current_user_sid()?;
        // D:P = protected access list (nothing inherited); A;;GA;;;<sid> = full access for that user only.
        let access_rules = HSTRING::from(format!("D:P(A;;GA;;;{current_user_sid})"));
        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(&access_rules, SDDL_REVISION_1, &mut security_descriptor, None)
        }
        .map_err(|error| format!("building the pipe's access rules failed: {error}"))?;
        // The descriptor lives as long as the probe; Windows frees it when the process ends.
        let security_attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: security_descriptor.0,
            bInheritHandle: false.into(),
        };
        Ok(Self { current_user_sid, security_attributes })
    }

    pub fn attributes(&self) -> *const SECURITY_ATTRIBUTES {
        &self.security_attributes
    }
}

fn read_current_user_sid() -> Result<String, String> {
    let mut process_token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut process_token) }
        .map_err(|error| format!("opening the process token failed: {error}"))?;
    let mut needed_length = 0u32;
    // The first call only reports the size the second call needs.
    let _ = unsafe { GetTokenInformation(process_token, TokenUser, None, 0, &mut needed_length) };
    let mut token_user_buffer = vec![0u8; needed_length as usize];
    let read_result = unsafe {
        GetTokenInformation(
            process_token,
            TokenUser,
            Some(token_user_buffer.as_mut_ptr() as *mut c_void),
            needed_length,
            &mut needed_length,
        )
    };
    let _ = unsafe { CloseHandle(process_token) };
    read_result.map_err(|error| format!("reading the token's user failed: {error}"))?;
    let token_user = unsafe { &*(token_user_buffer.as_ptr() as *const TOKEN_USER) };
    let mut sid_text = PWSTR::null();
    unsafe { ConvertSidToStringSidW(token_user.User.Sid, &mut sid_text) }
        .map_err(|error| format!("turning the user's SID into text failed: {error}"))?;
    let current_user_sid = unsafe { sid_text.to_string() }.map_err(|error| error.to_string());
    unsafe { LocalFree(Some(HLOCAL(sid_text.0 as *mut c_void))) };
    current_user_sid
}
