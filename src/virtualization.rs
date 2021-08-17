pub mod detect;

use std::convert::From;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Virtualization {
    KVM,
    Amazon,
    QEMU,
    Bochs,
    Xen,
    UML,
    VMware,
    Oracle,
    MicroSoft,
    ZVM,
    Parallels,
    Bhyve,
    QNX,
    ACRN,
    PowerVM,
    None,
    Other,
}

impl Display for Virtualization {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Virtualization {
    pub fn is_vm(&self) -> bool {
        *self != Virtualization::None
    }

    pub fn list() -> Vec<Virtualization> {
        use Virtualization::*;
        vec![
            KVM, Amazon, QEMU, Bochs, Xen, UML, VMware, Oracle, MicroSoft, ZVM, Parallels, Bhyve,
            QNX, ACRN, PowerVM,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        use Virtualization::*;
        match self {
            KVM => "KVM",
            Amazon => "Amazon",
            QEMU => "QEMU",
            Bochs => "Bochs",
            Xen => "Xen",
            UML => "UML",
            VMware => "VMware",
            Oracle => "Oracle",
            MicroSoft => "MicroSoft",
            ZVM => "ZVM",
            Parallels => "Parallels",
            Bhyve => "Bhyve",
            QNX => "QNX",
            ACRN => "ACRN",
            PowerVM => "PowerVM",
            Other => "other",
            None => "none",
        }
    }
}

impl From<raw_cpuid::Hypervisor> for Virtualization {
    fn from(ident: raw_cpuid::Hypervisor) -> Virtualization {
        use raw_cpuid::Hypervisor::*;
        match ident {
            Xen => Virtualization::Xen,
            VMware => Virtualization::VMware,
            HyperV => Virtualization::MicroSoft,
            KVM => Virtualization::KVM,
            QEMU => Virtualization::QEMU,
            Bhyve => Virtualization::Bhyve,
            QNX => Virtualization::QNX,
            ACRN => Virtualization::ACRN,
            Unknown(_, _, _) => Virtualization::Other,
        }
    }
}
