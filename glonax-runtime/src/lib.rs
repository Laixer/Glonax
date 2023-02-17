// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

mod algorithm;
pub mod core;
pub mod device;
pub mod kernel;
pub mod net;
mod signal;

#[macro_use]
extern crate log;

mod config;
pub use runtime::operand::{FunctionFactory, Operand};

pub use self::config::*;

mod runtime;
pub use self::runtime::builder::Builder as RuntimeBuilder;
pub use self::runtime::Error;
pub use self::runtime::RuntimeContext;
