use quadoculars::{Fstate, Watch};
use std::{env, path::PathBuf, str::FromStr, sync::mpsc::channel};

fn main() -> anyhow::Result<()> {
    let mut vec_files = Vec::new();
    {
        let mut files = Vec::new();
        files.push("files\\file 1.xml");
        files.push("files\\file 2.xaml");
        files.push("files\\file 3.json");
        files.push("files\\file 4.toml");
        files.push("files\\file 5.ron");
        for file in files {
            let mut absolute_file: PathBuf = PathBuf::from_str("")?;
            if let Ok(path) = env::var("CARGO_MANIFEST_DIR") {
                absolute_file.clear();
                absolute_file = PathBuf::from(path + "\\" + file);
            }
            vec_files.push(absolute_file);
        }
    }

    let (tx, rx) = channel();

    while let Ok(file_exist) = Watch::new()
        .set_timeout(0.6)
        .multiple_files(&mut vec_files, tx.clone())
    {
        if !file_exist {
            println!("no file to watch");
            break;
        } else {
            vec_files
                .iter()
                .for_each(|file| println!("watching... {:?}", file));
        }
        for state in &rx {
            match state {
                Fstate::Changed(file) => {
                    println!("{:?} changed", file)
                }
                Fstate::NotFound(file) => {
                    println!("file {:?} not found, possibly removed or something, can't watch it any longer.",file);
                    if Watch::is_continue(&mut vec_files, &file) {
                        println!(
                            "and there's {} file(s) remain still, watcher(s) continue watching:",
                            vec_files.len()
                        );
                        vec_files.iter().for_each(|f| println!("{:?}", f));
                    } else {
                        println!("there's no file remain, all watchers will be terminated.");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
