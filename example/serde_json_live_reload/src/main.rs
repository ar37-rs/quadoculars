use quadoculars::Watch;
use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let mut json = PathBuf::new();
    {
        if let Ok(_pth) = env::var("CARGO_MANIFEST_DIR") {
            json = PathBuf::from(_pth);
        }
        json.push("src\\json\\Btns.json");
    }

    let watch = Watch::new().set_timeout(0.6);
    let mut val = serde_json::json!({});
    watch.json_val_init(&json, &mut val);

    let btn_dec_label = val["btn_dec_label"].as_str().unwrap();
    let x = val["btn_dec_pos"].as_object().unwrap()["x"]
        .as_i64()
        .unwrap();
    let y = val["btn_dec_pos"].as_object().unwrap()["y"]
        .as_i64()
        .unwrap();
    println!("btn_dec_label: {}", btn_dec_label);
    println!("btn_dec_pos.x: {}", x);
    println!("btn_dec_pos.x: {}", y);
    println!(" ");

    while let Ok(watching) = watch.json_val(&json, &mut val) {
        if watching {
            let btn_dec_label = val["btn_dec_label"].as_str().unwrap();
            let x = val["btn_dec_pos"].as_object().unwrap()["x"]
                .as_i64()
                .unwrap();
            let y = val["btn_dec_pos"].as_object().unwrap()["y"]
                .as_i64()
                .unwrap();
            println!("btn_dec_label: {}", btn_dec_label);
            println!("btn_dec_pos.x: {}", x);
            println!("btn_dec_pos.x: {}", y);
            println!(" ");
        } else {
            println!("latest captured value:");
            let btn_dec_label = val["btn_dec_label"].as_str().unwrap();
            let x = val["btn_dec_pos"].as_object().unwrap()["x"]
                .as_i64()
                .unwrap();
            let y = val["btn_dec_pos"].as_object().unwrap()["y"]
                .as_i64()
                .unwrap();
            println!("btn_dec_label: {}", btn_dec_label);
            println!("btn_dec_pos.x: {}", x);
            println!("btn_dec_pos.x: {}", y);
            break;
        }
    }

    Ok(())
}
