use sysinfo::SystemExt;
use anyhow::{anyhow, Result};
use std::fmt;
use std::process::Command;
use sysinfo::System;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    name: String,
    cpu_cores: usize,
    memory_gb: f64,
    has_gpu: bool,
    has_fp16: bool,
    compute_tier: ComputeTier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputeTier {
    Minimal, // Raspberry Pi, embedded devices
    Low,     // Low-end desktop/laptop
    Medium,  // Modern desktop/laptop
    High,    // Workstation
    Extreme, // Server/H100
}

impl HardwareProfile {
    /// Detect current hardware capabilities
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_cores = sys.cpus().len();
        let memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

        // Real GPU detection across platforms
        let has_gpu = Self::detect_gpu();
        let has_fp16 = Self::detect_fp16();

        let compute_tier = Self::classify_tier(cpu_cores, memory_gb);
        let name = compute_tier.default_name();

        Self {
            name,
            cpu_cores,
            memory_gb,
            has_gpu,
            has_fp16,
            compute_tier,
        }
    }

    /// Detect GPU availability across different platforms
    fn detect_gpu() -> bool {
        // Try CUDA (NVIDIA)
        if Self::has_cuda() {
            debug!("CUDA GPU detected");
            return true;
        }

        // Try ROCm (AMD on Linux)
        if Self::has_rocm() {
            debug!("ROCm GPU detected");
            return true;
        }

        // Try Metal (Apple Silicon)
        if Self::has_metal() {
            debug!("Metal GPU detected");
            return true;
        }

        // Try DirectML (Windows)
        if Self::has_directml() {
            debug!("DirectML GPU detected");
            return true;
        }

        debug!("No GPU detected");
        false
    }

    fn has_cuda() -> bool {
        // Check for nvidia-smi command
        if let Ok(output) = Command::new("nvidia-smi").output() {
            return output.status.success();
        }

        // Check for CUDA libraries
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/local/cuda").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcuda.so").exists()
        }
        #[cfg(target_os = "windows")]
        {
            std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA").exists()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    fn has_rocm() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check for rocm-smi command
            if let Ok(output) = Command::new("rocm-smi").output() {
                return output.status.success();
            }

            // Check for ROCm installation
            std::path::Path::new("/opt/rocm").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn has_metal() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check for Metal via system_profiler
            if let Ok(output) = Command::new("system_profiler")
                .arg("SPDisplaysDataType")
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    return output_str.contains("Metal");
                }
            }

            // On Apple Silicon, Metal is always available
            if let Ok(output) = Command::new("sysctl")
                .arg("-n")
                .arg("machdep.cpu.brand_string")
                .output()
            {
                let brand = String::from_utf8_lossy(&output.stdout);
                return brand.contains("Apple");
            }

            false
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn has_directml() -> bool {
        #[cfg(target_os = "windows")]
        {
            // DirectML is available on Windows 10 1709+ with any GPU
            // Check for dxdiag or common GPU drivers
            if let Ok(output) = Command::new("wmic")
                .args(["path", "win32_VideoController", "get", "name"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    return output_str.lines().count() > 1; // Has GPU entries
                }
            }
            false
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    /// Detect FP16 (half precision) support
    fn detect_fp16() -> bool {
        // FP16 is supported on:
        // 1. Modern NVIDIA GPUs (Compute Capability >= 6.0)
        // 2. Apple Silicon (M1/M2/M3)
        // 3. AMD GPUs with FP16 support
        // 4. Modern CPUs with AVX2 (emulated, slower)

        // Check for Apple Silicon
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .arg("-n")
                .arg("machdep.cpu.brand_string")
                .output()
            {
                let brand = String::from_utf8_lossy(&output.stdout);
                if brand.contains("Apple") {
                    debug!("FP16 supported on Apple Silicon");
                    return true;
                }
            }
        }

        // Check for NVIDIA GPU with nvidia-smi
        if let Ok(output) = Command::new("nvidia-smi")
            .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
            .output()
        {
            if output.status.success() {
                let cap_str = String::from_utf8_lossy(&output.stdout);
                if let Some(cap_line) = cap_str.lines().next() {
                    if let Ok(cap) = cap_line.trim().parse::<f32>() {
                        if cap >= 6.0 {
                            debug!("FP16 supported on NVIDIA GPU (compute capability {:.1})", cap);
                            return true;
                        }
                    }
                }
            }
        }

        // Check for AMD ROCm
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/opt/rocm").exists() {
                debug!("FP16 likely supported on AMD ROCm");
                return true;
            }
        }

        // Fallback: Check for AVX2 (CPU can emulate FP16, but slower)
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                debug!("FP16 emulation possible with AVX2");
                return true;
            }
        }

        debug!("No FP16 support detected");
        false
    }

    /// Create a profile from a name
    pub fn from_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "auto" => Ok(Self::detect()),
            "raspberry-pi" => Ok(Self {
                name: "raspberry-pi".to_string(),
                cpu_cores: 4,
                memory_gb: 4.0,
                has_gpu: false,
                has_fp16: false,
                compute_tier: ComputeTier::Minimal,
            }),
            "desktop" => Ok(Self {
                name: "desktop".to_string(),
                cpu_cores: 8,
                memory_gb: 16.0,
                has_gpu: true,
                has_fp16: true,
                compute_tier: ComputeTier::Medium,
            }),
            "server" => Ok(Self {
                name: "server".to_string(),
                cpu_cores: 32,
                memory_gb: 64.0,
                has_gpu: false,
                has_fp16: false,
                compute_tier: ComputeTier::High,
            }),
            "h100" => Ok(Self {
                name: "h100".to_string(),
                cpu_cores: 64,
                memory_gb: 128.0,
                has_gpu: true,
                has_fp16: true,
                compute_tier: ComputeTier::Extreme,
            }),
            _ => Err(anyhow!("Unknown hardware profile: {}", name)),
        }
    }

    fn classify_tier(cpu_cores: usize, memory_gb: f64) -> ComputeTier {
        if cpu_cores <= 4 && memory_gb <= 8.0 {
            ComputeTier::Minimal
        } else if cpu_cores <= 8 && memory_gb <= 16.0 {
            ComputeTier::Low
        } else if cpu_cores <= 16 && memory_gb <= 32.0 {
            ComputeTier::Medium
        } else if cpu_cores <= 32 && memory_gb <= 64.0 {
            ComputeTier::High
        } else {
            ComputeTier::Extreme
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn cpu_cores(&self) -> usize {
        self.cpu_cores
    }

    #[allow(dead_code)]
    pub fn memory_gb(&self) -> f64 {
        self.memory_gb
    }

    #[allow(dead_code)]
    pub fn has_gpu(&self) -> bool {
        self.has_gpu
    }

    #[allow(dead_code)]
    pub fn has_fp16(&self) -> bool {
        self.has_fp16
    }

    #[allow(dead_code)]
    pub fn compute_tier(&self) -> &ComputeTier {
        &self.compute_tier
    }
}

impl ComputeTier {
    fn default_name(&self) -> String {
        match self {
            ComputeTier::Minimal => "minimal-device",
            ComputeTier::Low => "low-end-system",
            ComputeTier::Medium => "medium-system",
            ComputeTier::High => "high-end-system",
            ComputeTier::Extreme => "extreme-system",
        }
        .to_string()
    }
}

impl fmt::Display for HardwareProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hardware Profile: {}\n  CPU Cores: {}\n  Memory: {:.2} GB\n  GPU: {}\n  FP16: {}\n  Compute Tier: {:?}",
            self.name, self.cpu_cores, self.memory_gb,
            if self.has_gpu { "Yes" } else { "No" },
            if self.has_fp16 { "Yes" } else { "No" },
            self.compute_tier
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let profile = HardwareProfile::detect();
        assert!(profile.cpu_cores() > 0);
        assert!(profile.memory_gb() > 0.0);
    }

    #[test]
    fn test_from_name() {
        let profile = HardwareProfile::from_name("raspberry-pi").unwrap();
        assert_eq!(profile.cpu_cores(), 4);
        assert_eq!(profile.compute_tier(), &ComputeTier::Minimal);
    }
}
