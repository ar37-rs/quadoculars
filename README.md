# Quadoculars

[![Crates.io](https://img.shields.io/crates/v/quadoculars.svg)](https://crates.io/crates/quadoculars)

Concurrent, composable simple file watcher on top of notify-rs.

## Features
  * easy to use single and multiple files watcher
  * only notify when data of the file changes
  * fault tolerant, continue watching even if the file being replaced and gracefully shutdown itself when the file no longer exist.
  * fast live reloading values for DeserializeToOwned stuct.

## Installation
Add `quadoculars` as a dependency in your `Cargo.toml`:

```toml
quadoculars = "*"
```

or

```toml
quadoculars = { git = "https://github.com/Ar37-rs/quadoculars.git" }
```

## Quick Example

```rust
use quadoculars::{Fstate, Watch};
use std::{
    io::Result,
    path::{Path, PathBuf},
    str::FromStr,
    sync::mpsc::channel,
};

fn main() -> Result<()> {
    let file: PathBuf;
    match PathBuf::from_str("filename.extention") {
        Ok(file_) => file = file_,
        _ => file = Path::new("otherfilename.otherextension").to_path_buf(),
    }

    let (tx, rx) = channel();

    while let Ok(file_exist) = Watch::new().set_timeout(0.6).single_file(&file, tx.clone()) {
        if !file_exist {
            println!("no file to watch");
            break;
        } else {
            println!("watching... {:?}", file)
        }
        for state in &rx {
            match state {
                Fstate::Changed(file) => {
                    println!("{:?} changed", file);
                    // do something...
                }
                Fstate::NotFound(file) => {
                    // handle something...
                    break;
                }
            }
        }
    }
    Ok(())
}

```

## More Examples

Watching Multiple files and live reloading values can be found [here](https://github.com/Ar37-rs/quadoculars/tree/main/example).




