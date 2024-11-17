//! Utilities for getting system info

use std::sync::LazyLock;

use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};
use sysinfo::{Component, Components};

/// Helper struct to track gpu temperature
pub struct GpuTemp {
    maybe_device: Option<Device<'static>>,
}

impl GpuTemp {
    /// Construct a new gpu tempurature monitor, optionally selecting by device index
    pub fn new(index: u32) -> Self {
        static NVML: LazyLock<Option<Nvml>> = LazyLock::new(|| {
            let nvml = Nvml::init().ok();
            if nvml.is_none() {
                eprintln!("warning: nvml not found");
            }
            nvml
        });

        let maybe_device = NVML.as_ref().and_then(|nvml| {
            let device = nvml.device_by_index(index).ok();
            if device.is_none() {
                eprintln!("warning: device not found")
            }
            device
        });

        Self { maybe_device }
    }

    // Refresh and poll the current temperature
    pub fn get_temp(&self, farenheit: bool) -> Option<u8> {
        self.maybe_device
            .as_ref()
            .and_then(|d| d.temperature(TemperatureSensor::Gpu).ok())
            .map(|v| {
                if farenheit {
                    (v as f64 * 9. / 5. + 32.) as u8
                } else {
                    v as u8
                }
            })
    }
}

pub struct CpuTemp {
    maybe_cpu: Option<Component>,
}

impl CpuTemp {
    // Create a new cpu temp monitor, optionally selecting the component by a label search string
    pub fn new(search_label: &str) -> Self {
        let comps: Vec<_> = Components::new_with_refreshed_list().into();
        let maybe_cpu = comps.into_iter().find(|v| v.label().contains(search_label));
        if maybe_cpu.is_none() {
            eprintln!("warning: could not find coretemp package")
        }
        Self { maybe_cpu }
    }

    // Refresh and poll the current temperature
    pub fn get_temp(&mut self, farenheit: bool) -> Option<u8> {
        self.maybe_cpu.as_mut().map(|cpu| {
            cpu.refresh();
            let mut temp = cpu.temperature();
            if farenheit {
                temp = temp * 9. / 5. + 32.;
            }
            temp as u8
        })
    }
}
