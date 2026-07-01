use std::ffi::CStr;

use ash::vk;

const NVIDIA_VENDOR_ID: u32 = 0x10DE;

pub(super) struct VulkanDevice {
    pub index: usize,
    pub vendor_id: u32,
    pub description: String,
    pub device_type: &'static str,
    pub memory_total: u64,
    pub memory_free: u64,
}

pub(super) fn devices() -> Vec<VulkanDevice> {
    match unsafe { enumerate() } {
        Ok(devices) => devices,
        Err(err) => {
            tracing::warn!("vulkan gpu enumeration unavailable: {err}");
            Vec::new()
        }
    }
}

unsafe fn enumerate() -> anyhow::Result<Vec<VulkanDevice>> {
    unsafe {
        let entry = ash::Entry::load()?;

        let instance_version = entry
            .try_enumerate_instance_version()?
            .unwrap_or(vk::API_VERSION_1_0);
        let api_version = if instance_version >= vk::API_VERSION_1_1 {
            vk::API_VERSION_1_1
        } else {
            vk::API_VERSION_1_0
        };

        let app_info = vk::ApplicationInfo::default()
            .application_name(c"sprout")
            .api_version(api_version);
        let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);
        let instance = entry.create_instance(&create_info, None)?;

        let mut out = Vec::new();
        match instance.enumerate_physical_devices() {
            Ok(physical_devices) => {
                for pd in physical_devices {
                    if let Some(device) = describe_device(&instance, pd, api_version, out.len()) {
                        out.push(device);
                    }
                }
            }
            Err(err) => tracing::warn!("vulkan enumerate_physical_devices failed: {err}"),
        }

        instance.destroy_instance(None);
        Ok(out)
    }
}

unsafe fn describe_device(
    instance: &ash::Instance,
    pd: vk::PhysicalDevice,
    api_version: u32,
    index: usize,
) -> Option<VulkanDevice> {
    unsafe {
        let props = instance.get_physical_device_properties(pd);

        if props.vendor_id == NVIDIA_VENDOR_ID {
            return None;
        }

        let device_type = match props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => "Gpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "IntegratedGpu",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Gpu",
            vk::PhysicalDeviceType::CPU => return None,
            _ => "Unknown",
        };

        let description = CStr::from_ptr(props.device_name.as_ptr())
            .to_string_lossy()
            .into_owned();
        let (memory_total, memory_free) = device_memory(instance, pd, api_version);

        Some(VulkanDevice {
            index,
            vendor_id: props.vendor_id,
            description,
            device_type,
            memory_total,
            memory_free,
        })
    }
}

unsafe fn device_memory(
    instance: &ash::Instance,
    pd: vk::PhysicalDevice,
    api_version: u32,
) -> (u64, u64) {
    unsafe {
        let mem_props = instance.get_physical_device_memory_properties(pd);
        let heap_count = mem_props.memory_heap_count as usize;

        let mut total: u64 = 0;
        let mut device_local: Vec<usize> = Vec::new();
        for (i, heap) in mem_props.memory_heaps[..heap_count].iter().enumerate() {
            if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                total = total.saturating_add(heap.size);
                device_local.push(i);
            }
        }

        if api_version >= vk::API_VERSION_1_1 && device_supports_budget(instance, pd) {
            let mut budget = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
            let mut props2 = vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
            instance.get_physical_device_memory_properties2(pd, &mut props2);

            let mut free: u64 = 0;
            let mut reported = false;
            for &i in &device_local {
                let heap_budget = budget.heap_budget[i];
                if heap_budget > 0 {
                    free = free.saturating_add(heap_budget.saturating_sub(budget.heap_usage[i]));
                    reported = true;
                }
            }
            if reported {
                return (total, free);
            }
        }

        (total, total)
    }
}

unsafe fn device_supports_budget(instance: &ash::Instance, pd: vk::PhysicalDevice) -> bool {
    unsafe {
        let Ok(exts) = instance.enumerate_device_extension_properties(pd) else {
            return false;
        };
        exts.iter()
            .any(|ext| CStr::from_ptr(ext.extension_name.as_ptr()) == ash::ext::memory_budget::NAME)
    }
}
