// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

mod device;
pub mod kernel;
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
pub type ExcavatorService = LaunchStub<device::Hydraulic, kernel::excavator::Excavator>;

/// Opaque runtime service for the frozed excavator kernel. The frozen exacator service
/// act similair to the exacator service but without any commanded movements. This
/// effectively renders the kernel immobile.
///
/// The excavator builder binds the excavator kernel to the sink device. The caller should
/// tread this type as opaque.
pub type FrozenExcavatorService = LaunchStub<device::Sink, kernel::excavator::Excavator>;

pub struct LaunchStub<M, K> {
    _1: std::marker::PhantomData<M>,
    _2: std::marker::PhantomData<K>,
}

impl<M, K> LaunchStub<M, K>
where
    M: 'static + device::IoDevice + device::MotionDevice + Send,
    M::DeviceProfile: device::IoDeviceProfile,
    K: 'static + runtime::operand::Operand + glonax_core::Identity,
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
            self::runtime::Builder::<M, K>::from_config(&config)
                .await?
                .validate()
                .await
        })
    }

    /// Start the runtime service.
    pub fn launch<'a>(config: &'a Config) -> runtime::Result {
        Self::runtime_reactor(config).block_on(async {
            self::runtime::Builder::<M, K>::from_config(&config)
                .await?
                .spawn()
                .await
        })
    }
}
