// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{App, Arg};
use glonax::runtime::{Runtime, RuntimeSettings};

#[allow(dead_code)]
const SERIAL_HYDRAU1: &str = "/dev/ttyUSB0";
// #[allow(dead_code)]st
// const SERIAL_HYDRAU2: &str = "/dev/ttyUSB1";
#[allow(dead_code)]
const SERIAL_INTERTIAL1: &str = "/dev/ttyUSB0";
// #[allow(dead_code)]
// const SERIAL_INTERTIAL2: &str = "/dev/ttyUSB1";

// TODO: Should not return serial error.
async fn run(config: glonax::Config) -> glonax::device::Result<()> {
    use glonax::device::Device;
    use glonax::device::{Composer, Gamepad, Hydraulic, Inertial};

    // Motion.

    let mut hydraulic_motion = Hydraulic::new(SERIAL_HYDRAU1)?;
    log::info!("Name: {}", hydraulic_motion.name());
    hydraulic_motion.probe();
    // let mut hydraulic_motion2 = Hydraulic::new(SERIAL_HYDRAU2)?;
    // log::info!("Name: {}", hydraulic_motion2.name());
    // hydraulic_motion2.probe();

    // let mut hydraulic_compose = Composer::with_index(0);
    // log::info!("Name: {}", hydraulic_compose.name());
    // hydraulic_compose.insert(hydraulic_motion);
    // hydraulic_compose.probe();

    // TODO: Runtime builder.

    let mut rt = Runtime {
        motion_device: hydraulic_motion,
        actuator_map: None,
        event_bus: tokio::sync::mpsc::channel(128),
        settings: RuntimeSettings::from(&config),
        task_pool: vec![],
    };

    let dispatcher = rt.dispatch();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        log::info!("Termination requested");
        dispatcher.gracefull_shutdown().await.unwrap();
    });

    if config.enable_autopilot {
        let mut imu = Inertial::new(SERIAL_INTERTIAL1)?;
        log::info!("Name: {}", imu.name());
        imu.probe();

        // let mut imu2 = Inertial::new(SERIAL_INTERTIAL2)?;
        // log::info!("Name: {}", imu2.name());
        // imu2.probe();

        let mut measure_compose =
            Composer::<Box<dyn glonax::device::MetricDevice + Send + Sync>>::new();
        log::info!("Name: {}", measure_compose.name());
        measure_compose.insert(Box::new(imu));
        // measure_compose.insert(Box::new(imu2));
        measure_compose.probe();

        rt.spawn_program_queue(
            measure_compose,
            glonax::kernel::machine::ArmBalanceProgram::new(),
        );
    }

    if config.enable_command {
        let mut gamepad = Gamepad::new();
        log::info!("Name: {}", gamepad.name());
        gamepad.probe();

        rt.spawn_command_device(gamepad);
    }

    //
    // Start the runtime.
    //

    rt.run().await;

    // TODO: This should really be an error because we dont expect to return.
    Ok(())
}

#[tokio::main]
async fn main() {
    let matches = App::new("Glonax daemon")
        .version("0.3.1")
        .author("Copyright (C) 2021 Laixer Equipent B.V.")
        .about("Heavy machinery controller daemon")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Configuration file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("listen")
                .short("l")
                .value_name("address:port")
                .help("Network address to bind")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("record")
                .short("R")
                .long("record")
                .help("Record log to disk"),
        )
        .arg(
            Arg::with_name("no-auto")
                .long("no-auto")
                .help("Disable autopilot program"),
        )
        .arg(
            Arg::with_name("no-gamepad")
                .long("no-gamepad")
                .help("Disable gamepad controls"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let mut config = glonax::Config::default();

    if matches.is_present("no-auto") {
        config.enable_autopilot = false;
    }
    if matches.is_present("no-gamepad") {
        config.enable_command = false;
    }

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .set_time_format("%X %6f".to_owned())
        .set_target_level(log::LevelFilter::Trace)
        .build();

    let log_level = match matches.occurrences_of("v") {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        2 | _ => log::LevelFilter::Debug,
    };

    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            log_level,
            log_config.clone(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(
            log_level,
            log_config.clone(),
            std::fs::File::create(format!("log/manual_{}.log", std::process::id())).unwrap(),
        ),
    ])
    .unwrap();

    // NOTE: We'll never reach beyond this point on success.
    if let Err(e) = run(config).await {
        log::error!("{}", e);
    }
}
