// Copyright (c) 2022 Huawei Technologies Co.,Ltd. All rights reserved.
//
// sysMaster is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! utils of libdevmaster
//!
use snafu::prelude::*;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// devmaster error
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[non_exhaustive]
pub enum Error {
    /// Error in worker manager
    #[snafu(display("Worker Manager: {}", msg))]
    WorkerManagerError {
        /// message
        msg: &'static str,
    },

    /// Error in job queue
    #[snafu(display("Job Queue: {}", msg))]
    JobQueueError {
        /// message
        msg: &'static str,
    },

    /// Error in control manager
    #[snafu(display("Control Manager: {}", msg))]
    ControlManagerError {
        /// message
        msg: &'static str,
    },

    /// Error encountered in builtin commands
    #[snafu(display("Builtin: {}", msg))]
    BuiltinCommandError {
        /// message
        msg: &'static str,
    },

    /// Error encountered in rules loader
    #[snafu(display("Failed to load rule: {}", msg))]
    RulesLoadError {
        /// message
        msg: String,
    },

    /// Error encountered in rules loader
    #[snafu(display("Failed to execute rule: {}", msg))]
    RulesExecuteError {
        /// message
        msg: String,
        /// error number
        errno: nix::errno::Errno,
    },

    /// Errors that can be ignored
    #[snafu(display("Ignore error: {}", msg))]
    IgnoreError {
        /// message
        msg: String,
    },

    /// Other errors
    #[snafu(display("Other error: {}", msg))]
    Other {
        /// error message
        msg: String,
        /// error number
        errno: nix::errno::Errno,
    },
}

impl Error {
    pub(crate) fn get_errno(&self) -> nix::errno::Errno {
        match self {
            Self::RulesExecuteError { msg: _, errno: n } => *n,
            Self::Other { msg: _, errno: n } => *n,
            _ => nix::errno::Errno::EINVAL,
        }
    }
}
