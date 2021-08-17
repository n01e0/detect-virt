pub mod detect;

use std::convert::From;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Container {
    systemd_nspawn,
    lxc_libvirt,
    lxc,
    OpenVZ,
    Docker,
    Podman,
    rkt,
    WSL,
    PRoot,
    pouch,
    None,
    Other,
}

impl Display for Container {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Container {
    pub fn is_container(&self) -> bool {
        *self != Container::None
    }

    pub fn list() -> Vec<Container> {
        use Container::*;
        vec![
            systemd_nspawn,
            lxc_libvirt,
            lxc,
            OpenVZ,
            Docker,
            Podman,
            rkt,
            WSL,
            PRoot,
            pouch,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        use Container::*;
        match self {
            systemd_nspawn => "systemd_nspawn",
            lxc_libvirt => "lxc_libvirt",
            lxc => "lxc",
            OpenVZ => "OpenVZ",
            Docker => "Docker",
            Podman => "Podman",
            rkt => "rkt",
            WSL => "WSL",
            PRoot => "PRoot",
            pouch => "pouch",
            None => "none",
            Other => "other",
        }

    }
}

impl<'s> From<&'s str> for Container {
    fn from(s: &'s str) -> Self {
        use Container::*;
        match s {
            "lxc" => lxc,
            "lxv-libvirt" => lxc_libvirt,
            "systemd-nspawn" => systemd_nspawn,
            "docker" => Docker,
            "podman" => Podman,
            "rkt" => rkt,
            "wsl" => WSL,
            "proot" => PRoot,
            "pouch" => pouch,
            _ => Container::Other,
        }
    }
}

impl From<String> for Container {
    fn from(s: String) -> Self {
        Container::from(&s[..])
    }
}
