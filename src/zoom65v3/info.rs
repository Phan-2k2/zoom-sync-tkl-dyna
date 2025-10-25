//! Utilities for getting system info

use std::error::Error;
use std::sync::LazyLock;

use bpaf::Bpaf;
use either::Either;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::{Device, Nvml};
use sysinfo::{Component, Components};
use zoom65v3::Zoom65v3;

#[derive(Clone, Debug, bpaf::Bpaf)]
pub enum CpuMode {
    Label(
        /// Sensor label to search for
        #[bpaf(long("cpu"), argument("LABEL"), fallback("coretemp Package".into()), display_fallback)]
        String,
    ),
    Manual(
        /// Manually set CPU temperature
        #[bpaf(short('c'), long("cpu-temp"), argument("TEMP"))]
        u8,
    ),
}

impl CpuMode {
    pub fn either(&self) -> Either<CpuTemp, u8> {
        match self {
            CpuMode::Label(label) => Either::Left(CpuTemp::new(label)),
            CpuMode::Manual(v) => Either::Right(*v),
        }
    }
}

#[derive(Clone, Debug, bpaf::Bpaf)]
pub enum GpuMode {
    Id(
        /// GPU device id to fetch temperature data for (nvidia only)
        #[bpaf(long("gpu"), argument::<u32>("ID"), fallback(0), display_fallback)]
        u32,
    ),
    Manual(
        /// Manually set GPU temperature
        #[bpaf(short('g'), long("gpu-temp"), argument("TEMP"))]
        u8,
    ),
}

impl GpuMode {
    pub fn either(&self) -> Either<GpuTemp, u8> {
        match self {
            GpuMode::Id(i) => Either::Left(GpuTemp::new(*i)),
            GpuMode::Manual(v) => Either::Right(*v),
        }
    }
}

/// System info options:
#[derive(Clone, Debug, Bpaf)]
pub enum SystemArgs {
    /// Disable updating system info completely
    #[bpaf(long("no-system"))]
    Disabled,
    Enabled {
        #[bpaf(external)]
        cpu_mode: CpuMode,
        #[bpaf(external)]
        gpu_mode: GpuMode,
        /// Manually set download speed
        #[bpaf(short, long)]
        download: Option<f32>,
    },
}

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
            match cpu.temperature() {
                Some(mut temp) => {
                    if farenheit {
                        temp = temp * 9. / 5. + 32.;
                    }
                    temp as u8
                },
                None => 0,
            }
        })
    }
}

pub fn apply_system(
    keyboard: &mut Zoom65v3,
    farenheit: bool,
    cpu: &mut Either<CpuTemp, u8>,
    gpu: &Either<GpuTemp, u8>,
    download: Option<f32>,
) -> Result<(), Box<dyn Error>> {
    let mut cpu_temp = cpu
        .as_mut()
        .map_left(|c| c.get_temp(farenheit).unwrap_or_default())
        .map_right(|v| *v)
        .into_inner();
    if cpu_temp >= 100 {
        eprintln!("warning: actual cpu temperature at {cpu_temp}, clamping to 99");
        cpu_temp = 99;
    }

    let mut gpu_temp = gpu
        .as_ref()
        .map_left(|g| g.get_temp(farenheit).unwrap_or_default())
        .map_right(|v| *v)
        .into_inner();
    if gpu_temp >= 100 {
        eprintln!("warning: actual gpu temerature at {gpu_temp}. clamping to 99");
        gpu_temp = 99;
    }

    let download = download.unwrap_or_default();

    keyboard
        .set_system_info(cpu_temp, gpu_temp, download)
        .map_err(|e| format!("failed to set system info: {e}"))?;
    println!(
        "updated system info {{ cpu_temp: {cpu_temp}, gpu_temp: {gpu_temp}, download: {download} }}"
    );

    Ok(())
}
