use std::{fs::File, io::BufRead};

use crate::{core::program::ProgramArgument, runtime, InputConfig, RuntimeContext};

use super::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    _config: &InputConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut program_manager = runtime.new_program_manager();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    let file = File::open("/home/yorick/Projects/glonax/unit1.ini").expect("cannnot open file");

    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        let line_ok = line.unwrap();
        if !line_ok.starts_with('#') && line_ok.len() > 0 {
            let row: Vec<&str> = line_ok.split_whitespace().collect();

            let argument = ProgramArgument {
                id: row[0].parse().unwrap(),
                parameters: row[1..].iter().map(|f| f.parse().unwrap()).collect(),
            };

            program_manager.publish(argument).await;
            // dbg!(argument);
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}
