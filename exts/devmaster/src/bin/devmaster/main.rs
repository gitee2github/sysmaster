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

//! devmaster daemon
use event::Events;
use libdevmaster::framework::devmaster::Devmaster;
use std::rc::Rc;

fn main() {
    let events = Rc::new(Events::new().unwrap());

    let devmaster = Devmaster::new(events);

    devmaster.as_ref().borrow().run();

    devmaster.as_ref().borrow().exit();
}
