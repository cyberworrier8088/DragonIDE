#[cfg(feature = "cuda")]
pub type Gpu = burn::backend::Cuda;
#[cfg(not(feature = "cuda"))]
pub type Gpu = burn::backend::Wgpu;

#[cfg(feature = "cuda")]
pub type Device = burn::backend::cuda::CudaDevice;
#[cfg(not(feature = "cuda"))]
pub type Device = burn::backend::wgpu::WgpuDevice;

pub type TrainGpu = burn::backend::Autodiff<Gpu>;

#[cfg(feature = "cuda")]
pub fn device() -> Device {
    Default::default()
}

#[cfg(not(feature = "cuda"))]
pub fn device() -> Device {
    burn::backend::wgpu::WgpuDevice::DiscreteGpu(0)
}

#[cfg(feature = "cuda")]
pub fn device_name() -> &'static str {
    "NVIDIA CUDA"
}

#[cfg(not(feature = "cuda"))]
pub fn device_name() -> &'static str {
    "WGPU Discrete GPU"
}