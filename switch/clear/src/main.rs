//! wow-nx render step 2: wgpu's instance and adapter on the Switch, through the deko3d backend in the GL slot.

extern crate benilla_horizon;

fn main() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()));
    println!("adapter: {:?}", adapter.map(|a| a.get_info()));
}
