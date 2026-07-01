use super::GpuDevice;

pub fn collect() -> Vec<GpuDevice> {
    metal::Device::all()
        .iter()
        .enumerate()
        .map(|(index, device)| {
            let total = device.recommended_max_working_set_size();
            let used = device.current_allocated_size() as u64;
            let name = if index == 0 {
                "Metal".to_string()
            } else {
                format!("Metal{index}")
            };
            GpuDevice {
                index,
                name,
                description: device.name().to_string(),
                vendor: vendor_for(device),
                backend: "Metal".to_string(),
                memory_total: total,
                memory_free: total.saturating_sub(used),
                device_type: "Gpu".to_string(),
            }
        })
        .collect()
}

fn vendor_for(device: &metal::DeviceRef) -> String {
    if device.has_unified_memory() {
        return "apple".to_string();
    }
    let name = device.name().to_lowercase();
    if name.contains("amd") || name.contains("radeon") {
        "amd".to_string()
    } else if name.contains("intel") {
        "intel".to_string()
    } else {
        "apple".to_string()
    }
}
