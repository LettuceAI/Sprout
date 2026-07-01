use super::GpuDevice;
use super::vulkan::VulkanDevice;

const AMD_VENDOR_ID: u32 = 0x1002;

pub(super) fn collect(devices: &[VulkanDevice]) -> Vec<GpuDevice> {
    devices
        .iter()
        .filter(|device| device.vendor_id == AMD_VENDOR_ID)
        .map(|device| GpuDevice {
            index: 0,
            name: format!("Vulkan{}", device.index),
            description: device.description.clone(),
            vendor: "amd".to_string(),
            backend: "Vulkan".to_string(),
            memory_total: device.memory_total,
            memory_free: device.memory_free,
            device_type: device.device_type.to_string(),
        })
        .collect()
}
