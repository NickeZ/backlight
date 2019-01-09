use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::util::clamp;

// &mut self is used because we want to cache file system accesses
pub trait BacklightDevice {
    /// Set brightness in percent
    fn set(&mut self, value: f32) -> Result<(), Error>;
    /// Set brightness using native format
    fn set_native(&mut self, value: i32) -> Result<(), Error>;
    /// Return actual brightness in percent and native
    fn get(&mut self) -> Result<(f32, i32), Error>;
}

pub struct Intel {
    path: PathBuf,
    max: Option<i32>,
    actual: Option<i32>,
}

impl Intel {
    pub fn new(path: &Path) -> Self {
        Intel {
            path: PathBuf::from(path),
            max: None,
            actual: None,
        }
    }

    fn update_internal(&mut self) -> Result<(), Error> {
        if self.max.is_none() {
            self.max = Some(self.get_number("max_brightness")?);
        }
        if self.actual.is_none() {
            self.actual = Some(self.get_number("actual_brightness")?);
        }
        Ok(())
    }

    fn get_number(&self, file: &str) -> std::io::Result<i32> {
        let mut target = String::new();
        self.get_contents(file, &mut target).unwrap();
        let res = target.trim().parse().unwrap();
        Ok(res)
    }

    fn get_contents(&self, file: &str, target: &mut String) -> std::io::Result<()> {
        let mut filename = OsString::from(self.path.as_os_str());
        filename.push("/");
        filename.push(file);

        let mut file = File::open(&filename)?;
        file.read_to_string(target)?;
        Ok(())
    }
}

impl BacklightDevice for Intel {
    fn set(&mut self, value: f32) -> Result<(), Error> {
        self.update_internal()?;
        if let Some(max) = self.max {
            let value = value * max as f32 * 0.01;
            self.set_native(value.round() as i32)?;
        }
        Ok(())
    }
    fn set_native(&mut self, value: i32) -> Result<(), Error> {
        self.update_internal()?;
        if let Some(max) = self.max {
            let value = clamp(value, 0, max);
            let mut filename = OsString::from(self.path.as_os_str());
            filename.push("/brightness");
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&filename)?;
            write!(file, "{}", value)?;
        }
        Ok(())
    }
    fn get(&mut self) -> Result<(f32, i32), Error> {
        self.update_internal()?;
        // TODO: Is it possible to do these in one if statement?
        if let Some(max) = self.max {
            if let Some(actual) = self.actual {
                let perc = 100. * actual as f32 / max as f32;
                return Ok((perc, actual))
            }
        }
        unreachable!()
    }
}
