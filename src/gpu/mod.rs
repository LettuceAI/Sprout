use serde::Serialize;

#[cfg(not(target_os = "macos"))]
mod amd;
#[cfg(target_os = "macos")]
mod apple;
#[cfg(not(target_os = "macos"))]
mod intel;
#[cfg(not(target_os = "macos"))]
mod nvidia;
#[cfg(not(target_os = "macos"))]
mod vulkan;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuDevice {
    pub index: usize,
    pub name: String,
    pub description: String,
    pub vendor: String,
    pub backend: String,
    pub memory_total: u64,
    pub memory_free: u64,
    pub device_type: String,
}

pub fn collect_gpus() -> Vec<GpuDevice> {
    #[cfg(target_os = "macos")]
    let mut gpus = apple::collect();

    #[cfg(not(target_os = "macos"))]
    let mut gpus = {
        let mut gpus = nvidia::collect();
        let vulkan_devices = vulkan::devices();
        gpus.extend(amd::collect(&vulkan_devices));
        gpus.extend(intel::collect(&vulkan_devices));
        gpus
    };

    for (index, gpu) in gpus.iter_mut().enumerate() {
        gpu.index = index;
    }
    gpus
}
