use std::{
    io::{Read, Write},
    time::Duration,
};

#[macro_use]
extern crate log;

use clap::{App, Arg};
use glonax::gloproto::{Session, Sugar};
use serial::{SerialPort, SystemPort};

/// Read the incoming packets.
///
/// Data is read from the underlaying device. Incoming packets are
/// printed to the logger. Occasionally the session statistics are
/// shown. This function will run forever.
fn read_packet<T: Read + Write>(device: T) {
    info!("Wait for initialization");
    let mut session = Session::open(device);

    info!("Initialization done");
    info!("{:?}", session.peer_info.as_ref().unwrap());

    let start = std::time::Instant::now();

    loop {
        let stats = &session.stats;
        if stats.rx_count % 100 == 0 {
            debug!(
                "Statistics: RX: {}/{} [{}%] TX: {}/{} [{}%]",
                stats.rx_count - stats.rx_failure,
                stats.rx_count,
                stats.rx_faillure_rate(),
                stats.tx_count - stats.tx_failure,
                stats.tx_count,
                stats.tx_faillure_rate(),
            );
            debug!(
                "Packets per second: {}",
                stats.rx_count / start.elapsed().as_secs() as usize
            );
        }

        if let Some(sugar) = session.next() {
            match sugar {
                Sugar::Temperature(temp) => {
                    info!("Temperature: {}", temp);
                }
                Sugar::Acceleration(x, y, z) => {
                    info!("Position: {} {} {}", x, y, z);
                }
                Sugar::Orientation(x, y, z) => {
                    info!("Orientation: {} {} {}", x, y, z);
                }
                Sugar::Direction(x, y, z) => {
                    info!("Direction: {} {} {}", x, y, z);
                }
                _ => {
                    warn!("Unknown packet");
                }
            }
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
fn serial(port: &str, baud: &str, timeout: Duration) -> serial::Result<SystemPort> {
    let mut port = serial::open(port)?;

    let baudrate = match baud {
        "19200" => {
            info!("Using 19200 baud");
            serial::Baud19200
        }
        "38400" => {
            info!("Using 38400 baud");
            serial::Baud38400
        }
        "115200" => {
            info!("Using 115200 baud");
            serial::Baud115200
        }
        "460800" => {
            info!("Using 460800 baud");
            serial::BaudOther(460800)
        }
        _ => {
            info!("Using 9600 baud");
            serial::Baud9600
        }
    };

    port.reconfigure(&|settings| {
        settings.set_baud_rate(baudrate)?;
        settings.set_parity(serial::Parity::ParityNone);
        settings.set_stop_bits(serial::StopBits::Stop1);
        settings.set_flow_control(serial::FlowControl::FlowNone);
        Ok(())
    })?;

    port.set_timeout(timeout)?;

    Ok(port)
}

fn main() {
    let matches = App::new("Glonax protodump")
        .version("0.1-alpha")
        .author("Copyright (C) 2021 Laixer Equipent B.V.")
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
            Arg::with_name("timeout")
                .short("t")
                .value_name("ms")
                .help("Timeout in ms"),
        )
        .arg(
            Arg::with_name("hex")
                .short("x")
                .long("hex")
                .help("Print contents as hexadecimal"),
        )
        .get_matches();

    let config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .build();

    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    if let Some(port) = matches.value_of("serial") {
        let baud = matches.value_of("baud").unwrap_or("9600");
        match serial(port, baud, Duration::from_millis(100)) {
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
