use fltk::{app::*, button::*, frame::*, window::*};
use quadoculars::LiveJson;
use serde::Deserialize;
use std::{env, path::PathBuf, thread};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Increment,
    Decrement,
}

#[derive(Debug, Deserialize)]
struct Btns {
    btn_inc_label: String,
    btn_inc_pos: Btnspos,
    btn_dec_label: String,
    btn_dec_pos: Btnspos,
}

impl Btns {
    fn init() -> Self {
        Self {
            btn_inc_label: String::new(),
            btn_inc_pos: Btnspos::init(),
            btn_dec_label: String::new(),
            btn_dec_pos: Btnspos::init(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Btnspos {
    x: i32,
    y: i32,
}

impl Btnspos {
    fn init() -> Self {
        Self { x: 0, y: 0 }
    }
}

// Implement LiveReload trait just like so:
impl LiveJson for Btns {}

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

    {
        let mut wind = wind.clone();
        let btn_dec_lr = btn_dec.clone();
        let btn_inc_lr = btn_inc.clone();
        let frame = frame.clone();
        thread::spawn(move || -> anyhow::Result<()> {
            let mut btn_dec = btn_dec_lr;
            let mut btn_inc = btn_inc_lr;

            let mut btns = Btns::init();
            let mut json = PathBuf::new();
            {
                if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
                    json = PathBuf::from(_pth);
                }
                json.push("src\\json\\Btns.json");
                // Reload init json file (optional)
                btns.reinit_from_json(&json);
            }

            // Reset all if function reload_init called.
            btn_inc.set_label(&format!("{} +", btns.btn_inc_label));
            btn_inc.set_pos(btns.btn_inc_pos.x, btns.btn_inc_pos.y);
            btn_dec.set_label(&format!("{} -", btns.btn_dec_label));
            btn_dec.set_pos(btns.btn_dec_pos.x, btns.btn_dec_pos.y);
            // redraw widgets
            wind.redraw();

            // Start mutate here
            // or 'retry: while let Ok(watching) = btns.reload_from_json(&json) {...
            while let Ok(watching) = btns.reload_from_json(&json, 0.6) {
                // Set all while mutating.
                if watching {
                    btn_inc.set_label(&format!("{} +", btns.btn_inc_label));
                    btn_inc.set_pos(btns.btn_inc_pos.x, btns.btn_inc_pos.y);
                    btn_dec.set_label(&format!("{} -", btns.btn_dec_label));
                    btn_dec.set_pos(btns.btn_dec_pos.x, btns.btn_dec_pos.y);
                    wind.redraw();
                } else {
                    // Return to default position if button no longer mutate (usually caused by removing/renaming the json file).
                    btn_inc.above_of(&frame, 0);
                    btn_dec.below_of(&frame, 0);
                    wind.redraw();
                    // either continue 'retry; and do something else or break the loop, break the loop here for the sake of simple demo example.
                    break;
                }
            }
            Ok(())
        });
    }

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

    app.run()?;
    Ok(())
}

//// Without Deserialize struct, using fn mutate_json_val directly.
// use fltk::{app::*, button::*, frame::*, window::*};
// use oculars::Watch;
// use std::{env, path::PathBuf};

// #[derive(Debug, Clone, Copy)]
// pub enum Message {
//     Increment,
//     Decrement,
// }

// fn main() -> anyhow::Result<()> {
//     let app = App::default();
//     let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
//     let mut frame = Frame::default()
//         .with_size(100, 40)
//         .center_of(&wind)
//         .with_label("0");
//     frame.set_label_size(20);
//     let mut btn_inc = Button::default()
//         .size_of(&frame)
//         .with_label("+")
//         .above_of(&frame, 0);
//     let mut btn_dec = Button::default()
//         .size_of(&frame)
//         .with_label("-")
//         .below_of(&frame, 0);
//     wind.end();
//     wind.show();

//     let mut frame1 = frame.clone();
//     btn_inc.set_callback(move || {
//         let label = (frame1.label().parse::<i32>().unwrap() + 1).to_string();
//         frame1.set_label(&label);
//     });

//     let mut frame1 = frame.clone();
//     btn_dec.set_callback(move || {
//         let label = (frame1.label().parse::<i32>().unwrap() - 1).to_string();
//         frame1.set_label(&label);
//     });

//     let mut json = PathBuf::new();
//     {
//         if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
//             json = PathBuf::from(_pth);
//         }
//         json.push("src\\json\\Btns.json");
//     }

//     let mut val = serde_json::json!({});

//     while app.wait() {
//         // Start mutate here
//         Watch::new().mutate_json_val(&json, &mut val, 0.07);

//         let btn_inc_label = val["btn_inc_label"].as_str().unwrap();
//         let x = val["btn_inc_pos"].as_object().unwrap()["x"]
//             .as_i64()
//             .unwrap() as i32;
//         let y = val["btn_inc_pos"].as_object().unwrap()["y"]
//             .as_i64()
//             .unwrap() as i32;
//         btn_inc.set_pos(x, y);
//         btn_inc.set_label(&format!("{} -", btn_inc_label));

//         let btn_dec_label = val["btn_dec_label"].as_str().unwrap();
//         let x = val["btn_dec_pos"].as_object().unwrap()["x"]
//             .as_i64()
//             .unwrap() as i32;
//         let y = val["btn_dec_pos"].as_object().unwrap()["y"]
//             .as_i64()
//             .unwrap() as i32;
//         btn_dec.set_pos(x, y);
//         btn_dec.set_label(&format!("{} -", btn_dec_label));

//         wind.redraw();
//     }
//     Ok(())
// }
