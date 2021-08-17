use super::Virtualization;
use raw_cpuid::CpuId;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Result};
use std::path::Path;
use parse_int::parse;

pub fn vm() -> Result<Virtualization> {
    let dmi = detect_dmi()?;
    let mut other = false;
    match dmi {
        Virtualization::Oracle | Virtualization::Amazon => return Ok(dmi),
        Virtualization::Xen => {
            let dom0 = detect_xen_dom0()?;
            if dom0 == XenFeat::Dom0 {
                return Ok(Virtualization::None)
            }
        },
        _ => (),
    }
    if detect_uml()? { return Ok(Virtualization::UML) }

    if let Some(vm) = detect_cpuid() {
        if vm == Virtualization::Other {
            other = true;
        } else {
            return Ok(vm);
        }
    }

    if dmi == Virtualization::Other { other = true }
    if dmi != Virtualization::None { return Ok(dmi) }

    if detect_xen() {
        return Ok(Virtualization::Xen);
    }

    let hv = detect_hypervisor()?;
    if hv == Virtualization::Other { other = true }
    if hv != Virtualization::None { return Ok(hv) }

    let dev = detect_device_tree()?;
    if dev == Virtualization::Other { other = true }
    if dev != Virtualization::None { return Ok(dev) }

    let zvm = detect_zvm()?;
    if zvm != Virtualization::None { return Ok(zvm) }

    if other {return Ok(Virtualization::Other) }

    Ok(Virtualization::None)
}

fn detect_dmi() -> Result<Virtualization> {
    use Virtualization::*;
    #[cfg(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    {
        let r = detect_dmi_vendor()?;
        let smbios = detect_smbios()?;
        if r == Amazon && smbios == SMBIOS::VM_BIT_UNSET {
            return Ok(None);
        }
        if r == None && smbios == SMBIOS::VM_BIT_SET {
            return Ok(Virtualization::Other);
        }
        return Ok(r);
    }
    Ok(None)
}

fn detect_dmi_vendor() -> Result<Virtualization> {
    #[cfg(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    ))]
    {
        use Virtualization::*;
        let dmi_vendors = vec![
            "/sys/class/dmi/id/product_name",
            "/sys/class/dmi/id/sys_vendor",
            "/sys/class/dmi/id/board_vendor",
            "/sys/class/dmi/id/bios_vendor",
        ];

        let dmi_vendor_table = vec![
            ("KVM", KVM),
            ("Amazon EC2", Amazon),
            ("QEMU", QEMU),
            ("VMware", VMware),
            ("VMW", VMware),
            ("innotek GmbH", Oracle),
            ("Oracle Corporation", Oracle),
            ("Xen", Xen),
            ("Bochs", Bochs),
            ("Parallels", Parallels),
            ("BHYVE", Bhyve),
        ];

        for dmi_vendor in dmi_vendors {
            if !Path::new(dmi_vendor).exists() {
                continue;
            }
            let mut reader = BufReader::new(File::open(dmi_vendor)?);
            let mut buf = String::new();
            reader.read_line(&mut buf)?;
            for vendor in &dmi_vendor_table {
                if buf.starts_with(vendor.0) {
                    return Ok(vendor.1);
                }
            }
        }
    }

    Ok(Virtualization::None)
}

#[derive(Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
enum SMBIOS {
    VM_BIT_SET,
    VM_BIT_UNSET,
    VM_BIT_UNKNOWN,
}

macro_rules! unwrap_or_return {
    ($r:expr, $d:expr) => {
        match $r {
            Ok(o) => o,
            Err(_) => return $d,
        }
    };
}

fn detect_smbios() -> Result<SMBIOS> {
    let mut f = unwrap_or_return!(
        File::open("/sys/firmware/dmi/entries/0-0/raw"),
        Ok(SMBIOS::VM_BIT_UNKNOWN)
    );
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    if buf.len() < 20 || buf[1] < 20 {
        return Ok(SMBIOS::VM_BIT_UNKNOWN);
    }

    let byte = buf[19];
    if byte & (1 << 4) != 0 {
        Ok(SMBIOS::VM_BIT_SET)
    } else {
        Ok(SMBIOS::VM_BIT_UNSET)
    }
}

