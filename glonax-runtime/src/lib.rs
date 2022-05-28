// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

mod device;
pub mod kernel;
pub mod net;
mod workspace;

#[macro_use]
extern crate log;

mod config;
pub use self::config::Config;

mod runtime;
pub use self::runtime::Runtime;

/// Opaque runtime service for the excavator kernel. This is the recommended way
/// to instantiate a new excavator kernel on the reactor.
///
/// The excavator builder binds the excavator kernel to the hydraulic motion
/// device. The caller should tread this type as opaque.
type ExcavatorService =
    LaunchStub<device::Hydraulic, kernel::excavator::Excavator, runtime::NullTracer>;

/// Start the machine kernel from configuration. This is the recommended way to
/// run a machine kernel from an dynamic external caller. Call this factory for
/// the default machine behaviour.
///
/// This factory method obtains the service from the combination of configuration
/// settings. This service is then run to completion.
pub fn start_machine(config: &Config) -> runtime::Result {
    use device::{Hydraulic, Sink};
    use kernel::excavator::Excavator;
    use runtime::{CsvTracer, NullTracer};

    Ok(match config {
        // cnf if !cnf.enable_motion && cnf.enable_test => {
        //     LaunchStub::<Sink, Excavator, NullTracer>::test(&config)?
        // }
        // cnf if !cnf.enable_motion && cnf.enable_trace => {
        //     LaunchStub::<Sink, Excavator, CsvTracer>::launch(&config)?
        // }
        // cnf if !cnf.enable_motion => LaunchStub::<Sink, Excavator, NullTracer>::launch(&config)?,
        // cnf if cnf.enable_test => ExcavatorService::test(&config)?,
        // cnf if cnf.enable_trace => LaunchStub::<Hydraulic, Excavator, CsvTracer>::launch(&config)?,
        // _ => ExcavatorService::launch(&config)?,
        _ => LaunchStub::<device::ControlAreaUnit, Excavator, NullTracer>::launch(&config)?,
    })
}

struct LaunchStub<M, K, R> {
    _1: std::marker::PhantomData<M>,
    _2: std::marker::PhantomData<K>,
    _3: std::marker::PhantomData<R>,
}

impl<M, K, R> LaunchStub<M, K, R>
where
    M: 'static + device::MotionDevice + Default + Send,
    // M::DeviceProfile: device::IoDeviceProfile,
    K: 'static + runtime::operand::Operand + glonax_core::Identity,
    R: glonax_core::Tracer,
    R::Instance: glonax_core::TraceWriter + Send + 'static,
{
    /// Create the runtime reactor.
    ///
    /// The runtime reactor takes its configuration from the global application
    /// configuration.
    ///
    /// The runtime reactor should be setup as early as possible so that all
    /// subsequent methods can run on the asynchronous reactor.
    fn runtime_reactor(config: &Config) -> tokio::runtime::Runtime {
        debug!("Reactor runtime workers: {}", config.runtime_workers);

        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.runtime_workers)
            .enable_all()
            .thread_name("glonax-runtime-worker")
            .build()
            .unwrap()
    }

    /// Test the runtime service, then return.
    pub fn test<'a>(config: &'a Config) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            self::runtime::Builder::<M, K, R>::from_config(&config)
                .await?
                .validate()
                .await
        })
    }

    /// Start the runtime service.
    pub fn launch<'a>(config: &'a Config) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            self::runtime::Builder::<M, K, R>::from_config(&config)
                .await?
                .spawn()
                .await
        })
    }
}
