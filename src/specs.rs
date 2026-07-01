use serde::Serialize;
use sysinfo::System;

use crate::gpu::{collect_gpus, GpuDevice};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecsResponse {
    pub schema_version: u32,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub available_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub unified_memory: bool,
    pub gpus: Vec<GpuDevice>,
}

pub fn collect() -> SpecsResponse {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let cpu_name = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .unwrap_or_default();

    SpecsResponse {
        schema_version: SCHEMA_VERSION,
        hostname: System::host_name().unwrap_or_default(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        available_memory_bytes: sys.available_memory(),
        total_memory_bytes: sys.total_memory(),
        cpu_name,
        cpu_cores: sys.cpus().len(),
        unified_memory: cfg!(all(target_os = "macos", target_arch = "aarch64")),
        gpus: collect_gpus(),
    }
}
