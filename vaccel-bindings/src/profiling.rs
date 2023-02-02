#![allow(dead_code, unused_variables)]
use crate::ffi;

use std::{
    collections::{btree_map, BTreeMap},
    time::Duration,
    ops::Deref,
};

use protocols::profiling::ProfRegions as ProtProfRegions;
use protocols::profiling::ProfRegion as ProtProfRegion;
use protocols::profiling::ProfRegion_Sample as ProtSample;

const NSEC_PER_SEC: u32 = 1_000_000_000;

#[derive(Debug, Clone, Copy)]
struct Timespec {
    tv_sec: u64,
    tv_nsec: u32,
}

impl Timespec {
    pub const fn zero() -> Timespec {
        Timespec::new(0, 0)
    }

    const fn new(tv_sec: u64, tv_nsec: u32) -> Timespec {
        assert!(tv_nsec < NSEC_PER_SEC);
        Timespec { tv_sec, tv_nsec }
    }

    pub fn now() -> Timespec {
        let mut t = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        assert_eq!(
            unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut t) },
            0
        );
        Timespec::new(t.tv_sec as u64, t.tv_nsec as u32)
    }

    pub fn elapsed(&self) -> Duration {
        let t = Timespec::now();
        let d1 = Duration::new(self.tv_sec, self.tv_nsec);
        let d2 = Duration::new(t.tv_sec, t.tv_nsec);
        d2 - d1
    }

    pub const fn as_nanos(&self) -> u128 {
        Duration::new(self.tv_sec, self.tv_nsec).as_nanos()
    }

    pub const fn from_nanos(nanos: u64) -> Timespec {
        Timespec::new(nanos / (NSEC_PER_SEC as u64), (nanos % (NSEC_PER_SEC as u64)) as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sample {
    inner: ffi::vaccel_prof_sample,
    start: Timespec,
    time: Duration,
}

impl Sample {
    const fn new(start: Timespec, time: Duration) -> Self {
        Sample {
            inner: ffi::vaccel_prof_sample {
                start: start.as_nanos() as u64,
                time: time.as_nanos() as u64,
            },
            start: start,
            time: time,
        }
    }
}

impl Default for Sample {
    fn default() -> Self {
        Sample::new(Timespec::now(), Duration::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfRegions {
    map: BTreeMap<String, Vec<Sample>>,
    name: String,
}

// this will in turn implement the Iterator trait
impl Deref for ProfRegions {
    type Target = BTreeMap<String, Vec<Sample>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl IntoIterator for ProfRegions {
    type Item = (String, Vec<Sample>);
    type IntoIter = btree_map::IntoIter<String, Vec<Sample>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl Extend<(String, Vec<Sample>)> for ProfRegions
{
    fn extend<T: IntoIterator<Item = (String, Vec<Sample>)>>(&mut self, iter: T) {
        self.map.extend(iter)
    }
}

impl ProfRegions {
    pub fn new(name: &str) -> ProfRegions {
        ProfRegions {
            map: BTreeMap::new(),
            name: name.to_string(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn clear(&mut self) {
        #[cfg(debug_assertions)]
        self.map.clear();
    }

    pub fn insert(&mut self, name: &str, samples: Vec<Sample>) {
        #[cfg(debug_assertions)]
        self.map.insert(name.to_string(), samples);
    }

    pub fn start(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        {
            self.map
                .entry(format!("[{}] {}", self.name, name))
                .and_modify(|e| e.push(Sample::default()))
                .or_insert(vec![Sample::default()]);
        }
    }

    pub fn stop(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        {
            self.map
                .entry(format!("[{}] {}", self.name, name))
                .and_modify(|e| {
                    if let Some(t) = e.last_mut() {
                        t.time = t.start.elapsed();
                        t.inner.time = t.time.as_nanos() as u64;
                    }
                });
        }
    }

    pub fn get_single(&self, name: &str) -> Option<&Vec<Sample>> {
        #[cfg(debug_assertions)]
        {
            self.map.get(&format!("[{}] {}", self.name, name))
        }
        #[cfg(not(debug_assertions))]
        None
    }

    pub fn get(&self) -> Option<&BTreeMap<String, Vec<Sample>>> {
        #[cfg(debug_assertions)]
        {
            Some(&self.map)
        }
        #[cfg(not(debug_assertions))]
        None
    }

    pub fn get_ffi(&self) -> Option<BTreeMap<String, Vec<ffi::vaccel_prof_sample>>> {
        #[cfg(debug_assertions)]
        {
            Some(
                self.map
                    .iter()
                    .map(|(k, v)| {
                        let s: Vec<ffi::vaccel_prof_sample> = v.iter().map(|t| t.inner).collect();
                        (k.clone(), s)
                    })
                    .collect(),
            )
        }
        #[cfg(not(debug_assertions))]
        None
    }

    fn format(name: &str, time: u128, entries: usize) -> String {
        #[cfg(debug_assertions)]
        {
            format!("{name}: total_time: {time} nsec nr_entries: {entries}")
        }
        #[cfg(not(debug_assertions))]
        String::new()
    }

    pub fn print_single(&self, name: &str) {
        #[cfg(debug_assertions)]
        {
            let n = format!("[{}] {}", self.name, name);
            if let Some(e) = self.map.get(&n) {
                if let Some(t) = e.last() {
                    println!("{}", ProfRegions::format(&n, t.time.as_nanos(), 1));
                }
            }
        }
    }

    pub fn print_total_single(&self, name: &str) {
        #[cfg(debug_assertions)]
        {
            let n = format!("[{}] {}", self.name, name);
            if let Some(e) = self.map.get(&n) {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                println!("{}", ProfRegions::format(&n, s, e.len()));
            }
        }
    }

    pub fn print(&self) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.map {
                if let Some(t) = e.last() {
                    println!("{}", ProfRegions::format(&n, t.time.as_nanos(), 1));
                }
            }
        }
    }

    pub fn print_total(&self) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.map {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                println!("{}", ProfRegions::format(&n, s, e.len()));
            }
        }
    }

    pub fn print_to_buf(&self) -> String {
        #[cfg(debug_assertions)]
        {
            let mut buf = Vec::new();
            for (n, e) in &self.map {
                if let Some(t) = e.last() {
                    buf.push(ProfRegions::format(&n, t.time.as_nanos(), 1));
                }
            }
            buf.join("\n")
        }
        #[cfg(not(debug_assertions))]
        String::new()
    }

    pub fn print_total_to_buf(&self) -> String {
        #[cfg(debug_assertions)]
        {
            let mut buf = Vec::new();
            for (n, e) in &self.map {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                buf.push(ProfRegions::format(&n, s, e.len()));
            }
            buf.join("\n")
        }
        #[cfg(not(debug_assertions))]
        String::new()
    }
}

impl From<&mut ProtSample> for Sample {
    fn from(arg: &mut ProtSample) -> Self {
        Sample::new(Timespec::from_nanos(arg.get_start()), Duration::from_nanos(arg.get_time()))
    }
}

impl From<&ProtSample> for Sample {
    fn from(arg: &ProtSample) -> Self {
        Sample::new(Timespec::from_nanos(arg.get_start()), Duration::from_nanos(arg.get_time()))
    }
}

impl From<ProtProfRegions> for ProfRegions {
    fn from(arg: ProtProfRegions) -> Self {
        let mut t = ProfRegions::new("test");

        for pt in arg.get_timer().into_iter() {
            let s: Vec<Sample> = pt.get_samples().into_iter().map(|x| x.into()).collect();
            t.insert(pt.get_name(), s);
        }
        t
    }
}

impl From<&Sample> for ProtSample {
    fn from(arg: &Sample) -> Self {
        let mut s = ProtSample::new();
        s.set_start(arg.start.as_nanos() as u64);
        s.set_time(arg.time.as_nanos() as u64);
        s
    }
}

impl From<ProfRegions> for ProtProfRegions {
    fn from(arg: ProfRegions) -> Self {
        let mut pt: Vec<ProtProfRegion> = Vec::new();
        for (n, t) in arg.iter() {
            let mut p = ProtProfRegion::new();
            p.set_name(n.to_string());
            p.set_samples(t.iter().map(|x| x.into()).collect());
            pt.push(p);
        }
        let mut t = ProtProfRegions::new();
        t.set_timer(pt.into());
        t
    }
}

/*
fn main() {
    let t = Timespec::now();
    std::thread::sleep(Duration::from_secs(1));
    println!("{}", t.elapsed().as_nanos());

    let mut timers = ProfRegions::new();

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(1));
    timers.stop("test");
    timers.print("test", "");

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(2));
    timers.stop("test");
    timers.print("test", "");

    timers.start("test1");
    #[cfg(debug_assertions)]
    std::thread::sleep(Duration::from_secs(1));
    timers.stop("test1");

    timers.print_avg("test", "");
    timers.stop("test2");
    timers.print_avg("test2", "");
    #[cfg(debug_assertions)]
    println!("ALL:");
    timers.print_all("");

    #[cfg(debug_assertions)]
    {
        println!("{:?}", timers.get("test"));
        println!("{:?}", timers.get("test2"));
    }

    #[cfg(debug_assertions)]
    println!("{}", timers.print_all_avg_to_buf("vaccel"));
}
*/
