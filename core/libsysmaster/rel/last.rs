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

use crate::error::*;
use heed::types::{OwnedType, SerdeBincode, Str};
use heed::{Database, Env, EnvOpenOptions};
use std::cell::RefCell;
use std::path::Path;
use std::{fmt, fs};

const RELI_LAST_DIR: &str = "last.mdb";
const RELI_LAST_MAX_DBS: u32 = 2;
const RELI_DB_LUNIT: &str = "unit";
const RELI_DB_LFRAME: &str = "frame";
#[allow(dead_code)]
static RELI_LAST_DB_NAME: [&str; RELI_LAST_MAX_DBS as usize] = [RELI_DB_LUNIT, RELI_DB_LFRAME];
const RELI_LAST_KEY: u32 = 0; // singleton

pub struct ReliLast {
    // control
    ignore: RefCell<bool>,

    // data
    /* environment */
    env: Env,

    /* database: singleton(1) */
    unit: Database<OwnedType<u32>, Str>, // RELI_DB_LUNIT; key: RELI_LAST_KEY, data: unit_id;
    frame: Database<OwnedType<u32>, SerdeBincode<Vec<ReliFrame>>>, // RELI_DB_LFRAME; key: RELI_LAST_KEY, data: vec<f1+f2+f3>;
}

impl fmt::Debug for ReliLast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReliLast")
            .field("env.path", &self.env.path())
            .field("unit.len", &self.unit_len().unwrap_or(0))
            .field("frame.len", &self.frame_len().unwrap_or(0))
            .finish()
    }
}

impl ReliLast {
    pub fn new(dir_str: &str) -> ReliLast {
        // init environment
        let path = Path::new(dir_str).join(RELI_LAST_DIR);
        let env = EnvOpenOptions::new()
            .max_dbs(RELI_LAST_MAX_DBS)
            .open(path)
            .unwrap();

        // create db
        let unit = env.create_database(Some(RELI_DB_LUNIT)).unwrap();
        let frame = env.create_database(Some(RELI_DB_LFRAME)).unwrap();

        // return
        ReliLast {
            ignore: RefCell::new(false),
            env,
            unit,
            frame,
        }
    }

    pub fn data_clear(&self) {
        let mut wtxn = self.env.write_txn().expect("last.write_txn");
        self.unit.clear(&mut wtxn).expect("clear.put");
        self.frame.clear(&mut wtxn).expect("clear.put");
        wtxn.commit().expect("last.commit");
    }

    pub fn ignore_set(&self, ignore: bool) {
        *self.ignore.borrow_mut() = ignore;
    }

    pub fn set_unit(&self, unit_id: &str) {
        if self.ignore() {
            return;
        }

        let mut wtxn = self.env.write_txn().expect("last.write_txn");
        self.unit
            .put(&mut wtxn, &RELI_LAST_KEY, unit_id)
            .expect("last.put");
        wtxn.commit().expect("last.commit");
    }

    pub fn clear_unit(&self) {
        if self.ignore() {
            return;
        }

        let mut wtxn = self.env.write_txn().expect("last.write_txn");
        self.unit
            .delete(&mut wtxn, &RELI_LAST_KEY)
            .expect("last.delete");
        wtxn.commit().expect("last.commit");
    }

    pub fn set_frame(&self, f1: u32, f2: Option<u32>, f3: Option<u32>) {
        if self.ignore() {
            return;
        }

        let mut wtxn = self.env.write_txn().expect("last.write_txn");
        let mut frame = match self.frame.get(&wtxn, &RELI_LAST_KEY).unwrap_or(None) {
            Some(f) => f,
            None => Vec::new(),
        };
        frame.push((f1, f2, f3));
        self.frame
            .put(&mut wtxn, &RELI_LAST_KEY, &frame)
            .expect("last.put");
        wtxn.commit().expect("last.commit");
    }

    pub fn clear_frame(&self) {
        if self.ignore() {
            return;
        }

        let mut wtxn = self.env.write_txn().expect("last.write_txn");
        let mut frame = match self.frame.get(&wtxn, &RELI_LAST_KEY).unwrap_or(None) {
            Some(f) => f,
            None => Vec::new(),
        };
        frame.pop();
        self.frame
            .put(&mut wtxn, &RELI_LAST_KEY, &frame)
            .expect("last.put");
        wtxn.commit().expect("last.commit");
    }

    pub fn unit(&self) -> Option<String> {
        let rtxn = self.env.read_txn().expect("last.read_txn");
        let unit_id = self.unit.get(&rtxn, &RELI_LAST_KEY).unwrap_or(None);
        unit_id.map(|u| u.to_string())
    }

    pub fn frame(&self) -> Option<ReliFrame> {
        let rtxn = self.env.read_txn().expect("last.read_txn");
        let frame = self.frame.get(&rtxn, &RELI_LAST_KEY).unwrap_or(None);
        match frame {
            Some(mut f) => f.pop(),
            None => None,
        }
    }

    pub fn ignore(&self) -> bool {
        *self.ignore.borrow()
    }

    fn unit_len(&self) -> Result<u64> {
        let rtxn = self.env.read_txn().context(HeedSnafu)?;
        self.unit.len(&rtxn).context(HeedSnafu)
    }

    fn frame_len(&self) -> Result<u64> {
        let rtxn = self.env.read_txn().context(HeedSnafu)?;
        let frame = self.frame.get(&rtxn, &RELI_LAST_KEY).unwrap_or(None);
        let len = match frame {
            Some(f) => f.len(),
            None => 0,
        };
        Ok(len as u64)
    }
}

pub fn prepare(dir_str: &str) -> Result<()> {
    let last = Path::new(dir_str).join(RELI_LAST_DIR);
    if !last.exists() {
        fs::create_dir_all(&last).context(IoSnafu)?;
    }

    Ok(())
}

type ReliFrame = (u32, Option<u32>, Option<u32>);
