// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

#[derive(Parser)]
#[command(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Commandline interface", long_about = None)]
struct Args {
    /// MQTT broker address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1")]
    address: String,

    /// MQTT broker port.
    #[arg(short, long, default_value_t = 1883)]
    port: u16,

    /// MQTT broker username.
    #[arg(short = 'U', long)]
    username: Option<String>,

    /// MQTT broker password.
    #[arg(short = 'P', long)]
    password: Option<String>,

    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Node commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Target node.
    Program {
        /// Program id.
        id: String,
        // #[command(subcommand)]
        // command: NodeCommand,
    },
    /// Continuously scan for network nodes.
    Scan,
    /// Show raw frames on screen.
    Dump {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Option<u32>,

        /// Filter on node.
        #[arg(long)]
        node: Option<String>,
    },
    /// Analyze network frames.
    Analyze {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Option<u32>,

        /// Filter on node.
        #[arg(long)]
        node: Option<String>,
    },
}

#[derive(clap::Subcommand)]
enum NodeCommand {
    /// Enable or disable identification LED.
    Led { toggle: String },
    /// Assign the node a new address.
    Assign { address_new: String },
    /// Reset the node.
    Reset,
    /// Enable or disable motion lock.
    Motion { toggle: String },
    /// Actuator motion.
    Actuator { actuator: u8, value: i16 },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut config = glonax::InputConfig {
        device: "fake0".to_string(),
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
    config.global.mqtt_host = args.address;
    config.global.mqtt_port = args.port;
    config.global.mqtt_username = args.username;
    config.global.mqtt_password = args.password;

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_location_level(log::LevelFilter::Off)
        .add_filter_ignore_str("sled")
        .add_filter_ignore_str("mio")
        .build();

    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    log::trace!("{:#?}", config);

    glonax::runtime_cli(&config)?;

    // let mut eventhub = glonax::EventHub::new(&config);

    // let yy = Builder::<Excavator>::from_config(config)?.build();

    // // Self::runtime_reactor(config).block_on(async {
    // //     runtime::input::exec_service(
    // //         config,
    // //         ,
    // //     )
    // //     .await
    // // })

    // let progman = glonax::core::program::ProgramManager::new(eventhub.client.clone());

    // tokio::task::spawn(async move {
    //     loop {
    //         eventhub.next().await
    //     }
    // });

    // match args.command {
    //     Command::Program { id } => {

    //         // eventhub.
    //     }
    //     Command::Scan => {
    //         // let router = Router::new(std::sync::Arc::new(net));

    //         // scan_nodes(router).await?;
    //     }
    //     Command::Dump { pgn, node } => {
    //         // let mut router = Router::new(std::sync::Arc::new(net));

    //         // if let Some(pgn) = pgn {
    //         //     router.add_pgn_filter(pgn);
    //         // }
    //         // if let Some(node) = node.map(|s| node_address(s).unwrap()) {
    //         //     router.add_node_filter(node);
    //         // }

    //         // print_frames(router).await?;
    //     }
    //     Command::Analyze { pgn, node } => {
    //         // let net = std::sync::Arc::new(net);

    //         // let mut router = Router::new(net.clone());

    //         // if let Some(pgn) = pgn {
    //         //     router.add_pgn_filter(pgn);
    //         // }
    //         // if let Some(node) = node.map(|s| node_address(s).unwrap()) {
    //         //     router.add_node_filter(node);
    //         // }

    //         // analyze_frames(net, router).await?;
    //     }
    // }

    Ok(())
}
