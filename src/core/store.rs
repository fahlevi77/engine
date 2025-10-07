// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt::Debug;

pub trait Store: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn Store>;
}

impl Clone for Box<dyn Store> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
