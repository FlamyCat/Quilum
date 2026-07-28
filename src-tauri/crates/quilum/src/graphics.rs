use std::{fs, path::Path};

/// Check if the computer is running on NVIDIA + nouveau driver.
pub fn has_nvidia_nouveau() -> bool {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if !entry_is_a_gpu(&name) {
            continue;
        }

        let device_dir = entry.path().join("device");

        let vendor = fs::read_to_string(device_dir.join("vendor")).unwrap_or_default();
        if !is_nvidia(&vendor) {
            continue;
        }

        if driver_is_nouveau(&device_dir) {
            return true;
        }
    }

    false
}

fn entry_is_a_gpu(name: &str) -> bool {
    name.starts_with("card") && !name.contains('-')
}

fn driver_is_nouveau(device_dir: &Path) -> bool {
    if let Ok(target) = fs::read_link(device_dir.join("driver"))
        && target.file_name().and_then(|s| s.to_str()) == Some("nouveau")
    {
        return true;
    }

    false
}

fn is_nvidia(vendor: &str) -> bool {
    let nvidia_vendor_code = "0x10de";
    vendor.trim() == nvidia_vendor_code
}
