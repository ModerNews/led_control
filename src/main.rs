#[macro_use]
extern crate rocket;

mod commands;
mod rest_api;
use crate::commands::commmands::{Commands, Strip};
use crate::rest_api::rest_api::rocket;
//use async_std::task::block_on;
//use futures::future::join;

#[tokio::main]
async fn main() {
    let _ = tokio::join!(rocket().launch(), async_call());
    // join(async_call()).await;
}

async fn async_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut bed_strip = Strip::new("192.168.0.212:5577".into(), None).await;
    let mut desk_strip = Strip::new("192.168.0.125:5577".into(), Some(true)).await;
    let command = Commands::GetStatus;
    let data = bed_strip.execute(&command).await;
    println!("Desk strip status: {:?}", data);
    let data = desk_strip.execute(&command).await;
    println!("Bed strip status: {:?}", data);
    Ok(())
}
