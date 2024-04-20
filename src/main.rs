#[macro_use]
extern crate rocket;

mod commands;
mod config_utils;
mod macros;
mod rest_api;

use crate::commands::commmands::wake_signal;
use crate::config_utils::configs::Config;
use crate::macros::led_macro::Macro;
use crate::rest_api::rest_api_mod::rocket;

use async_std::task::sleep;
// use futures::future::join;
use lazy_static::lazy_static;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Executing startup tasks...");
    let config = Config::default();
    println!("Config loaded: {:?}", &config);
    let builded_rocket = rocket(config.clone());
    println!("Sending wake signal...");
    let _ = wake_signal().await;
    sleep(Duration::from_secs(5)).await;
    println!("Awaiting response... [5s]");
    let _ = tokio::join!(builded_rocket.launch(), async_call(&config));
    // join(async_call()).await;
}

async fn async_call(config: &Config) {
    let macros = config.macros.clone();
    let macros = macros.iter().map(|x| Macro::new(x, config));
    for macro_ in macros {
        println!("Running macro: {:?}", macro_);
        macro_.run().await;
    }
}

/* async fn async_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut bed_strip = Strip::new("192.168.0.212:5577".into(), None).await;
    let mut desk_strip = Strip::new("192.168.0.125:5577".into(), Some(true)).await;
    let command = Commands::GetStatus;
    let data = bed_strip.execute(&command).await;
    println!("Desk strip status: {:?}", data);
    let data = desk_strip.execute(&command).await;
    println!("Bed strip status: {:?}", data);
    Ok(())
} */
