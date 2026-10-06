//! Who is who on the pipe: the current user's SID (for the pipe's name and its access rule) and
//! the SID of the process at the other end (so the hook only talks to a Crest run by the same user).

use std::ffi::c_void;

use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION};

pub fn read_current_user_sid() -> Result<String, String> {
    // SAFETY: the pseudo handle of the current process needs no closing.
    read_process_user_sid(unsafe { GetCurrentProcess() })
}

/// The user a running process belongs to (e.g. the Crest at the other end of the pipe).
pub fn read_user_sid_of_process(process_id: u32) -> Result<String, String> {
    // SAFETY: the handle is closed below; "limited information" is all that reading the owner needs.
    let process_handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }
        .map_err(|error| format!("opening process {process_id} failed: {error}"))?;
    let process_user_sid = read_process_user_sid(process_handle);
    let _ = unsafe { CloseHandle(process_handle) };
    process_user_sid
}

fn read_process_user_sid(process_handle: HANDLE) -> Result<String, String> {
    let mut process_token = HANDLE::default();
    // SAFETY: the token handle is closed below; every buffer outlives the call that fills it.
    unsafe { OpenProcessToken(process_handle, TOKEN_QUERY, &mut process_token) }
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
    // SAFETY: Windows filled the buffer with a TOKEN_USER followed by the SID it points to.
    let token_user = unsafe { &*(token_user_buffer.as_ptr() as *const TOKEN_USER) };
    let mut sid_text = PWSTR::null();
    unsafe { ConvertSidToStringSidW(token_user.User.Sid, &mut sid_text) }
        .map_err(|error| format!("turning the SID into text failed: {error}"))?;
    let user_sid = unsafe { sid_text.to_string() }.map_err(|error| error.to_string());
    unsafe { LocalFree(Some(HLOCAL(sid_text.0 as *mut c_void))) };
    user_sid
}

/// Security attributes that let only one user open the pipe. The default ones also give
/// "Everyone" read access; for a channel that will carry Allow/Deny answers, nobody else may connect.
pub struct UserOnlyPipeAccess {
    security_attributes: SECURITY_ATTRIBUTES,
    security_descriptor: PSECURITY_DESCRIPTOR,
}

impl UserOnlyPipeAccess {
    pub fn for_user(user_sid: &str) -> Result<Self, String> {
        // D:P = protected access list (nothing inherited); A;;GA;;;<sid> = full access for that user only.
        let access_rules = HSTRING::from(format!("D:P(A;;GA;;;{user_sid})"));
        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();
        // SAFETY: Windows allocates the descriptor; it's freed in `drop`.
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(&access_rules, SDDL_REVISION_1, &mut security_descriptor, None)
        }
        .map_err(|error| format!("building the pipe's access rule failed: {error}"))?;
        Ok(Self {
            security_attributes: SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: security_descriptor.0,
                bInheritHandle: false.into(),
            },
            security_descriptor,
        })
    }

    pub fn security_attributes(&self) -> *const SECURITY_ATTRIBUTES {
        &self.security_attributes
    }
}

impl Drop for UserOnlyPipeAccess {
    fn drop(&mut self) {
        // SAFETY: allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW, freed once.
        unsafe { LocalFree(Some(HLOCAL(self.security_descriptor.0))) };
    }
}
