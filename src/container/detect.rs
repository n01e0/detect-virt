use super::Container;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Result};
use std::path::Path;
use std::env;
use procfs::process::Process;

pub fn container() -> Container {
    if detect_vz() { return Container::OpenVZ }

    if detect_wsl() { return Container::WSL }

    if detect_proot() { return Container::PRoot }
    
    if let Some(container) = check_container_manager() {
        return container;
    }

    if let Some(container) = detect_systemd_container() {
        return container;
    }

    if let Some(container) = detect_init_env() {
        return container;
    }

    Container::None
}

fn detect_vz() -> bool {
    Path::new("/proc/vz").exists() && !Path::new("/proc/bc").exists()
}

fn detect_wsl() -> bool {
    let osrelease = Path::new("/proc/sys/kernel/osrelease");
    if osrelease.exists() {
        if let Ok(osrelease) = File::open(osrelease) {
            let mut reader = BufReader::new(osrelease);
            let mut buf = String::new();
            let _ = reader.read_line(&mut buf);
            if buf.contains("Microsoft") || buf.contains("WSL") {
                return true
            }
        }
    }

    return false
}

fn detect_proot() -> bool {
    if let Ok(status) = Process::myself().and_then(|s| s.status()) {
        let tracerpid = status.tracerpid;
        if tracerpid != 0 {
            if let Ok(ptrace_comm) = File::open(format!("/proc/{}/comm", tracerpid)) {
                let mut reader = BufReader::new(ptrace_comm);
                let mut buf = String::new();
                let _ = reader.read_line(&mut buf);
                if buf.starts_with("proot") {
                    return true;
                }
            }
        }
    }
    false
}

fn check_container_manager() -> Option<Container> {
    let path = Path::new("/run/systemd/container-manager");
    if !path.exists() { return None }
    if let Ok(container) = File::open(path) {
        let mut reader = BufReader::new(container);
        let mut buf = String::new();
        let _ = reader.read_line(&mut buf);
        return parse_container_from_manager(buf)
    }
    None
}

fn parse_container_from_manager(manager: String) -> Option<Container> {
    return if &manager[..] == "oci" {
        let c = detect_container_files();
        return if c == Container::None {
            None
        } else {
            Some(c)
        }
    } else {
        Some(Container::from(&manager[..]))
    }
}

fn detect_container_files() -> Container {
    let container_file_table = vec![("/run/.containerenv", Container::Podman), ("/.dockerenv", Container::Docker)];
    for container_file in container_file_table {
        if Path::new(container_file.0).exists() {
            return container_file.1;
        }
    }

    Container::Other
}

fn detect_init_env() -> Option<Container> {
    return if std::process::id() == 1 {
        match env::var("container") {
            Ok(v) => Some(Container::from(v)),
            Err(_) => None,
        }
    } else {
        if let Ok(proc) = Process::new(1) {
            if let Ok(e) = proc.environ() {
                e.get(&std::ffi::OsString::from("container")).map(|v| Container::from(v.to_str().unwrap_or("".into())))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn detect_systemd_container() -> Option<Container> {
    let path = Path::new("/run/systemd/container");
    if !path.exists() { return None }
    if let Ok(container) = File::open(path) {
        let mut reader = BufReader::new(container);
        let mut buf = String::new();
        let _ = reader.read_line(&mut buf);
        return Some(Container::from(buf))
    }
    None
}