fn detect_uml() -> Result<bool> {
    let cpuinfo = Path::new("/proc/cpuinfo");
    if !cpuinfo.exists() {
        Ok(false)
    } else {
        Ok(!BufReader::new(File::open(cpuinfo)?)
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| l.starts_with("vendor_id\t: "))
            .map(|s| s.split("\t: ").collect::<Vec<_>>()[1].to_owned())
            .filter(|s| s.starts_with("User Mode Linux"))
            .collect::<Vec<_>>()
            .is_empty())
    }
}

fn detect_cpuid() -> Option<Virtualization> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64",))]
    {
        let cpuid = CpuId::new();
        if let Some(hv) = cpuid.get_hypervisor_info() {
            return Some(Virtualization::from(hv.identify()));
        }
    }
    None
}

fn detect_xen() -> bool {
    Path::new("/proc/xen").exists()
}

fn detect_hypervisor() -> Result<Virtualization> {
    let hv_type = Path::new("/sys/hypervisor/type");
    if !hv_type.exists() {
        Ok(Virtualization::None)
    } else {
        let mut reader = BufReader::new(File::open(hv_type)?);
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        if &buf[..] == "xen\n" {
            Ok(Virtualization::Xen)
        } else {
            Ok(Virtualization::Other)
        }
    }
}

fn detect_device_tree() -> Result<Virtualization> {
    #[cfg(any(
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "powerpc",
        target_arch = "powerpc64"
    ))]
    {
        let compatible = Path::new("/proc/device-tree/hypervisor/compatible");
        if !compatible.exists() {
            if Path::new("/proc/device-tree/ibm,partition-name").exists()
                && Path::new("/proc/device-tree/hmc-managed?").exists()
                && Path::new("/proc/device-tree/chosen/qemu,graphic-width").exists()
            {
                return Ok(Virtualization::PowerVM);
            }

            let device_tree = Path::new("/proc/device-tree");
            if !device_tree.exists() {
                return Ok(Virtualization::None);
            }

            for entry in fs::read_dir(device_tree)? {
                let entry = entry?;
                let path = entry.path();
                if path.as_str().contains("fw-ctf") {
                    return Ok(Virtualization::QEMU);
                }
            }
            return Ok(Virtualization::None);
        } else {
            let mut reader = BufReader::new(File::open(path)?);
            let mut buf = String::new();
            reader.read_line(&mut buf);
            return match &buf[..] {
                "linux,kvm\n" => Ok(Virtualization::KVM),
                "xen\n" => Ok(Virtualization::Xen),
                "vmware\n" => Ok(Virtualization::VMware),
                _ => Ok(Virtualization::Other),
            }
        }
    }
    Ok(Virtualization::None)
}

fn detect_zvm() -> Result<Virtualization> {
    #[cfg(target_arch = "s390")]
    {
        let sysinfo = Path::new("/proc/sysinfo");
        if !sysinfo.exists() { return Ok(Virtualization::None) }

        return if BufReader::new(File::open(sysinfo)?)
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| l.starts_with("VM00 Control Program"))
            .map(|s| s.split(":").collect::<Vec<_>>()[1].to_owned())
            .filter(|s| s.contains("z/VM"))
            .collect::<Vec<_>>()
            .is_empty()
        {
            Ok(Virtualization::KVM)
        } else {
            Ok(Virtualization::ZVM) 
        }
    }
    Ok(Virtualization::None)
}

#[derive(Debug, Eq, PartialEq)]
enum XenFeat {
    DomU,
    Dom0,
    None,
}

fn detect_xen_dom0() -> Result<XenFeat> {
    let features = Path::new("/sys/hypervisor/properties/features");
    let xenfeat_dom0 = 11;
    if !features.exists() { return Ok(XenFeat::None) }
    let mut buf = Vec::new();
    File::open(features)?.read_to_end(&mut buf)?;

    if let Ok(features) = parse::<u32>(&String::from_utf8_lossy(&buf[..])) {
        if features & (1 << xenfeat_dom0) != 0 { return Ok(XenFeat::Dom0) }
    }

    let cap = Path::new("/proc/xen/capabilities");
    if !cap.exists() { return Ok(XenFeat::None) }

    let mut reader = BufReader::new(File::open(cap)?);
    let mut domcap = String::new();
    reader.read_line(&mut domcap);
    let cap = domcap.split(",").collect::<Vec<_>>();
    if cap.is_empty() {
        return Ok(XenFeat::DomU)
    } else if cap[0] == "control_d" {
        return Ok(XenFeat::Dom0)
    }

    Ok(XenFeat::None)

}
