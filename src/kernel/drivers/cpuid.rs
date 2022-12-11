use alloc::vec::Vec;
use alloc::string::String;

use x86::cpuid;
use x86::cpuid::Hypervisor;

#[derive(Debug)]
pub struct Processor {
    pub vendor: String,
    pub family: u8,
    pub model: u8,
    pub model_name: String,
    pub stepping: u8,
    pub cores: Vec<Core>,
    pub hypervisor: Option<&'static str>,
}

#[derive(Debug)]
pub struct Core {
    pub id: u8,
    pub logical_processors: u16
}

pub fn get_processor_info() -> Processor {
    let mut cpu_vendor = String::from("Unknown");
    let mut cpu_family = 0;
    let mut cpu_model = 0;
    let mut cpu_stepping = 0;
    let mut cpu_model_name = String::from("Unknown");
    let mut cpu_cores = Vec::new();
    let mut cpu_hypervisor = None;

    let cpuid = cpuid::CpuId::new();

    if let Some(vendor_info) = cpuid.get_vendor_info() {
        cpu_vendor = String::from(vendor_info.as_str());
    }

    let feature_info = cpuid.get_feature_info();
    if feature_info.is_some() {
        let feature_info = feature_info.unwrap();

        cpu_family = feature_info.family_id();
        cpu_model = feature_info.model_id();
        cpu_stepping = feature_info.stepping_id();
    }

    if let Some(brand_string) = cpuid.get_processor_brand_string() {
        cpu_model_name = String::from(brand_string.as_str());
    }

    let hypervisor_info = cpuid.get_hypervisor_info();
    if hypervisor_info.is_some() {
        cpu_hypervisor = match hypervisor_info.unwrap().identify() {
            Hypervisor::Xen => Some("Xen"),
            Hypervisor::VMware => Some("VMware"),
            Hypervisor::HyperV => Some("Hyper-V"),
            Hypervisor::KVM => Some("KVM"),
            Hypervisor::QEMU => Some("QEMU"),
            Hypervisor::Bhyve => Some("Bhyve"),
            Hypervisor::QNX => Some("QNX"),
            Hypervisor::ACRN => Some("ACRN"),
            Hypervisor::Unknown(x, y, z) => {
                if x == 0x4d4d5648 && y == 0x564d5868 && z == 0x65584d56 {
                    Some("VirtualBox")
                } else {
                    None
                }
            }
        };
    }

    let topology_info = cpuid.get_extended_topology_info();
    if topology_info.is_some() {
        let topology_info = topology_info.unwrap();

        for info in topology_info {
            cpu_cores.push(Core {
                id: info.level_number(),
                logical_processors: info.processors()
            });
        }
    }

    return Processor {
        vendor: cpu_vendor,
        family: cpu_family,
        model: cpu_model,
        model_name: cpu_model_name,
        stepping: cpu_stepping,
        cores: cpu_cores,
        hypervisor: cpu_hypervisor,
    };
}
