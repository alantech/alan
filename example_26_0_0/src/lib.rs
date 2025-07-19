use std::sync::OnceLock;

pub struct GPU {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GPU {
    pub fn list() -> Vec<wgpu::Adapter> {
        let instance = wgpu::Instance::default();
        let mut out = Vec::new();
        for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
            if adapter.get_downlevel_capabilities().is_webgpu_compliant() {
                out.push(adapter);
            }
        }
        out
    }
    pub fn init(adapters: Vec<wgpu::Adapter>) -> Vec<GPU> {
        let mut out = Vec::new();
        for adapter in adapters {
            let features = adapter.features();
            let limits = adapter.limits();
            let info = adapter.get_info();
            let device_future = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(&format!("{} on {}", info.name, info.backend.to_str())),
                    required_features: features,
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                },
            );
            match futures::executor::block_on(device_future) {
                Ok((device, queue)) => {
                    out.push(GPU {
                        adapter,
                        device,
                        queue,
                    });
                }
                Err(_) => { /* Do nothing */ }
            };
        }
        out
    }
}

static GPUS: OnceLock<Vec<GPU>> = OnceLock::new();

fn gpu() -> &'static GPU {
    match GPUS.get_or_init(|| GPU::init(GPU::list())).first() {
        Some(g) => g,
        None => panic!(
            "This program requires a GPU but there are no WebGPU-compliant GPUs on this machine"
        ),
    }
}

mod test_gpu_init {
    #[test]
    fn test_gpu_init() -> Result<(), Box<dyn std::error::Error>> {
        let _gpu = crate::gpu();
        Ok(())
    }
}