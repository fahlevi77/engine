// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::stream::output::stream_callback::StreamCallback;
use std::fmt::Debug;

pub trait Sink: StreamCallback + Debug + Send + Sync {
    fn start(&self) {}
    fn stop(&self) {}
    fn clone_box(&self) -> Box<dyn Sink>;
}

impl Clone for Box<dyn Sink> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
