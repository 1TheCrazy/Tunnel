use std::process::Command;

#[cfg(target_os = "windows")]
const BIN: &str = r"C:\Program Files\WireGuard\wg.exe";

#[cfg(not(target_os = "windows"))]
const BIN: &str = r"wg";

pub fn is_wireguard_available() -> bool {
    Command::new(BIN).arg("--version").output().is_ok()
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
