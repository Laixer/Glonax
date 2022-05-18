// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use std::{convert::TryInto, path::Path};

#[macro_use]
extern crate log;

use clap::{App, Arg, Command};
use glonax_ice::{eval::ContainerSession, DeviceInfo, PayloadType, Session, Vector3x16};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

/// This is our local device address.
const DEVICE_ADDR: u16 = 0x60;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Read the incoming packets.
///
/// Data is read from the underlaying device. Incoming packets are
/// printed to the logger. Occasionally the session statistics are
/// shown. This function will run forever.
async fn read_packet<T: 'static + AsyncRead + AsyncWrite + Unpin + Send + Sync>(device: T) {
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

        if let Err(e) = session.trigger_scheduler().await {
            warn!("Session fault: {:?}", e);
        }

        match session.accept().await {
            Ok(frame) => match frame.packet().payload_type.try_into().unwrap() {
                PayloadType::DeviceInfo => {
                    let dev_info: DeviceInfo = frame.get(6).unwrap();
                    info!("{:?}", dev_info);
                }
                PayloadType::MeasurementAcceleration => {
                    let acc: Vector3x16 = frame.get(6).unwrap();
                    let acc_x = acc.x;
                    let acc_y = acc.y;
                    let acc_z = acc.z;
                    info!(
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
async fn read_buffer<T: AsyncRead + Unpin>(device: &mut T) {
    let mut buf = [0; 1024];

    let mut i = 0;
    loop {
        let read_size = device.read(&mut buf).await.unwrap();
        for x in &buf[0..read_size] {
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

/// Diagnose the device.
///
/// Run the operation when the device is not performing. Any possible errors
/// or inconsistencies will be logged.
async fn diagnose<T: AsyncRead + AsyncWrite + Unpin>(device: T) {
    info!("⚠ Wait until the test is finished");

    if let Err(e) = ContainerSession::new(device).diagnose().await {
        error!("Result ➤ session fault: {:?}", e);
    } else {
        info!("Result ➤ Device is healthy");
    }
}

/// Open the serial device.
///
/// The timeout will be set to an hour which basically means
/// we will wait for the connections indefinitely.
fn serial(path: &Path, baud: usize) -> anyhow::Result<glonax_serial::Uart> {
    use glonax_serial::*;

    info!("Open {} at {} baud", path.to_str().unwrap(), baud);

    let port = glonax_serial::builder(path)?
        .set_baud_rate(BaudRate::from_speed(baud))?
        .set_parity(Parity::ParityNone)
        .set_stop_bits(StopBits::Stop1)
        .set_flow_control(FlowControl::FlowNone)
        .build()?;

    Ok(port)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new(BIN_NAME)
        .version(PKG_VERSION)
        .author("Copyright (C) 2022 Laixer Equipment B.V.")
        .about("Hardware communication diagnostics")
        .arg(
            Arg::with_name("serial")
                .short('p')
                .value_name("port")
                .help("Serial port to use (/dev/tty0)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("baud")
                .short('b')
                .help("Serial baud rate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("hex")
                .short('x')
                .long("hex")
                .help("Show the raw data buffer"),
        )
        .arg(
            Arg::with_name("diagnose")
                .short('d')
                .long("diag")
                .help("Diagnose the device"),
        )
        .arg(
            Arg::with_name("v")
                .short('v')
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .build();

    let log_level = match matches.occurrences_of("v") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 | _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    if let Some(port) = matches.value_of("serial") {
        let baud = matches.value_of("baud").unwrap_or("9600");
        let mut port = serial(Path::new(port), baud.parse()?)?;

        if matches.is_present("hex") {
            read_buffer(&mut port).await;
        } else if matches.is_present("diagnose") {
            diagnose(&mut port).await;
        } else {
            read_packet(port).await;
        }
    } else {
        println!("See help");
    }

    Ok(())
}
