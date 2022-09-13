use std::{
    collections::{BinaryHeap, HashMap},
    mem,
    rc::Rc,
};

use crate::{EventType, Source};

use std::u64::MAX as USEC_INFINITY;
const USEC_PER_SEC: u64 = 1000000;
const NSEC_PER_USEC: u64 = 1000;
const NSEC_PER_SEC: u64 = 1000000000;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Timestamp {
    realtime: u64,
    monotonic: u64,
    boottime: u64,
}

impl Timestamp {
    pub fn new() -> Timestamp {
        Self {
            realtime: 0,
            monotonic: 0,
            boottime: 0,
        }
    }

    pub fn now(&mut self) -> Self {
        unsafe {
            let mut tp = mem::MaybeUninit::zeroed().assume_init();
            libc::clock_gettime(libc::CLOCK_REALTIME, &mut tp);
            self.realtime = self.load_nsec(tp);
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut tp);
            self.monotonic = self.load_nsec(tp);
            libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut tp);
            self.boottime = self.load_nsec(tp);
        }
        *self
    }

    fn load_nsec(&self, ts: libc::timespec) -> u64 {
        if ts.tv_sec < 0 || ts.tv_nsec < 0 {
            return USEC_INFINITY;
        }

        if (ts.tv_sec as u64)
            > (USEC_INFINITY - ((ts.tv_nsec as u64) / NSEC_PER_SEC) / USEC_PER_SEC)
        {
            return USEC_INFINITY;
        }

        (ts.tv_sec as u64) * USEC_PER_SEC + (ts.tv_nsec as u64) / NSEC_PER_USEC
    }
}

/// 为timer处理添加的辅助数据结构
#[derive(Debug)]
pub(crate) struct Timer {
    timer_set: HashMap<EventType, TimerInner>,
    timestamp: Timestamp,
}

impl Timer {
    pub fn new() -> Timer {
        Self {
            timer_set: HashMap::new(),
            timestamp: Timestamp::new(),
        }
    }

    pub fn clockid(&self, et: &EventType) -> libc::clockid_t {
        match et {
            EventType::TimerRealtime => libc::CLOCK_REALTIME,
            EventType::TimerBoottime => libc::CLOCK_BOOTTIME,
            EventType::TimerMonotonic => libc::CLOCK_MONOTONIC,
            EventType::TimerRealtimeAlarm => libc::CLOCK_REALTIME_ALARM,
            EventType::TimerBoottimeAlarm => libc::CLOCK_BOOTTIME_ALARM,
            _ => unreachable!(),
        }
    }

    pub fn timerid(&mut self, et: &EventType) -> u64 {
        self.now();
        match et {
            EventType::TimerRealtime => self.timestamp.realtime,
            EventType::TimerBoottime => self.timestamp.boottime,
            EventType::TimerMonotonic => self.timestamp.monotonic,
            EventType::TimerRealtimeAlarm => self.timestamp.realtime,
            EventType::TimerBoottimeAlarm => self.timestamp.boottime,
            _ => unreachable!(),
        }
    }

    pub fn next(&mut self, et: &EventType) -> Option<u64> {
        match self.timer_set.get_mut(et) {
            Some(next) => Some(next.data.peek()?.next()),
            None => None,
        }
    }

    pub fn timer_stored(&self, next: u64) -> libc::itimerspec {
        libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: (next / USEC_PER_SEC) as i64,
                tv_nsec: (next % USEC_PER_SEC) as i64,
            },
        }
    }

    pub fn is_empty(&mut self, et: &EventType) -> bool {
        if let Some(inner) = self.timer_set.get_mut(et) {
            return inner.data.is_empty();
        }
        true
    }

    pub fn push(&mut self, source: Rc<dyn Source>) {
        // calc the time
        let mut next = source.time_relative();
        if next != u64::MAX {
            next += self.timerid(&source.event_type());
        } else {
            next = source.time();
        }

        if next == u64::MAX {
            panic!(
                "You need to implement time() or time_relative(), and cannot be set to u64::MAX"
            );
        }

        let cd = ClockData::new(source.clone(), next);
        let et = source.event_type();
        match self.timer_set.get_mut(&et) {
            Some(t) => {
                t.push(cd);
            }
            None => {
                let mut t = TimerInner::new();
                t.push(cd);
                self.timer_set.insert(et, t);
            }
        };
    }

    pub fn pop(&mut self, et: &EventType) -> Option<Rc<dyn Source>> {
        let next = self.timerid(et);
        // self.now();
        match self.timer_set.get_mut(et) {
            Some(timer) => {
                if timer.data.is_empty() {
                    self.timer_set.remove(et);
                    None
                } else {
                    Some(timer.pop(next)?.source())
                }
            }
            None => None,
        }
    }

    pub fn now(&mut self) -> Timestamp {
        self.timestamp.now()
    }
}

#[derive(Debug)]
pub(crate) struct TimerInner {
    data: BinaryHeap<ClockData>,
}

impl TimerInner {
    pub fn new() -> TimerInner {
        Self {
            data: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, source: ClockData) {
        self.data.push(source);
    }

    pub fn pop(&mut self, next: u64) -> Option<ClockData> {
        match self.data.peek() {
            Some(cd) => {
                if cd.next() <= next {
                    self.data.pop()
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ClockData {
    source: Rc<dyn Source>,
    next: u64,
}

impl ClockData {
    pub fn new(source: Rc<dyn Source>, next: u64) -> ClockData {
        Self { source, next }
    }

    pub fn source(&self) -> Rc<dyn Source> {
        self.source.clone()
    }

    pub fn next(&self) -> u64 {
        self.next
    }
}

impl Ord for ClockData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next.cmp(&other.next).reverse()
    }
}

impl PartialOrd for ClockData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.next.cmp(&other.next).reverse())
    }
}

impl PartialEq for ClockData {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

impl Eq for ClockData {}

#[cfg(test)]
mod test {
    use super::Timestamp;

    #[test]
    fn timestamp() {
        let mut ts = Timestamp::new();
        ts.now();
        println!("{:?}", ts);
    }
}
