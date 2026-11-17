use sysinfo::System;

#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub brand: String,
    pub p_cores: usize,
    pub e_cores: usize,
    pub logical_threads: usize,
    pub avx2: bool,
    pub neon: bool,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: &'static str,
    pub vram_mb: u64,
    pub shared_mb: u64,
    pub vulkan_available: bool,
    pub openvino_available: bool,
}

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub cpu: CpuTopology,
    pub gpu: GpuInfo,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub ram_available_bytes: u64,
    pub recommended_model_budget_bytes: u64,
    pub cpu_usage_percent: f32,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu_all();

        let brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown x64 CPU".to_string());

        let logical_threads = sys.cpus().len().max(1);
        let (p_cores, e_cores) = detect_core_topology(logical_threads);
        let (avx2, neon) = detect_simd_caps();

        let ram_total_bytes = sys.total_memory();
        let ram_used_bytes = sys.used_memory();
        let ram_available_bytes = sys.available_memory().max(
            ram_total_bytes.saturating_sub(ram_used_bytes),
        );
        let recommended_model_budget_bytes = estimate_model_ram_budget(ram_available_bytes);

        let gpu = detect_gpu();

        Self {
            cpu: CpuTopology {
                brand,
                p_cores,
                e_cores,
                logical_threads,
                avx2,
                neon,
            },
            gpu,
            ram_used_bytes,
            ram_total_bytes,
            ram_available_bytes,
            recommended_model_budget_bytes,
            cpu_usage_percent: sys.global_cpu_usage(),
        }
    }

    pub fn refresh_live(&mut self, sys: &mut System) {
        sys.refresh_memory();
        sys.refresh_cpu_usage();
        self.ram_total_bytes = sys.total_memory();
        self.ram_used_bytes = sys.used_memory();
        self.ram_available_bytes = sys.available_memory().max(
            self.ram_total_bytes.saturating_sub(self.ram_used_bytes),
        );
        self.recommended_model_budget_bytes = estimate_model_ram_budget(self.ram_available_bytes);
        self.cpu_usage_percent = sys.global_cpu_usage();
    }
}

/// Estimates safe memory headroom available for loading a quantized GGUF model
/// after reserving ~1.25 GiB for Windows OS & desktop stability.
pub fn estimate_model_ram_budget(available_bytes: u64) -> u64 {
    const OS_RESERVE_BYTES: u64 = 1_342_177_280; // 1.25 GiB
    available_bytes.saturating_sub(OS_RESERVE_BYTES)
}

pub fn format_gb(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{gb:.1} ГБ")
}

fn detect_simd_caps() -> (bool, bool) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        (std::arch::is_x86_feature_detected!("avx2"), false)
    }
    #[cfg(target_arch = "aarch64")]
    {
        (false, true)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        (false, false)
    }
}

fn detect_core_topology(logical_threads: usize) -> (usize, usize) {
    #[cfg(windows)]
    {
        if let Some((p, e)) = query_windows_hybrid_cores() {
            return (p.max(1), e);
        }
    }
    let physical = System::physical_core_count().unwrap_or(logical_threads.max(1));
    (physical.max(1), 0)
}

#[cfg(windows)]
fn query_windows_hybrid_cores() -> Option<(usize, usize)> {
    use windows::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationProcessorCore,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let mut len: u32 = 0;
    let first = unsafe { GetLogicalProcessorInformationEx(RelationProcessorCore, None, &mut len) };
    if first.is_ok() || len == 0 {
        return None;
    }

    let mut buf = vec![0u8; len as usize];
    unsafe {
        GetLogicalProcessorInformationEx(
            RelationProcessorCore,
            Some(buf.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX),
            &mut len,
        )
        .ok()?;
    }

    let mut efficiency_classes = Vec::new();
    let mut offset = 0usize;
    while offset + std::mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>() <= len as usize {
        let info = unsafe {
            &*(buf.as_ptr().add(offset) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX)
        };
        if info.Size == 0 {
            break;
        }
        if info.Relationship == RelationProcessorCore {
            let eff = unsafe { info.Anonymous.Processor.EfficiencyClass };
            efficiency_classes.push(eff);
        }
        offset += info.Size as usize;
    }

    if efficiency_classes.is_empty() {
        return None;
    }

    let max_eff = *efficiency_classes.iter().max()?;
    let min_eff = *efficiency_classes.iter().min()?;

    if max_eff > min_eff {
        let p_cores = efficiency_classes.iter().filter(|&&e| e == max_eff).count();
        let e_cores = efficiency_classes.len().saturating_sub(p_cores);
        Some((p_cores, e_cores))
    } else {
        Some((efficiency_classes.len(), 0))
    }
}

