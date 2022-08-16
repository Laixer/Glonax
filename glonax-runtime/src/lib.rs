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

pub use self::config::Config;

mod runtime;
pub use self::runtime::Runtime;

/// Start the machine kernel from configuration. This is the recommended way to
/// run a machine kernel from an dynamic external caller. Call this factory for
/// the default machine behaviour.
///
/// This factory method obtains the service from the combination of configuration
/// settings. This service is then run to completion.
pub fn start_machine(config: &Config) -> runtime::Result {
    use kernel::excavator::Excavator;
    use runtime::{CsvTracer, NullTracer};

    /// Opaque runtime service for the excavator kernel. This is the recommended way
    /// to instantiate a new excavator kernel on the reactor.
    ///
    /// The excavator builder binds the excavator kernel to the hydraulic motion
    /// device. The caller should tread this type as opaque.
    type ExcavatorService<T> = LaunchStub<Excavator, T>;

    Ok(match config {
        cnf if cnf.enable_test => ExcavatorService::<NullTracer>::test(&config)?,
        cnf if cnf.enable_trace => ExcavatorService::<CsvTracer>::launch(&config)?,
        _ => ExcavatorService::<NullTracer>::launch(&config)?,
    })
}

struct LaunchStub<K, R> {
    _1: std::marker::PhantomData<K>,
    _2: std::marker::PhantomData<R>,
}

impl<K, R> LaunchStub<K, R>
where
    K: 'static + Operand + core::Identity + ProgramFactory,
    R: core::Tracer + 'static,
    R::Instance: core::TraceWriter + Send + 'static,
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
            self::runtime::Builder::<K, R>::from_config(&config)
                .await?
                .validate()
                .await
        })
    }

    /// Start the runtime service.
    pub fn launch<'a>(config: &'a Config) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            self::runtime::Builder::<K, R>::from_config(&config)
                .await?
                .spawn()
                .await
        })
    }
}
