// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

mod algorithm;
pub mod core;
mod device;
pub mod kernel;
pub mod net;
mod signal;

#[macro_use]
extern crate log;

mod config;
use runtime::operand::{Operand, ProgramFactory};

pub use self::config::*;

mod runtime;
pub use self::runtime::Runtime;

use kernel::excavator::Excavator;

/// Opaque runtime service for the excavator kernel. This is the recommended way
/// to instantiate a new excavator kernel on the reactor.
///
/// The excavator builder binds the excavator kernel to the hydraulic motion
/// device. The caller should tread this type as opaque.
type ExcavatorService = LaunchStub<Excavator>;

/// Start the machine kernel from configuration. This is the recommended way to
/// run a machine kernel from an dynamic external caller. Call this factory for
/// the default machine behaviour.
///
/// This factory method obtains the service from the combination of configuration
/// settings. This service is then run to completion.
pub fn runtime_program(config: &config::ProgramConfig) -> runtime::Result {
    Ok(ExcavatorService::exec_program(config)?)
}

/// Start the machine kernel from configuration. This is the recommended way to
/// run a machine kernel from an dynamic external caller. Call this factory for
/// the default machine behaviour.
///
/// This factory method obtains the service from the combination of configuration
/// settings. This service is then run to completion.
pub fn runtime_input(config: &config::InputConfig) -> runtime::Result {
    Ok(ExcavatorService::exec_input(config)?)
}

struct LaunchStub<K> {
    _1: std::marker::PhantomData<K>,
}

impl<K> LaunchStub<K>
where
    K: Operand + core::Identity + ProgramFactory,
{
    /// Create the runtime reactor.
    ///
    /// The runtime reactor takes its configuration from the global application
    /// configuration.
    ///
    /// The runtime reactor should be setup as early as possible so that all
    /// subsequent methods can run on the asynchronous reactor.
    fn runtime_reactor(config: &impl config::Configurable) -> tokio::runtime::Runtime {
        debug!(
            "Reactor runtime workers: {}",
            config.global().runtime_workers
        );

        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.global().runtime_workers)
            .enable_all()
            .thread_name("glonax-runtime-worker")
            .build()
            .unwrap()
    }

    /// Start the runtime service.
    pub fn exec_program(config: &config::ProgramConfig) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            runtime::RuntimeProgram::new(config)
                .exec_service(
                    self::runtime::Builder::<K>::from_config(config)
                        .await?
                        .enable_term_shutdown()
                        .build_with_core_service(),
                )
                .await
        })
    }

    /// Start the runtime service.
    pub fn exec_input(config: &config::InputConfig) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            runtime::RuntimeInput::new(&config)
                .exec_service(
                    self::runtime::Builder::<K>::from_config(config)
                        .await?
                        .build(),
                )
                .await
        })
    }
}
