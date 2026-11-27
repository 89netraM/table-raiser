use std::time::Duration;

use anyhow::Result;
use serialport::{SerialPortType, UsbPortInfo};

fn main() -> Result<()> {
    let table_raiser_ports = serialport::available_ports()?
        .into_iter()
        .filter(|i| {
            matches!(
                i.port_type,
                SerialPortType::UsbPort(UsbPortInfo {
                    vid: 0xbada,
                    pid: 0x5501,
                    ..
                })
            )
        })
        .collect::<Vec<_>>();

    let [table_raiser_port] = &table_raiser_ports[..] else {
        return Ok(());
    };

    eprintln!("Found one port: {}", &table_raiser_port.port_name);

    let mut serial = serialport::new(&table_raiser_port.port_name, 9600)
        .timeout(Duration::from_secs(5))
        .open()?;
    let mut buffer = [0; 256];
    loop {
        let length = serial.read(&mut buffer)?;
        if length == 0 {
            eprintln!("End of stream");
            return Ok(());
        }
        let data = &buffer[..length];
        if let Ok(string) = str::from_utf8(&data) {
            print!("{}", string);
        } else {
            println!("{:?}", data);
        }
    }
}
