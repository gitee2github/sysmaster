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

//! SocketUnit is the entrance of the sub unit，implement the trait UnitObj,UnitMngUtil and UnitSubClass.
//! Trait UnitObj defines the behavior of the sub unit.
//! Trait UnitMngUtil is used to attach the Unitmanager to the sub unit.
//! Trait UnitSubClass implement the convert from sub unit to UnitObj.

use crate::{
    base::PLUGIN_NAME, comm::SocketUnitComm, config::SocketConfig, load::SocketLoad, mng::SocketMng,
};
use basic::logger;
use nix::sys::wait::WaitStatus;
use std::{path::PathBuf, rc::Rc};
use sysmaster::error::*;
use sysmaster::exec::ExecContext;
use sysmaster::rel::{ReStation, Reliability};
use sysmaster::unit::{SubUnit, UmIf, UnitActiveState, UnitBase, UnitMngUtil};

// the structuer of the socket unit type
struct SocketUnit {
    comm: Rc<SocketUnitComm>,
    config: Rc<SocketConfig>,
    mng: SocketMng,
    load: SocketLoad,
}

impl ReStation for SocketUnit {
    // input: do nothing

    // compensate: do nothing

    // data
    fn db_map(&self) {
        self.config.db_map();
        self.mng.db_map();
    }

    fn db_insert(&self) {
        self.config.db_insert();
        self.mng.db_insert();
    }

    // reload: entry-only
    fn entry_coldplug(&self) {
        // rebuild external connections, like: timer, ...
        self.mng.entry_coldplug();
    }

    fn entry_clear(&self) {
        // release external connection, like: timer, ...
        self.mng.entry_clear();
    }
}

impl SubUnit for SocketUnit {
    fn load(&self, paths: Vec<PathBuf>) -> Result<()> {
        log::debug!("socket begin to load conf file");
        self.config.load(paths, true)?;

        let ret = self.load.socket_add_extras();
        if ret.is_err() {
            self.config.reset();
            return ret;
        }

        self.mng.build_ports();

        self.verify()
    }

    // the function entrance to start the unit
    fn start(&self) -> Result<()> {
        let starting = self.mng.start_check()?;
        if starting {
            log::debug!("socket already in start");
            return Ok(());
        }

        self.mng.start_action();

        Ok(())
    }

    fn stop(&self, force: bool) -> Result<()> {
        if !force {
            let stopping = self.mng.stop_check()?;
            if stopping {
                log::debug!("socket already in stop, return immediretly");
                return Ok(());
            }
        }

        self.mng.stop_action();

        Ok(())
    }

    fn sigchld_events(&self, wait_status: WaitStatus) {
        self.mng.sigchld_event(wait_status)
    }

    fn current_active_state(&self) -> UnitActiveState {
        self.mng.current_active_state()
    }

    fn get_subunit_state(&self) -> String {
        self.mng.get_state()
    }

    fn collect_fds(&self) -> Vec<i32> {
        self.mng.collect_fds()
    }

    fn attach_unit(&self, unit: Rc<dyn UnitBase>) {
        self.comm.attach_unit(unit);
        self.db_insert();
    }
}

// attach the UnitManager for weak reference
impl UnitMngUtil for SocketUnit {
    fn attach_um(&self, um: Rc<dyn UmIf>) {
        self.comm.attach_um(um);
    }

    fn attach_reli(&self, reli: Rc<Reliability>) {
        self.comm.attach_reli(reli);
    }
}

impl SocketUnit {
    fn new(_um: Rc<dyn UmIf>) -> SocketUnit {
        let context = Rc::new(ExecContext::new());
        let _comm = Rc::new(SocketUnitComm::new());
        let _config = Rc::new(SocketConfig::new(&_comm));
        SocketUnit {
            comm: Rc::clone(&_comm),
            config: Rc::clone(&_config),
            mng: SocketMng::new(&_comm, &_config, &context),
            load: SocketLoad::new(&_config, &_comm),
        }
    }

    fn find_symlink_target(&self) -> Option<String> {
        if self.config.ports().is_empty() {
            return None;
        }
        let mut res: Option<String> = None;
        for port in self.config.ports() {
            if !port.can_be_symlinked() {
                continue;
            }
            /* Already found one target, refuse if there are more. */
            if res.is_some() {
                return None;
            }
            res = Some(port.listen().to_string());
        }
        res
    }

    fn verify(&self) -> Result<()> {
        let config = self.config.config_data();
        if config.borrow().Socket.Symlinks.is_some()
            && !config.borrow().Socket.Symlinks.as_ref().unwrap().is_empty()
            && self.find_symlink_target().is_none()
        {
            /* Set to None, so we won't create symlinks by mistake. */
            config.borrow_mut().Socket.Symlinks = None;
            log::error!("Symlinks in [Socket] is configured, but there are none or more than one listen files.");
        }
        Ok(())
    }
}

// define the method to create the instance of the unit
use sysmaster::declure_unitobj_plugin_with_param;
declure_unitobj_plugin_with_param!(SocketUnit, SocketUnit::new, PLUGIN_NAME);
