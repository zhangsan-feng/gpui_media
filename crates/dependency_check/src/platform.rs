use std::env;

pub struct Platform {
    pub gstreamer_url: String,
    pub gstreamer_file: String,
    pub gstreamer_bin: String,
    pub save_path: String,
    pub application: String,
}

impl Platform {
    pub fn new() -> Platform {
        #[cfg(windows)]
        {
            let mut gstreamer_bin =
                std::path::PathBuf::from(env::var_os("LOCALAPPDATA").unwrap_or_default());
            gstreamer_bin.push("Programs");
            gstreamer_bin.push("gstreamer");
            gstreamer_bin.push("1.0");
            gstreamer_bin.push("msvc_x86_64");
            gstreamer_bin.push("bin");

            return Platform{
                gstreamer_url: "https://gstreamer.freedesktop.org/data/pkg/windows/1.28.4/msvc/gstreamer-1.0-msvc-x86_64-1.28.4.exe".to_string(),
                gstreamer_file: "gstreamer-1.0-msvc-x86_64-1.28.4.exe".to_string(),
                gstreamer_bin: gstreamer_bin.to_string_lossy().to_string(),
                save_path: env::temp_dir().to_string_lossy().to_string(),
                application: "gui.exe".to_string(),
            };
        }
    }

    pub fn check_dependencies(&self) -> bool {
        #[cfg(windows)]
        {
            return self.check_dependencies_for_windows();
        }
    }

    pub fn install_dependencies(&self) {
        #[cfg(windows)]
        {
            self.install_dependencies_for_windows()
        }
    }

    pub fn add_evn_path(&self) {
        #[cfg(windows)]
        {
            self.add_evn_path_for_windows()
        }
    }

    pub fn start_app(&self) {
        #[cfg(windows)]
        {
            self.start_app_for_windows()
        }
    }

    fn check_dependencies_for_windows(&self) -> bool {
        let gst_commands = ["gst-launch-1.0", "gst-inspect-1.0"];

        if gst_commands.iter().any(|command| {
            std::process::Command::new(command)
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }) {
            return true;
        }

        let gst_exes = ["gst-launch-1.0.exe", "gst-inspect-1.0.exe"];
        let gst_bin = std::path::PathBuf::from(&self.gstreamer_bin);
        gst_bin.is_dir() && gst_exes.iter().any(|exe| gst_bin.join(exe).is_file())
    }

    fn install_dependencies_for_windows(&self) {
        let save_path = if self.save_path.is_empty() {
            env::temp_dir()
        } else {
            std::path::PathBuf::from(&self.save_path)
        };
        let installer = save_path.join(&self.gstreamer_file);

        let _ = std::process::Command::new(installer).arg("/S").status();
    }

    fn add_evn_path_for_windows(&self) {
        let gst_bin = std::path::PathBuf::from(&self.gstreamer_bin);

        let mut paths = env::var_os("PATH")
            .map(|path| env::split_paths(&path).collect::<Vec<_>>())
            .unwrap_or_default();

        if !paths.iter().any(|path| path == &gst_bin) {
            paths.push(gst_bin);
            if let Ok(path) = env::join_paths(paths) {
                unsafe {
                    env::set_var("PATH", path);
                }
            }
        }
    }
    fn start_app_for_windows(&self) {
        let _ = std::process::Command::new(&self.application).spawn();
    }

    // fn check_dependencies_for_linux(&self) -> Platform{}
    // fn check_dependencies_for_mac(&self) -> Platform{}
}
