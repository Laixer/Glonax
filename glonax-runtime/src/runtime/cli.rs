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

    let (models, _) = tobj::load_obj(
        "/home/yorick/Projects/glonax/path.obj",
        &tobj::LoadOptions::default(),
    )
    .expect("ouch");

    for model in &models {
        let mesh = &model.mesh;

        for v in 0..mesh.positions.len() / 3 {
            let x = mesh.positions[3 * v];
            let y = mesh.positions[3 * v + 1];
            let z = mesh.positions[3 * v + 2];

            let yy = ProgramArgument {
                id: 603,
                parameters: vec![x, y, z],
            };

            program_manager.publish(yy).await;
        }
    }

    // dbg!(models);

    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![5.21, 0.0, 0.0],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![3.77, 1.10, 4.07],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![5.28, 1.32, 5.70],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![3.16, -0.45, 3.41],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![3.16, 0.55, 3.41],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![4.45, 0.55, -0.33],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![6.73, 2.35, -4.27],
    //     })
    //     .await;
    // program_manager
    //     .publish(ProgramArgument {
    //         id: 603,
    //         parameters: vec![5.21, 0.0, 0.0],
    //     })
    //     .await;

    // info!("Program 901 submitted");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}
