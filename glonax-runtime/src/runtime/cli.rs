use std::{fs::File, io::Read};

use crate::{runtime, CliConfig, RuntimeContext};

use super::operand::Operand;

#[derive(Debug, serde::Deserialize)]
struct Program {
    name: String,
    version: String,
    commands: Vec<Command>,
}

#[derive(Debug, serde::Deserialize)]
struct Command {
    command: String,
    parameter: Option<Parameter>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Parameter {
    Vector((f32, f32, f32)),
    Preset(String),
    Duration(u32),
    Distance(f32),
}

pub(crate) async fn exec_service<K: Operand>(
    config: &CliConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut program_manager = runtime.new_program_manager();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    let mut file = File::open(&config.file).expect("cannnot open file");

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let program: Program = serde_json::from_str(&data).expect("JSON was not well-formatted");

    debug!("Parsing {} with version {}", program.name, program.version);

    for cmd in program.commands {
        match cmd.command.as_str() {
            "noop" => {
                program_manager
                    .publish(ProgramArgument {
                        id: 900,
                        parameters: vec![],
                    })
                    .await;
            }
            "test" => {
                program_manager
                    .publish(ProgramArgument {
                        id: 910,
                        parameters: vec![],
                    })
                    .await;
            }
            "sleep" => {
                if let Some(Parameter::Duration(duration)) = cmd.parameter {
                    program_manager
                        .publish(ProgramArgument {
                            id: 901,
                            parameters: vec![duration as f32],
                        })
                        .await;
                }
            }
            "drive" => {
                if let Some(Parameter::Distance(distance)) = cmd.parameter {
                    program_manager
                        .publish(ProgramArgument {
                            id: 700,
                            parameters: vec![distance],
                        })
                        .await;
                }
            }
            "position" => {
                if let Some(Parameter::Preset(name)) = cmd.parameter {
                    if name == "default" {
                        program_manager
                            .publish(ProgramArgument {
                                id: 603,
                                parameters: vec![5.21, 0.0, 0.0],
                            })
                            .await;
                    }
                } else if let Some(Parameter::Vector((x, y, z))) = cmd.parameter {
                    program_manager
                        .publish(ProgramArgument {
                            id: 603,
                            parameters: vec![x, y, z],
                        })
                        .await;
                }
            }
            _ => {}
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}
