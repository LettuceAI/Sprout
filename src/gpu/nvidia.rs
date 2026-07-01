use nvml_wrapper::Nvml;

use super::GpuDevice;

pub(super) fn collect() -> Vec<GpuDevice> {
    let nvml = match Nvml::init() {
        Ok(nvml) => nvml,
        Err(err) => {
            tracing::debug!("nvml unavailable, no nvidia gpus reported: {err}");
            return Vec::new();
        }
    };

    let count = match nvml.device_count() {
        Ok(count) => count,
        Err(err) => {
            tracing::warn!("nvml device_count failed: {err}");
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for i in 0..count {
        let device = match nvml.device_by_index(i) {
            Ok(device) => device,
            Err(err) => {
                tracing::warn!("nvml device {i} unavailable: {err}");
                continue;
            }
        };

        let memory = match device.memory_info() {
            Ok(memory) => memory,
            Err(err) => {
                tracing::warn!("nvml memory for device {i} unavailable: {err}");
                continue;
            }
        };

        let description = device.name().unwrap_or_else(|_| format!("NVIDIA GPU {i}"));

        out.push(GpuDevice {
            index: 0,
            name: format!("CUDA{i}"),
            description,
            vendor: "nvidia".to_string(),
            backend: "CUDA".to_string(),
            memory_total: memory.total,
            memory_free: memory.free,
            device_type: "Gpu".to_string(),
        });
    }
    out
}
