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

#[cfg(test)]
pub(crate) mod test_utils {
    use std::rc::Rc;

    use crate::{
        plugin::Plugin,
        unit::{data::DataManager, unit_name_to_type},
        unit::{entry::UnitX, rentry::UnitRe, util::UnitFile},
    };
    use basic::path_lookup::LookupPaths;
    use sysmaster::rel::Reliability;
    use sysmaster::unit::UmIf;
    pub(crate) struct UmIfD;
    impl UmIf for UmIfD {}

    pub(crate) fn create_unit_for_test_pub(
        dmr: &Rc<DataManager>,
        relir: &Rc<Reliability>,
        rentryr: &Rc<UnitRe>,
        name: &str,
    ) -> Rc<UnitX> {
        let mut l_path = LookupPaths::new();
        l_path.init_lookup_paths();
        let lookup_path = Rc::new(l_path);

        let file = Rc::new(UnitFile::new(&lookup_path));
        let unit_type = unit_name_to_type(name);
        let umifd = Rc::new(UmIfD);
        let plugins = Plugin::get_instance();
        let subclass = plugins.create_subunit_with_um(unit_type, umifd).unwrap();
        subclass.attach_reli(Rc::clone(relir));
        Rc::new(UnitX::new(dmr, rentryr, &file, unit_type, name, subclass))
    }
}
