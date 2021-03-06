use quadoculars::LiveRon;
use serde::Deserialize;
use std::{env, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct Btns {
    btn_dec_label: String,
    btn_dec_pos: BtnDecPos,
}

#[derive(Debug, Clone, Deserialize)]
struct BtnDecPos {
    x: i32,
    y: i32,
}

// implement LiveRon
impl LiveRon for Btns {}

fn main() -> anyhow::Result<()> {
    let mut ron = PathBuf::new();

    {
        if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
            ron = PathBuf::from(_pth);
        }
        ron.push("src\\ron\\Btns.ron");
    }

    let mut btns: Btns = Btns {
        btn_dec_label: String::from("label"),
        btn_dec_pos: BtnDecPos { x: 0, y: 0 },
    };

    btns.reinit_from_ron(&ron);

    let btn_dec_label = &btns.btn_dec_label;
    let x = btns.btn_dec_pos.x;
    let y = btns.btn_dec_pos.y;
    println!("btn_dec_label: {}", btn_dec_label);
    println!("btn_dec_pos.x: {}", x);
    println!("btn_dec_pos.x: {}", y);
    println!(" ");

    while let Ok(watching) = btns.reload_from_ron(&ron, 0.6) {
        if watching {
            let btn_dec_label = btns.btn_dec_label.clone();
            let x = btns.btn_dec_pos.x;
            let y = btns.btn_dec_pos.y;
            println!("btn_dec_label: {}", btn_dec_label);
            println!("btn_dec_pos.x: {}", x);
            println!("btn_dec_pos.x: {}", y);
            println!(" ");
        } else {
            println!("latest captured value:");
            let btn_dec_label = &btns.btn_dec_label;
            let x = btns.btn_dec_pos.x;
            let y = btns.btn_dec_pos.y;
            println!("btn_dec_label: {}", btn_dec_label);
            println!("btn_dec_pos.x: {}", x);
            println!("btn_dec_pos.x: {}", y);
            break;
        }
    }

    Ok(())
}

// Multi-thread example
// use quadoculars::LiveRon;
// use serde::Deserialize;
// use std::{
//     env,
//     path::PathBuf,
//     thread,
//     time::Duration,
// };

// #[derive(Debug, Clone, Deserialize)]
// struct Btns {
//     btn_dec_label: String,
//     btn_dec_pos: BtnDecPos,
// }

// #[derive(Debug, Clone, Deserialize)]
// struct BtnDecPos {
//     x: i32,
//     y: i32,
// }

// impl LiveRon for Btns {}

// fn main() -> anyhow::Result<()> {
//     let mut ron = PathBuf::new();

//     {
//         if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
//             ron = PathBuf::from(_pth);
//         }
//         ron.push("src\\ron\\Btns.ron");
//     }

//     let mut btns: Btns = Btns {
//         btn_dec_label: String::from("label"),
//         btn_dec_pos: BtnDecPos { x: 0, y: 0 },
//     };

//     btns.reinit_from_ron(&ron);

//     let btn_dec_label = &btns.btn_dec_label;
//     let x = btns.btn_dec_pos.x;
//     let y = btns.btn_dec_pos.y;
//     println!("btn_dec_label: {}", btn_dec_label);
//     println!("btn_dec_pos.x: {}", x);
//     println!("btn_dec_pos.x: {}", y);
//     println!(" ");

//     let (tx, rx) = std::sync::mpsc::channel();

//     let _ron = ron.clone();
//     let _btns = btns.clone();
//     // no need Arc<Mutex>, only clone Sender<Btns>. unless spawning Arc<Mutex<Receiver<Btns>>>.
//     let _tx = tx.clone();

//     thread::spawn(move || -> anyhow::Result<()> {
//         let tx = _tx;
//         let mut btns = _btns;
//         let ron = _ron;
//         while btns.reload_from_ron(&ron, 0.6)? {
//             let btns_ = btns.clone();
//             tx.send(btns_)?;
//         }
//         Ok(())
//     });

//     // timeout for a while if nothing changed in ron file, so that the program won't freeze.
//     let timeout = Duration::from_secs_f32(0.05);

//     // Mutate in event loop.
//     loop {
//         if let Ok(btns_) = rx.recv_timeout(timeout) {
//             btns = btns_;
//             let btn_dec_label = btns.btn_dec_label;
//             let x = btns.btn_dec_pos.x;
//             let y = btns.btn_dec_pos.y;
//             println!("btn_dec_label: {}", btn_dec_label);
//             println!("btn_dec_pos.x: {}", x);
//             println!("btn_dec_pos.x: {}", y);
//             println!(" ");
//         }
//         // println!("continue other logic...");
//         // request redraw etc.
//     }
// }
