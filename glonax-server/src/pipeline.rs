//
// The entire pipeline execution should be moved to a MCU
//

// let mut ctx = glonax::runtime::ComponentContext::default();

// world::construct(&mut ctx);

// let mut pipe = service::ComponentExecutor::new(ctx);

// pipe.add_init_component::<glonax::components::Acquisition>();

// // if config.is_simulation {
// //     pipe.add_component_default::<glonax::components::EncoderSimulator>();
// //     pipe.add_component_default::<glonax::components::EngineSimulator>();
// // }

// pipe.add_component_default::<components::Perception>();

// if config.mode == config::OperationMode::Autonomous {
//     pipe.add_component_default::<components::Planner>();
// }

// pipe.add_component_default::<components::Controller>(); // TODO: Rename to something more specific

// pipe.add_post_component::<glonax::components::CommitComponent>();
// pipe.add_post_component::<glonax::components::SignalComponent>();

// runtime.run_interval(pipe, SERVICE_PIPELINE_INTERVAL).await;
