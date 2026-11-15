use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
pub fn verify(_window: &WebviewWindow, prompt: bool) -> bool {
    extern "C" {
        fn cuetuck_touch_id(prompt: i32) -> i32;
    }
    unsafe { cuetuck_touch_id(i32::from(prompt)) == 1 }
}

#[cfg(target_os = "windows")]
pub fn verify(window: &WebviewWindow, prompt: bool) -> bool {
    use windows::core::{factory, Result, HSTRING};
    use windows::Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    };
    use windows::Win32::{Foundation::HWND, System::WinRT::IUserConsentVerifierInterop};
    use windows_future::IAsyncOperation;
    let check = || -> Result<bool> {
        let interop: IUserConsentVerifierInterop = factory::<UserConsentVerifier, _>()?;
        if !prompt {
            return Ok(UserConsentVerifier::CheckAvailabilityAsync()?.get()?
                == UserConsentVerifierAvailability::Available);
        }
        let hwnd = HWND(
            window
                .hwnd()
                .map_err(|_| windows::core::Error::from_win32())?
                .0,
        );
        let operation: IAsyncOperation<UserConsentVerificationResult> = unsafe {
            interop.RequestVerificationForWindowAsync(hwnd, &HSTRING::from("解锁 CueTuck"))?
        };
        Ok(operation.get()? == UserConsentVerificationResult::Verified)
    };
    check().unwrap_or(false)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn verify(_window: &WebviewWindow, _prompt: bool) -> bool {
    false
}
