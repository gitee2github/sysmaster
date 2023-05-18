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

use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub enum TableOp<'a, K, V> {
    TableInsert(&'a K, &'a V),
    TableRemove(&'a K, &'a V),
}

pub trait TableSubscribe<K, V> {
    fn filter(&self, _op: &TableOp<K, V>) -> bool {
        // default: everything is allowed
        true
    }

    fn notify(&self, op: &TableOp<K, V>);
}

//#[derive(Debug)]
pub struct Table<K, V> {
    data: HashMap<K, V>,                                        // key + value
    subscribers: HashMap<String, Rc<dyn TableSubscribe<K, V>>>, // key: name, value: subscriber
}

impl<K, V> Table<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Table<K, V> {
        Table {
            data: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    pub fn data_clear(&mut self) {
        // clear all data without notifying subscribers
        self.data.clear();
    }

    pub fn clear(&mut self) {
        // clear all, including data and subscribers
        self.data_clear();
        self.subscribers.clear();
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let key = k.clone();
        let ret = self.data.insert(k, v);
        let value = self
            .data
            .get(&key)
            .expect("something inserted is not found.");
        let op = TableOp::TableInsert(&key, value);
        self.notify(&op);
        ret
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let ret = self.data.remove(k);
        if let Some(v) = &ret {
            let op = TableOp::TableRemove(k, v);
            self.notify(&op);
        }
        ret
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.data.get(k)
    }

    pub fn get_all(&self) -> Vec<&V> {
        self.data.values().collect::<Vec<_>>()
    }

    pub fn subscribe(
        &mut self,
        name: String,
        subscriber: Rc<dyn TableSubscribe<K, V>>,
    ) -> Option<Rc<dyn TableSubscribe<K, V>>> {
        self.subscribers.insert(name, subscriber)
    }

    #[allow(dead_code)]
    pub fn unsubscribe(&mut self, name: &str) -> Option<Rc<dyn TableSubscribe<K, V>>> {
        self.subscribers.remove(name)
    }

    fn notify(&self, op: &TableOp<'_, K, V>) {
        for (_, subscriber) in self.subscribers.iter() {
            if subscriber.filter(op) {
                subscriber.notify(op);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn table_insert() {
        let mut table: Table<u32, char> = Table::new();

        let old = table.insert(1, 'a');
        assert_eq!(old, None);

        let old = table.insert(1, 'b');
        assert_eq!(old, Some('a'));

        let old = table.insert(2, 'a');
        assert_eq!(old, None);
    }

    #[test]
    fn table_remove() {
        let mut table: Table<u32, char> = Table::new();

        let old = table.remove(&1);
        assert_eq!(old, None);

        table.insert(1, 'a');
        let old = table.remove(&1);
        assert_eq!(old, Some('a'));

        table.insert(1, 'a');
        table.insert(2, 'b');
        let old = table.remove(&3);
        assert_eq!(old, None);
        let old = table.remove(&2);
        assert_eq!(old, Some('b'));
    }

    #[test]
    fn table_get() {
        let mut table: Table<u32, char> = Table::new();

        let value = table.get(&1);
        assert_eq!(value, None);

        table.insert(1, 'a');
        let value = table.get(&1);
        assert_eq!(value.cloned(), Some('a'));
        let value = table.get(&2);
        assert_eq!(value, None);

        table.insert(2, 'b');
        let value = table.get(&2);
        assert_eq!(value.cloned(), Some('b'));
    }

    #[test]
    fn table_subscribe() {
        let mut table: Table<u32, char> = Table::new();
        let sub_test1 = Rc::new(TableTest::new());
        let sub_test2 = Rc::new(TableTest::new());

        let sub = Rc::clone(&sub_test1);
        let old = table.subscribe(String::from("test1"), sub);
        assert!(old.is_none());

        let sub = Rc::clone(&sub_test2);
        let old = table.subscribe(String::from("test1"), sub);
        assert!(old.is_some());

        let sub = Rc::clone(&sub_test2);
        let old = table.subscribe(String::from("test2"), sub);
        assert!(old.is_none());
    }

    #[test]
    fn table_unsubscribe() {
        let mut table: Table<u32, char> = Table::new();
        let sub_test1 = Rc::new(TableTest::new());
        let sub_test2 = Rc::new(TableTest::new());

        let old = table.unsubscribe("test1");
        assert!(old.is_none());

        let sub = Rc::clone(&sub_test1);
        table.subscribe(String::from("test1"), sub);
        let old = table.unsubscribe("test1");
        assert!(old.is_some());

        let sub = Rc::clone(&sub_test1);
        table.subscribe(String::from("test1"), sub);
        let sub = Rc::clone(&sub_test2);
        table.subscribe(String::from("test2"), sub);
        let old = table.unsubscribe("test3");
        assert!(old.is_none());
        let old = table.unsubscribe("test2");
        assert!(old.is_some());
    }

    #[test]
    fn table_notify() {
        let mut table: Table<u32, char> = Table::new();
        let sub_test1 = Rc::new(TableTest::new());
        let sub_test2 = Rc::new(TableTest::new());

        let sub = Rc::clone(&sub_test1);
        table.subscribe(String::from("test1"), sub);
        table.insert(1, 'a');
        assert_eq!(sub_test1.get_key(), Some(1));
        assert_eq!(sub_test1.get_value(), Some('a'));
        table.remove(&1);
        assert_eq!(sub_test1.get_key(), None);
        assert_eq!(sub_test1.get_value(), None);
        table.insert(1, 'a');
        assert_eq!(sub_test1.get_key(), Some(1));
        assert_eq!(sub_test1.get_value(), Some('a'));
        table.insert(2, 'b');
        assert_eq!(sub_test1.get_key(), Some(2));
        assert_eq!(sub_test1.get_value(), Some('b'));

        let sub = Rc::clone(&sub_test2);
        table.subscribe(String::from("test2"), sub);
        table.insert(1, 'a');
        assert_eq!(sub_test1.get_key(), Some(1));
        assert_eq!(sub_test1.get_value(), Some('a'));
        assert_eq!(sub_test2.get_key(), Some(1));
        assert_eq!(sub_test2.get_value(), Some('a'));
    }

    struct TableTest {
        key: RefCell<Option<u32>>,
        value: RefCell<Option<char>>,
    }

    impl TableTest {
        fn new() -> TableTest {
            TableTest {
                key: RefCell::new(None),
                value: RefCell::new(None),
            }
        }

        fn get_key(&self) -> Option<u32> {
            *self.key.borrow()
        }

        fn get_value(&self) -> Option<char> {
            *self.value.borrow()
        }
    }

    impl TableSubscribe<u32, char> for TableTest {
        fn notify(&self, op: &TableOp<u32, char>) {
            match op {
                TableOp::TableInsert(k, v) => {
                    *self.key.borrow_mut() = Some(**k);
                    *self.value.borrow_mut() = Some(**v);
                }
                TableOp::TableRemove(_, _) => {
                    *self.key.borrow_mut() = None;
                    *self.value.borrow_mut() = None;
                }
            }
        }
    }
}
