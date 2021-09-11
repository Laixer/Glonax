use std::{
    convert::TryInto,
    io::{Read, Write},
    time::Duration,
};

#[macro_use]
extern crate log;

use clap::{App, Arg};
use glonax_ice::{DeviceInfo, PayloadType, Session, Vector3x16};
use serial::{SerialPort, SystemPort};

/// This is our local device address.
const DEVICE_ADDR: u16 = 0x21;

/// Read the incoming packets.
///
/// Data is read from the underlaying device. Incoming packets are
/// printed to the logger. Occasionally the session statistics are
/// shown. This function will run forever.
fn read_packet<T: Read + Write>(device: T) {
    info!("Reading packets...");

    let mut session = Session::new(device, DEVICE_ADDR);

    let start = std::time::Instant::now();

    loop {
        let stats = &session.stats;
        if stats.rx_count % 250 == 0 && stats.rx_count > 0 && start.elapsed().as_secs() > 0 {
            info!(
                "Statistics: RX: {}/{} [{:.1}%] TX: {}/{} [{:.1}%]",
                stats.rx_count - stats.rx_failure,
                stats.rx_count,
                stats.rx_failure_rate(),
                stats.tx_count - stats.tx_failure,
                stats.tx_count,
                stats.tx_failure_rate(),
            );
            info!(
                "Average packets/s: {}",
                stats.rx_count / start.elapsed().as_secs() as usize
            );
        }

        if stats.rx_count % 50 == 0 {
            if let Err(err) = session.announce_device() {
                error!("Session error: {:?}", err);
            }
        }

        match session.accept() {
            Ok(frame) => match frame.packet().payload_type.try_into().unwrap() {
                PayloadType::DeviceInfo => {
                    let dev_info: DeviceInfo = frame.get(6).unwrap();
                    debug!("{:?}", dev_info);
                }
                PayloadType::MeasurementAcceleration => {
                    let acc: Vector3x16 = frame.get(6).unwrap();
                    let acc_x = acc.x;
                    let acc_y = acc.y;
                    let acc_z = acc.z;
                    debug!(
                        "Acceleration: X: {:>+5} Y: {:>+5} Z: {:>+5}",
                        acc_x, acc_y, acc_z
                    );
                }
                _ => {}
            },
            Err(e) => warn!("Session fault: {:?}", e),
        }
    }
}

/// Show the raw data buffer on screen.
fn read_buffer<T: Read>(device: &mut T) {
    let mut buf = [0; 1024];

    let mut i = 0;
    loop {
        let read_sz = device.read(&mut buf).unwrap();
        for x in &buf[0..read_sz] {
            print!("{:02x?} ", x);
            i += 1;
            if i % 128 == 0 {
                print!("\n\n{:06} | ", i);
            } else if i % 16 == 0 {
                print!("\n{:06} | ", i);
            } else if i % 8 == 0 {
                print!(" ");
            }
        }
    }
}

/// Open the serial device.
///
/// The timeout will be set to an hour which basically means
/// we will wait for the connections indefinitely.
fn serial(port: &str, baud: usize) -> serial::Result<SystemPort> {
    info!("Open {} at {} baud", port, baud);

    let mut port = serial::open(port)?;

    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::BaudRate::from_speed(baud))?;
        settings.set_parity(serial::Parity::ParityNone);
        settings.set_stop_bits(serial::StopBits::Stop1);
        settings.set_flow_control(serial::FlowControl::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_secs(3600))?;

    Ok(port)
}

fn main() {
    let matches = App::new("Glonax icedump")
        .version("0.1-alpha")
        .author("Copyright (C) 2021 Laixer Equipment B.V.")
        .about("Comminication diagnostics tool")
        .arg(
            Arg::with_name("serial")
                .short("p")
                .value_name("port")
                .help("Serial port to use (/dev/tty0)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("baud")
                .short("b")
                .help("Serial baud rate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("hex")
                .short("x")
                .long("hex")
                .help("Print contents as hexadecimal"),
        )
        .get_matches();

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .build();

    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    if let Some(port) = matches.value_of("serial") {
        let baud = matches.value_of("baud").unwrap_or("9600");
        match serial(port, baud.parse().expect("Invalid baud rate")) {
            Ok(mut port) => {
                if matches.is_present("hex") {
                    read_buffer(&mut port);
                } else {
                    read_packet(port);
                }
            }
            Err(e) => error!("{}", e),
        }
    } else {
        println!("{}", matches.usage());
    }
}
