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

/// Opaque runtime builder for excavator kernel.
///
/// The excavator service uses the hydraulic device to control motion.
pub type ExcavatorBuilder<'a> =
    runtime::Builder<'a, device::Hydraulic, kernel::excavator::Excavator>;
