// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod events_output_rate;
pub mod output_rate;
pub mod snapshot_output_rate;
pub mod time_output_rate; // This is the main file with the OutputRate enum/struct

pub use self::events_output_rate::EventsOutputRate;
pub use self::output_rate::{OutputRate, OutputRateBehavior, OutputRateVariant};
pub use self::snapshot_output_rate::SnapshotOutputRate;
pub use self::time_output_rate::TimeOutputRate;
