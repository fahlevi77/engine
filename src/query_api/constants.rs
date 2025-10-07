// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.util.EventFluxConstants

pub const ANNOTATION_INFO: &str = "info";
pub const ANNOTATION_ELEMENT_NAME: &str = "name";

pub const FAULT_STREAM_FLAG: &str = "!";
pub const INNER_STREAM_FLAG: &str = "#";
pub const TRIGGERED_TIME: &str = "triggered_time";

pub const LAST_INDEX: i32 = -2; // Renamed from LAST to avoid keyword clash and be more descriptive
                                // Variable.java also has a LAST = -2, which was translated to LAST_INDEX there.
                                // This makes it consistent.
