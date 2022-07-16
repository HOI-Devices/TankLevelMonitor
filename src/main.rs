
extern crate tungstenite;
extern crate url;
extern crate chrono;
extern crate gpio;
extern crate ansi_term;
#[macro_use]
extern crate serde_json;

mod client;
mod gpio_handler;
mod tests;
mod console_logger;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::sync::{Arc, Mutex};
use gpio_handler::GpioHandler;
use client::Client;
use std::thread;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

fn start_gpio_thread(
    pin:u16,
    distance:&Arc<Mutex<i8>>,
    running:&Arc<Mutex<bool>>)-> JoinHandle<()> {
    let distance = Arc::clone(distance);
    let running = Arc::clone(running);
    let mut gpio_handler = GpioHandler::new(pin);
    let handle:JoinHandle<()> = thread::spawn(move || {
        gpio_handler.begin_monitoring(&distance,&running);
    });
    return handle;
}

fn main() {
    let mut client = Client::new(
        "192.168.1.109".to_string(),
        "50223".to_string(),
        "".to_string(),
        "test".to_string(),
        "infared".to_string(),
        "test".to_string());

    let running = Arc::new(Mutex::new(true));
    let distance = Arc::new(Mutex::new(0));
    let mut encountered_error =false;
     
    while true{
        client.begin_monitoring(&mut encountered_error,&distance);
        if encountered_error == false{
            *running.lock().unwrap() = false;
            break;
        }
        thread::sleep(Duration::from_millis(10000));
    }    
}