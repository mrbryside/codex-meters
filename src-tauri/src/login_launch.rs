#[cfg(target_os = "macos")]
pub fn set_enabled(_app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use auto_launch::{AutoLaunch, MacOSLaunchMode};

    let path = std::env::current_exe()
        .map_err(|error| format!("Could not find app executable: {error}"))?;
    let path = path.to_string_lossy().into_owned();
    let bundle_ids = ["com.codex.tokenmeter"];
    let args: [&str; 0] = [];
    let login_item = AutoLaunch::new(
        "Codex Meters",
        &path,
        MacOSLaunchMode::SMAppService,
        &args,
        &bundle_ids,
        "",
    );
    let launch_agent = AutoLaunch::new(
        "Codex Meters",
        &path,
        MacOSLaunchMode::LaunchAgent,
        &args,
        &bundle_ids,
        "",
    );

    if enabled {
        login_item
            .enable()
            .or_else(|_| launch_agent.enable())
            .map_err(|error| format!("Could not enable launch at login: {error}"))
    } else {
        let _ = login_item.disable();
        let _ = launch_agent.disable();
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_enabled(_app: &tauri::AppHandle, _enabled: bool) -> Result<(), String> {
    Ok(())
}
