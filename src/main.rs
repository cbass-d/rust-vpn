use device::TunnelDevice;
use rust_vpn::parse_packet;
use std::io::Read;

mod device;

fn main() {
    let mut config = tun::Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 255))
        .destination((10, 0, 0, 10))
        .layer(tun::Layer::L3)
        .up();

    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let mut device = TunnelDevice::new(&config);
    let mut buf = [0; 4096];

    loop {
        let amount = device.tun.read(&mut buf).unwrap();
        let data = &buf[0..amount];

        if amount > 0 {
            println!("data read: {amount}");
            parse_packet(data);
        }
    }
}
