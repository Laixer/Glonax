use std::{fs::File, io::Read};

use crate::{runtime, CliConfig, RuntimeContext};

use super::operand::Operand;

#[derive(Debug, serde::Deserialize)]
struct Program {
    name: String,
    version: String,
    steps: Vec<Step>,
}

#[derive(Debug, serde::Deserialize)]
struct Step {
    run: String,
    parameters: Vec<f32>,
}

pub(crate) async fn exec_service<K: Operand + runtime::operand::FunctionFactory>(
    config: &CliConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut program_manager = runtime.new_program_manager();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    let mut file = File::open(&config.file).map_err(|op| runtime::error::Error::Io(op))?;

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let program: Program = serde_json::from_str(&data)
        .map_err(|op| runtime::error::Error::Generic(format!("input file malformatted: {}", op)))?;

    debug!("Parsing {} with version {}", program.name, program.version);

    if program.version != "1.0" {
        return Err(runtime::error::Error::Generic(format!(
            "input file has incompatible version"
        )));
    }

    for step in program.steps {
        program_manager
            .publish(runtime.operand.parse_function(&step.run, step.parameters))
            .await;
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}
