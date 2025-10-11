use tun::{Configuration, Device};

pub struct TunnelDevice {
    pub tun: Device,
}

impl TunnelDevice {
    pub fn new(config: &Configuration) -> Self {
        let device = tun::create(config).unwrap();

        Self { tun: device }
    }
}
