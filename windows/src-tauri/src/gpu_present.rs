// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Phase 15 · Item 3 — GPU present path.
// Stage 0: probe GPU availability and the selected backend (Decision 7:
// DX12 primary, Vulkan fallback, WARP software fallback). No surface yet.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuProbe {
    /// True when a working logical device could be created.
    pub available: bool,
    /// Selected backend, e.g. "Dx12" or "Vulkan" (or "none").
    pub backend: String,
    /// Adapter/GPU name reported by the driver.
    pub adapter_name: String,
    /// Adapter class: DiscreteGpu / IntegratedGpu / Cpu (software) / Other.
    pub device_type: String,
    /// True when the software (WARP) fallback adapter was used.
    pub is_fallback: bool,
    /// Human-readable summary for the Settings UI / logs.
    pub message: String,
}

impl GpuProbe {
    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            available: false,
            backend: "none".to_string(),
            adapter_name: String::new(),
            device_type: String::new(),
            is_fallback: false,
            message: message.into(),
        }
    }
}

/// Probes GPU availability synchronously. Tries a high-performance DX12/Vulkan
/// adapter first, then a software (WARP) fallback, then confirms a logical
/// device can actually be created. Call from a blocking context
/// (`tokio::task::spawn_blocking`); it drives wgpu's async APIs via pollster.
pub fn probe() -> GpuProbe {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
        ..Default::default()
    });

    // Primary: high-performance hardware adapter.
    let primary = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }));

    let (adapter, is_fallback) = match primary {
        Ok(adapter) => (adapter, false),
        Err(primary_err) => {
            // Fallback: software rasterizer (WARP on DX12).
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
            })) {
                Ok(adapter) => (adapter, true),
                Err(fallback_err) => {
                    let probe = GpuProbe::unavailable(format!(
                        "No usable GPU adapter (primary: {primary_err}; fallback: {fallback_err}) — using CPU preview"
                    ));
                    eprintln!("[GPU] {}", probe.message);
                    return probe;
                }
            }
        }
    };

    let info = adapter.get_info();
    let backend = format!("{:?}", info.backend);
    let device_type = format!("{:?}", info.device_type);
    let adapter_name = info.name.clone();

    let device_ok = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("clawvinci-gpu-probe"),
        ..Default::default()
    }))
    .is_ok();

    let message = if device_ok {
        format!(
            "GPU ready: {adapter_name} · {backend} · {device_type}{}",
            if is_fallback { " · software fallback" } else { "" }
        )
    } else {
        format!("Adapter {adapter_name} ({backend}) found but device creation failed — using CPU preview")
    };

    eprintln!("[GPU] {message}");

    GpuProbe {
        available: device_ok,
        backend,
        adapter_name,
        device_type,
        is_fallback,
        message,
    }
}
