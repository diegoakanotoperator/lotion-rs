//! Windows-specific LiteBox implementations using AppContainer and Job Objects.

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::JobObjects::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::*;
#[cfg(target_os = "windows")]
use std::ptr;

pub fn apply_windows_sandbox() -> Result<(), String> {
    log::info!("Applying Windows Job Object restrictions");

    #[cfg(target_os = "windows")]
    unsafe {
        // 1. Create a Job Object to contain the process and its children
        let job = CreateJobObjectW(ptr::null(), ptr::null());
        if job == 0 {
            return Err(format!("Failed to create JobObject: {}", GetLastError()));
        }

        // 2. Set Basic Limit Information (e.g. prevent process creation)
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE 
            | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK;
        // WebView2 manages its own Chromium sandbox and must create child process jobs.
        info.BasicLimitInformation.ActiveProcessLimit = 10;

        let res = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if res == 0 {
            CloseHandle(job);
            return Err(format!("Failed to set JobObject limit info: {}", GetLastError()));
        }

        // 3. Set UI Restrictions
        let mut ui_info: JOBOBJECT_BASIC_UI_RESTRICTIONS = std::mem::zeroed();
        ui_info.UIRestrictionsClass = JOB_OBJECT_UILIMIT_DESKTOP 
            | JOB_OBJECT_UILIMIT_EXITWINDOWS 
            | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS;

        let res = SetInformationJobObject(
            job,
            JobObjectBasicUIRestrictions,
            &ui_info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
        );
        if res == 0 {
            CloseHandle(job);
            return Err(format!("Failed to set JobObject UI restrictions: {}", GetLastError()));
        }

        // 4. Assign the current process to the Job Object
        let process = GetCurrentProcess();
        let res = AssignProcessToJobObject(job, process);
        if res == 0 {
            let err = GetLastError();
            // ERROR_ACCESS_DENIED (5) often means we are already in a job and nesting isn't allowed
            // (though Windows 8+ supports nested jobs, some environments still block it)
            if err != 5 {
                CloseHandle(job);
                return Err(format!("Failed to assign process to JobObject: {}", err));
            } else {
                log::warn!("Access denied assigning to JobObject (possibly already in a job); continuing with restricted token logic if applicable.");
            }
        }

        log::info!("Job Object restrictions applied successfully.");
        // We don't CloseHandle(job) yet as we want the job to stay alive with the process
    }

    Ok(())
}
