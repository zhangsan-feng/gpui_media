use std::env;

pub mod music_player;
pub mod video_player;



unsafe fn append_to_env_unix(key: &str, value: &str) {
    let current = env::var(key).unwrap_or_default();
    let new_value = if current.is_empty() {
        value.to_string()
    } else {
        format!("{}:{}", current, value)
    };
    env::set_var(key, new_value);
}

/// 追加到环境变量（Windows 风格，用分号分隔）
unsafe fn append_to_env_windows(key: &str, value: &str) {
    let current = env::var(key).unwrap_or_default();
    let new_value = if current.is_empty() {
        value.to_string()
    } else {
        format!("{};{}", current, value)
    };
    env::set_var(key, new_value);
}

/// 跨平台追加（自动检测 OS）
pub unsafe fn append_to_env(key: &str, value: &str) {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let current = env::var(key).unwrap_or_default();
    let new_value = if current.is_empty() {
        value.to_string()
    } else {
        format!("{}{}{}", current, separator, value)
    };

    env::set_var(key, new_value);
}

pub unsafe fn append_sys_path(){
    let local_programs = env::var("LOCALAPPDATA").unwrap() + "\\Programs";
    let gst_bin = local_programs + "\\gstreamer\\1.0\\msvc_x86_64\\bin";
    log::info!("{}", gst_bin);

    unsafe {
        let current = env::var("PATH").unwrap_or_default();
        env::set_var("PATH", format!("{};{}", current, gst_bin));
    }
}