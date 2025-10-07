// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod time_period;
pub mod within;

pub use self::time_period::{Duration as TimeDuration, Operator as TimeOperator, TimePeriod};
pub use self::within::Within; // Aliasing to avoid potential conflicts
