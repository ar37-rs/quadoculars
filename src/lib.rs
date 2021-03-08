use cfg_if::cfg_if;
cfg_if! {
   if #[cfg(feature = "crossbeam_channel")] {
       use crossbeam_channel::{bounded, Sender};
   } else if #[cfg(feature = "flume_channel")] {
       use flume::{bounded, Sender};
   } else {
       use std::sync::mpsc::{sync_channel, Sender};
   }
}

cfg_if! {
    if #[cfg(feature = "live_json")] {
        use serde_json::Value;
    }
}

use notify::{
    event::ModifyKind, EventKind, RecommendedWatcher, RecursiveMode::NonRecursive, Result, Watcher,
};
use std::{
    cmp::Ordering,
    fs::{File, OpenOptions},
    io::{BufReader, Read},
    path::PathBuf,
    thread::{sleep, spawn},
    time::Duration,
    usize,
};

const ZERO: usize = 0;
const MILLIS: f32 = 1000.0;
const BREAK_POINT: usize = 7;
const TRUE: bool = true;
const FALSE: bool = false;

/// File state.
#[derive(Clone)]
pub enum Fstate<T> {
    Changed(T),
    NotFound(T),
}

#[inline]
fn watch(file: PathBuf, tx: Sender<Fstate<PathBuf>>, timeout: f32) -> Result<()> {
    cfg_if! {
        if  #[cfg(feature = "crossbeam_channel")] {
           let (tx1, rx1) = bounded(ZERO);
        } else if #[cfg(feature = "flume_channel")] {
           let (tx1, rx1) = bounded(ZERO);
        }  else {
           let (tx1, rx1) = sync_channel(ZERO);
        }
    }

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |result| {
        let _ = tx1.try_send(result);
    })?;
    watcher.watch(&file, NonRecursive)?;

    let mut first_data;
    {
        match OpenOptions::new().read(TRUE).open(&file) {
            Ok(tmp_file) => {
                let mut buf = Vec::new();
                BufReader::new(tmp_file).read_to_end(&mut buf)?;
                first_data = buf;
            }
            Err(e) => {
                panic!("{} {}", e, file.to_string_lossy());
            }
        }
    }

    let duration: Duration;
    {
        let timeout = (timeout * MILLIS) as u32;
        duration = Duration::from_millis((timeout / BREAK_POINT as u32).into());
    }

    while let Some(_file_) = Some(file.exists()) {
        if _file_ {
            if let Ok(event) = rx1.recv_timeout(duration) {
                match event?.kind {
                    EventKind::Modify(kind) => match kind {
                        ModifyKind::Any => match File::open(&file) {
                            Ok(tmp_file) => {
                                let mut seconds_data = Vec::new();
                                BufReader::new(tmp_file).read_to_end(&mut seconds_data)?;
                                let seconds_data_len = seconds_data.len();
                                if seconds_data_len > ZERO {
                                    match first_data.cmp(&seconds_data) {
                                        Ordering::Less | Ordering::Greater => {
                                            let _ = tx.send(Fstate::Changed(file.clone()));
                                            first_data = seconds_data;
                                        }
                                        _ => drop(seconds_data),
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = tx.send(Fstate::NotFound(file.clone()));
                                break;
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                }
            }
        } else {
            let mut check_point = ZERO;
            'retry: while check_point <= BREAK_POINT {
                sleep(duration);
                match File::open(&file) {
                    Ok(tmp_file) => {
                        let mut seconds_data = Vec::new();
                        BufReader::new(tmp_file).read_to_end(&mut seconds_data)?;
                        let seconds_data_len = seconds_data.len();
                        if seconds_data_len > ZERO {
                            match first_data.cmp(&seconds_data) {
                                Ordering::Less | Ordering::Greater => {
                                    let _ = tx.send(Fstate::Changed(file.clone()));
                                    first_data = seconds_data;
                                    break;
                                }
                                _ => drop(seconds_data),
                            }
                        }
                    }
                    Err(_) => {
                        check_point += 1;
                        continue 'retry;
                    }
                }
            }
            if check_point > BREAK_POINT {
                let _ = tx.send(Fstate::NotFound(file.clone()));
                break;
            }
        }
    }

    drop(first_data);
    watcher.unwatch(file)?;
    drop(watcher);
    Ok(())
}

const TIMEOUT: f32 = 0.63;
#[derive(Clone)]
pub struct Watch {
    timeout: f32,
}

impl Watch {
    pub fn new() -> Watch {
        Self { timeout: TIMEOUT }
    }

    /// Set timeout. so if the file renamed/removed permanently, the watcher will be able to terminate itself.
    ///
    /// default value is 0.63 seconds, set large than that if intended for watching huge file.
    pub fn set_timeout(mut self, duration: f32) -> Watch {
        if duration > TIMEOUT {
            self.timeout = duration;
        }
        self
    }

    /// Single file watcher
    #[inline]
    pub fn single_file<'a>(&self, file: &'a PathBuf, tx: Sender<Fstate<PathBuf>>) -> Result<bool> {
        let timeout = self.timeout;
        if file.exists() {
            let file = file.clone();
            spawn(move || {
                watch(file, tx, timeout).expect("error occured while spawning watcher.");
            });
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }

    /// Technically the same as single_file watcher, but for multilple files.
    #[inline]
    pub fn multiple_files<'a>(
        &self,
        vec_files: &mut Vec<PathBuf>,
        tx: Sender<Fstate<PathBuf>>,
    ) -> Result<bool> {
        let tmp_vec_files: Vec<PathBuf>;
        {
            tmp_vec_files = vec_files
                .clone()
                .into_iter()
                .filter(|path| path.is_file())
                .collect::<Vec<_>>();
        }
        if tmp_vec_files.len() > ZERO {
            let timeout = self.timeout;
            *vec_files = tmp_vec_files;
            let mut vec_transmitter = Vec::new();
            {
                for _ in ZERO..vec_files.len() {
                    vec_transmitter.push(tx.clone());
                }
            }
            for (i, tx) in vec_transmitter.iter().enumerate() {
                let tx = tx.to_owned();
                let file = vec_files[i].clone();
                spawn(move || {
                    watch(file, tx, timeout).expect("error occured while spawning watcher.");
                });
            }
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }

    /// Additional for multiple_files to check if there's watcher(s) still continue watching the file(s).
    #[inline]
    pub fn is_continue<'a>(vec_files: &mut Vec<PathBuf>, file: &'a PathBuf) -> bool {
        let tmp_files = vec_files.clone();
        if !file.exists() {
            for (i, _file) in tmp_files.iter().enumerate() {
                if file == _file {
                    vec_files.remove(i);
                }
            }
        }
        match vec_files.len().cmp(&ZERO) {
            Ordering::Greater => TRUE,
            _ => FALSE,
        }
    }

    #[cfg(feature = "live_json")]
    /// (Optional, if needed) reinit mutable stuct before calling fn json_de.
    pub fn reinit_de_json<'a, T>(&mut self, mut_struct: &mut T, json: &'a PathBuf)
    where
        T: serde::de::DeserializeOwned,
    {
        if let Ok(file) = File::open(json) {
            match serde_json::from_reader(BufReader::new(file)) {
                Ok(loaded) => {
                    // if implemented from missing members, rust analyzer usually will change *self to *oculars (this crate), just change it back from *oculars to *self
                    *mut_struct = loaded;
                }
                _ => (),
            }
        }
    }

    #[cfg(feature = "live_json")]
    /// Live reload DeserializeOwned struct from json.
    #[inline]
    pub fn de_json<'a, T>(&self, mut_struct: &mut T, json: &'a PathBuf) -> Result<bool>
    where
        T: serde::de::DeserializeOwned,
    {
        let timeout = self.timeout;
        cfg_if! {
            if  #[cfg(feature = "crossbeam_channel")] {
               let (tx, rx) = bounded(ZERO);
            } else if #[cfg(feature = "flume_channel")] {
               let (tx, rx) = bounded(ZERO);
            }  else {
               let (tx, rx) = std::sync::mpsc::channel();
            }
        }

        if json.exists() {
            let json = json.clone();
            spawn(move || {
                watch(json, tx, timeout).expect("error occured while spawning watcher.");
            });
            if let Ok(state) = rx.recv() {
                match state {
                    Fstate::Changed(json) => {
                        let mut reader = BufReader::new(File::open(json)?);
                        match serde_json::from_reader(&mut reader) {
                            Ok(new_data) => {
                                *mut_struct = new_data;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }

    #[cfg(feature = "live_json")]
    /// (Optional, if needed) Initialize empty json Value before calling fn json_val.
    pub fn json_val_init<'a>(&self, json: &'a PathBuf, val: &mut Value) {
        if let Ok(file) = File::open(json) {
            match serde_json::from_reader(BufReader::new(file)) {
                Ok(loaded) => {
                    *val = loaded;
                }
                _ => (),
            }
        }
    }

    #[cfg(feature = "live_json")]
    /// Live reload serde_json Value.
    #[inline]
    pub fn json_val<'a>(&self, json: &'a PathBuf, val: &mut Value) -> Result<bool> {
        let timeout = self.timeout;
        cfg_if! {
            if  #[cfg(feature = "crossbeam_channel")] {
               let (tx, rx) = bounded(ZERO);
            } else if #[cfg(feature = "flume_channel")] {
               let (tx, rx) = bounded(ZERO);
            }  else {
               let (tx, rx) = std::sync::mpsc::channel();
            }
        }
        if json.exists() {
            let json = json.clone();
            spawn(move || {
                watch(json, tx, timeout).expect("error occured while spawning watcher.");
            });
            if let Ok(state) = rx.recv() {
                match state {
                    Fstate::Changed(json) => {
                        let mut reader = BufReader::new(File::open(json)?);
                        match serde_json::from_reader(&mut reader) {
                            Ok(new_data) => {
                                *val = new_data;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }

    #[cfg(feature = "live_ron")]
    /// (Optional, if needed) reinit mutable stuct before calling fn de_ron.
    pub fn reinit_de_ron<'a, T>(&mut self, mut_struct: &mut T, ron: &'a PathBuf)
    where
        T: serde::de::DeserializeOwned,
    {
        if let Ok(file) = File::open(ron) {
            match ron::de::from_reader(BufReader::new(file)) {
                Ok(loaded) => {
                    *mut_struct = loaded;
                }
                _ => (),
            }
        }
    }

    #[cfg(feature = "live_ron")]
    /// Live reload DeserializeOwned struct from ron.
    #[inline]
    pub fn de_ron<'a, T>(&self, mut_struct: &mut T, ron: &'a PathBuf) -> Result<bool>
    where
        T: serde::de::DeserializeOwned,
    {
        let timeout = self.timeout;
        cfg_if! {
            if  #[cfg(feature = "crossbeam_channel")] {
               let (tx, rx) = bounded(ZERO);
            } else if #[cfg(feature = "flume_channel")] {
               let (tx, rx) = bounded(ZERO);
            }  else {
               let (tx, rx) = std::sync::mpsc::channel();
            }
        }

        if ron.exists() {
            let ron = ron.clone();
            spawn(move || {
                watch(ron, tx, timeout).expect("error occured while spawning watcher.");
            });
            if let Ok(state) = rx.recv() {
                match state {
                    Fstate::Changed(ron) => {
                        let mut reader = BufReader::new(File::open(ron)?);
                        match ron::de::from_reader(&mut reader) {
                            Ok(new_data) => {
                                *mut_struct = new_data;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }
}

#[cfg(feature = "live_json")]
/// Instant trait for live reloading json values for DeserializeOwned struct,
pub trait LiveJson {
    /// (Optional) reinit mutable stuct if needed.
    fn reinit_from_json<'a>(&mut self, json: &'a PathBuf)
    where
        Self: serde::de::DeserializeOwned,
    {
        if let Ok(file) = File::open(json) {
            match serde_json::from_reader(BufReader::new(file)) {
                Ok(loaded) => {
                    // if implemented from missing members, rust analyzer usually will change *self to *quadoculars (this crate), just change it back from *oculars to *self
                    *self = loaded;
                }
                _ => (),
            }
        }
    }

    /// Start live reload mutable stuct,
    ///
    /// Note: default timeout value is 0.63 seconds.
    ///
    /// if the json file renamed/removed permanently, the watcher will terminate itself according to the given timeout value.
    #[inline]
    fn reload_from_json<'a>(&mut self, json: &'a PathBuf, timeout: f32) -> Result<bool>
    where
        Self: serde::de::DeserializeOwned,
    {
        // and this, change input mut_struct: from quadoculars to self
        Watch::new().set_timeout(timeout).de_json(self, json)
    }
}

#[cfg(feature = "live_ron")]
/// Instant trait for live reloading ron values for DeserializeOwned struct.
pub trait LiveRon {
    /// (Optional) reinit mutable stuct if needed.
    fn reinit_from_ron<'a>(&mut self, ron: &'a PathBuf)
    where
        Self: serde::de::DeserializeOwned,
    {
        if let Ok(file) = File::open(ron) {
            match ron::de::from_reader(BufReader::new(file)) {
                Ok(loaded) => {
                    // if implemented from missing members, rust analyzer usually will change *self to *quadoculars (this crate), just change it back from *oculars to *self
                    *self = loaded;
                }
                _ => (),
            }
        }
    }

    /// Start live reload mutable stuct,
    ///
    /// Note: default timeout value is 0.63 seconds.
    ///
    /// if the ron file renamed/removed permanently, the watcher will terminate itself according to the given timeout value.
    #[inline]
    fn reload_from_ron<'a>(&mut self, ron: &'a PathBuf, timeout: f32) -> Result<bool>
    where
        Self: serde::de::DeserializeOwned,
    {
        // and this, change input mut_struct: from quadoculars to self
        Watch::new().set_timeout(timeout).de_ron(self, ron)
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use cfg_if::cfg_if;
    cfg_if! {
       if #[cfg(feature = "crossbeam_channel")] {
           use crossbeam_channel::{bounded, Sender};
       } else if #[cfg(feature = "flume_channel")] {
           use flume::{bounded, Sender};
       } else {
           use std::sync::mpsc::{sync_channel, Sender};
       }
    }
    use std::{env, str::FromStr};

    #[test]
    #[allow(unused_variables)]
    fn test_if_file_not_exist() {
        cfg_if! {
            if  #[cfg(feature = "crossbeam_channel")] {
               let (tx, rx) = bounded(ZERO);
            } else if #[cfg(feature = "flume_channel")] {
               let (tx, rx) = bounded(ZERO);
            }  else {
               let (tx, rx) = std::sync::mpsc::channel();
            }
        }

        let file_not_exist = PathBuf::from_str("file.not_exist").unwrap();

        while let Ok(is_watching_file_not_yet_exist) = Watch::new()
            .set_timeout(0.6)
            .single_file(&file_not_exist, tx.to_owned())
        {
            assert_eq!(is_watching_file_not_yet_exist, false);
            break;
        }
    }

    #[test]
    #[allow(unused_variables)]
    fn test_if_file_exist() {
        cfg_if! {
            if  #[cfg(feature = "crossbeam_channel")] {
               let (tx, rx) = bounded(ZERO);
            } else if #[cfg(feature = "flume_channel")] {
               let (tx, rx) = bounded(ZERO);
            }  else {
               let (tx, rx) = std::sync::mpsc::channel();
            }
        }

        let mut file_exist = PathBuf::new();
        {
            if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
                file_exist = PathBuf::from(_pth);
            }
            file_exist.push("Cargo.toml");
        }

        while let Ok(is_watching_cargo_toml) = Watch::new()
            .set_timeout(0.6)
            .single_file(&file_exist, tx.to_owned())
        {
            assert_eq!(is_watching_cargo_toml, true);
            break;
        }
    }
}
