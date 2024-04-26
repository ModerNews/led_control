#[macro_use]
extern crate rocket;

mod commands;
mod config_utils;
mod macros;
mod rest_api;

use crate::config_utils::configs::Config;
use crate::macros::led_macro::Macro;
use crate::rest_api::rest_api_mod::rocket;
use crate::{commands::commmands::wake_signal, config_utils::configs::Read};

use async_std::task::sleep;
// use futures::future::join;
use lazy_static::lazy_static;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

#[tokio::main]
async fn main() {
    println!("Executing startup tasks...");
    let (stop_signal, _) = broadcast::channel::<()>(1);
    let stop_signal = Arc::new(stop_signal);
    // let config = Config::default();
    // println!("Config loaded: {:?}", &CONFIG);
    let builded_rocket = rocket(&CONFIG);
    println!("Sending wake signal...");
    let _ = wake_signal().await;
    sleep(Duration::from_secs(5)).await;
    println!("Awaiting response... [5s]");
    // task vector contains all the tasks that need to be stopped manually,
    // rocket.launch() has it's own handler built in
    tokio::spawn(builded_rocket.launch());
    let tasks = vec![tokio::spawn(async_call(&CONFIG))];
    tokio::spawn(handle_stop_signal(Arc::clone(&stop_signal)));

    stop_signal.subscribe().recv().await.unwrap();

    println!("SIGINT received: Now stopping gracefully...");
    tokio::join!(handle_shutdown(tasks, Arc::clone(&stop_signal)));
}

async fn handle_shutdown(
    tasks: Vec<tokio::task::JoinHandle<()>>,
    stop_signal: Arc<broadcast::Sender<()>>,
) {
    tokio::spawn(graceful_shutdown(tasks));
    tokio::spawn(handle_stop_signal(Arc::clone(&stop_signal)));

    stop_signal.subscribe().recv().await.unwrap();

    println!("Second SIGINT received: send third to stop forcefully...");

    tokio::spawn(handle_stop_signal(Arc::clone(&stop_signal)));

    stop_signal.subscribe().recv().await.unwrap();

    println!("Third SIGINT received: stopping forcefully...");
    exit(1);
}

async fn graceful_shutdown(tasks: Vec<tokio::task::JoinHandle<()>>) {
    for task in tasks {
        task.abort();
    }
    let _ = &CONFIG.lock().await.write("config.yaml").await;
    exit(0);
}

async fn handle_stop_signal(stop_signal: Arc<broadcast::Sender<()>>) {
    tokio::signal::ctrl_c().await.unwrap();
    let _ = stop_signal.send(());
}

async fn async_call(config: &Mutex<Config>) {
    let config = config.read().await;
    let macros = config.macros.clone();
    let macros = macros.iter().map(|x| Macro::new(x, &config));
    for macro_ in macros {
        println!("Running macro: {:?}", macro_);
        macro_.run().await;
    }
    println!("Now awaiting stop signal...");
    loop {
        sleep(Duration::from_secs(5)).await;
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
