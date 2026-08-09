use std::process::Command;

pub fn is_wireguard_available() -> bool {
    let ok: bool;

    #[cfg(target_os = "windows")] {
        use crate::util::terminal::WINDOWS_INVISIBLE_TERMIAL;
        use std::os::windows::process::CommandExt;

        ok = Command::new(r"C:\Program Files\WireGuard\wg.exe")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .arg("--version")
            .output()
            .is_ok();
    };

    #[cfg(target_os = "linux")] {
        ok = Command::new("wg")
            .arg("--version")
            .output()
            .is_ok();
    }

    ok
}

/*
TODO: implement this
https://download.wireguard.com/windows-client/latest.sig -> parse -> download -> check signature -> install using msiexec.exe
pub async fn install_wireguard() -> Result<(), WireguardInstallError> {
    #[cfg(target_os = "windows")]
    {
        return install_wireguard_windows().await
    }
}*/
