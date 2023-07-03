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

//! This crate provides common, functions for unit tests
use std::{
    env,
    fs::read_dir,
    io::{self, ErrorKind},
    path::PathBuf,
};

/// get the source project root path
pub fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;

    for p in path.as_path().ancestors() {
        let has_cargo = read_dir(p)?.any(|p| p.unwrap().file_name().eq("Cargo.lock"));
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}

/// get the crate root path
pub fn get_crate_root() -> io::Result<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Ok(PathBuf::from(manifest_dir))
}

#[cfg(test)]
mod tests {
    use crate::{get_crate_root, get_project_root};

    #[test]
    fn test_get_project_root() {
        let mut file_path = get_project_root().unwrap();
        file_path.push("tests/test_units/config.service.toml");

        println!("{file_path:?}");
        assert!(file_path.is_file())
    }

    #[test]
    fn test_get_crate_root() {
        let mut file_path = get_crate_root().unwrap();
        file_path.push("Cargo.toml");

        println!("{file_path:?}");
        assert!(file_path.is_file())
    }
}
