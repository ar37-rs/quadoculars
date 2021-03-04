use fltk::{app::*, button::*, frame::*, window::*};
use quadoculars::LiveRon;
use serde::Deserialize;
use std::{env, path::PathBuf, thread, time::Duration, sync::mpsc::channel};
// std channel can be replaced alternatively either with flume channel or crossbeam channel,
// set dependencies:

// quadoculars = { version = "*", features = ["live_ron", "crossbeam_channel"] }
// use crossbeam_channel::{bounded, Sender};

// or

// quadoculars = { version = "*", features = ["live_ron", "flume_channel"] }
// use flume::{bounded, Sender};

#[derive(Debug, Clone, Deserialize)]
struct Btns {
    btn_inc_label: String,
    btn_inc_pos: BtnIncPos,
    btn_dec_label: String,
    btn_dec_pos: BtnDecPos,
}

#[derive(Debug, Clone, Deserialize)]
struct BtnIncPos {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct BtnDecPos {
    x: i32,
    y: i32,
}

// Implement LiveRon
impl LiveRon for Btns {}

fn main() -> anyhow::Result<()> {
    let app = App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut frame = Frame::default()
        .with_size(100, 40)
        .center_of(&wind)
        .with_label("0");
    frame.set_label_size(20);
    let mut btn_inc = Button::default()
        .size_of(&frame)
        .with_label("+")
        .above_of(&frame, 0);
    let mut btn_dec = Button::default()
        .size_of(&frame)
        .with_label("-")
        .below_of(&frame, 0);
    wind.end();
    wind.show();

    let mut frame1 = frame.clone();
    btn_inc.set_callback(move || {
        let label = (frame1.label().parse::<i32>().unwrap() + 1).to_string();
        frame1.set_label(&label);
    });

    let mut frame1 = frame.clone();
    btn_dec.set_callback(move || {
        let label = (frame1.label().parse::<i32>().unwrap() - 1).to_string();
        frame1.set_label(&label);
    });

    let mut ron = PathBuf::new();

    {
        if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
            ron = PathBuf::from(_pth);
        }
        ron.push("src\\ron\\Btns.ron");
    }

    let mut btns: Btns = Btns {
        btn_inc_label: String::from("inc_label"),
        btn_inc_pos: BtnIncPos { x: 0, y: 0 },
        btn_dec_label: String::from("dec_label"),
        btn_dec_pos: BtnDecPos { x: 0, y: 0 },
    };

    btns.reinit_from_ron(&ron);

    btn_inc.set_label(&format!("{}", btns.btn_inc_label));
    btn_inc.set_pos(btns.btn_inc_pos.x, btns.btn_inc_pos.y);

    btn_dec.set_label(&format!("{} -", btns.btn_dec_label));
    btn_dec.set_pos(btns.btn_dec_pos.x, btns.btn_dec_pos.y);

    wind.redraw();

    // let (tx, rx) = bounded(0);
    let (tx, rx) = channel();

    let _ron = ron.clone();
    let _btns = btns.clone();
    // no need Arc<Mutex>, only clone Sender<Btns>. unless spawning Arc<Mutex<Receiver<Btns>>>.
    let _tx = tx.clone();

    thread::spawn(move || -> anyhow::Result<()> {
        let tx = _tx;
        let mut btns = _btns;
        let ron = _ron;
        // while btns.reload_from_ron(&ron, 0.6)? {
        //     let btns_ = btns.clone();
        //     tx.send(btns_)?;
        // }
        while let Ok(watching) = btns.reload_from_ron(&ron, 0.6) {
            if watching {
                let btns_ = btns.clone();
                tx.send(btns_)?;
            } else {
                break;
            }
        }
        Ok(())
    });

    // timeout for a while if nothing changed in ron file, so that the app won't freeze.
    let timeout = Duration::from_secs_f32(0.01);

    while app.wait() {
        // Start mutate here
        if let Ok(btns_) = rx.recv_timeout(timeout) {
            btns = btns_;
            btn_inc.set_label(&format!("{}", btns.btn_inc_label));
            btn_inc.set_pos(btns.btn_inc_pos.x, btns.btn_inc_pos.y);

            btn_dec.set_label(&format!("{} -", btns.btn_dec_label));
            btn_dec.set_pos(btns.btn_dec_pos.x, btns.btn_dec_pos.y);
        }
        // request redraw
        wind.redraw();
    }
    Ok(())
}