fn detect_gpu() -> GpuInfo {
    #[cfg(windows)]
    {
        let vulkan_available = check_windows_dll("vulkan-1.dll");
        let openvino_available = check_openvino_available();

        if let Some((name, vendor, vram_mb, shared_mb)) = query_dxgi_primary_adapter() {
            return GpuInfo {
                name,
                vendor,
                vram_mb,
                shared_mb,
                vulkan_available,
                openvino_available,
            };
        }

        GpuInfo {
            name: "Integrated / Software Adapter".to_string(),
            vendor: "Generic",
            vram_mb: 0,
            shared_mb: 0,
            vulkan_available,
            openvino_available,
        }
    }

    #[cfg(not(windows))]
    {
        GpuInfo {
            name: "Host Graphics Adapter".to_string(),
            vendor: "Generic",
            vram_mb: 0,
            shared_mb: 0,
            vulkan_available: std::path::Path::new("/usr/lib/libvulkan.so.1").exists(),
            openvino_available: false,
        }
    }
}

#[cfg(windows)]
fn check_windows_dll(dll_name: &str) -> bool {
    let sys_dir = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join(dll_name);
    if sys_dir.exists() {
        return true;
    }
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| dir.join(dll_name).exists())
    })
}

#[cfg(windows)]
fn check_openvino_available() -> bool {
    if std::env::var_os("INTEL_OPENVINO_DIR").is_some() {
        return true;
    }
    check_windows_dll("openvino.dll") || check_windows_dll("openvino_c.dll")
}

#[cfg(windows)]
fn query_dxgi_primary_adapter() -> Option<(String, &'static str, u64, u64)> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

    const DXGI_SOFTWARE_FLAG: u32 = 2;

    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1().ok()? };
    let mut best: Option<(String, &'static str, u64, u64)> = None;

    for idx in 0..8 {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(idx) }) else {
            break;
        };
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if (desc.Flags & DXGI_SOFTWARE_FLAG) != 0 || desc.VendorId == 0x1414 {
            continue;
        }

        let end = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());
        let name = String::from_utf16_lossy(&desc.Description[..end])
            .trim()
            .to_string();
        let vendor = match desc.VendorId {
            0x8086 => "Intel",
            0x10DE => "NVIDIA",
            0x1002 => "AMD",
            _ => "GPU",
        };
        let vram_mb = (desc.DedicatedVideoMemory as u64) / (1024 * 1024);
        let shared_mb = (desc.SharedSystemMemory as u64) / (1024 * 1024);

        let is_better = match &best {
            None => true,
            Some((_, _, best_vram, _)) => vram_mb > *best_vram,
        };
        if is_better {
            best = Some((name, vendor, vram_mb, shared_mb));
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_memory_bytes(input: &str) -> Option<u64> {
        let trimmed = input.trim().to_ascii_uppercase();
        let split_idx = trimmed
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(trimmed.len());
        let num_part = trimmed[..split_idx].trim();
        let unit_part = trimmed[split_idx..].trim();

        let value: f64 = num_part.parse().ok()?;
        if value < 0.0 {
            return None;
        }

        let multiplier: f64 = match unit_part {
            "" | "B" => 1.0,
            "K" | "KB" | "KIB" => 1024.0,
            "M" | "MB" | "MIB" => 1024.0 * 1024.0,
            "G" | "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
            _ => return None,
        };

        Some((value * multiplier).round() as u64)
    }

    #[test]
    fn parses_memory_units_accurately() {
        assert_eq!(parse_memory_bytes("512 MB"), Some(512 * 1024 * 1024));
        assert_eq!(parse_memory_bytes("2.5GB"), Some(2_684_354_560));
        assert_eq!(parse_memory_bytes("1024 K"), Some(1024 * 1024));
        assert_eq!(parse_memory_bytes("invalid"), None);
    }

    #[test]
    fn model_ram_budget_reserves_os_headroom() {
        let eight_gb = 8 * 1024 * 1024 * 1024;
        let budget = estimate_model_ram_budget(eight_gb);
        assert!(budget < eight_gb);
        assert!(budget > 6 * 1024 * 1024 * 1024);
        assert_eq!(estimate_model_ram_budget(500 * 1024 * 1024), 0);
    }
}
