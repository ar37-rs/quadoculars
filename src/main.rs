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
                    // do handling...
                    break;
                }
            }
        }
    }
    Ok(())
}
