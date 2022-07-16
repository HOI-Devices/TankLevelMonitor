use gpio::{GpioIn, GpioOut,GpioValue};
use client::Client;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
pub struct GpioHandler{
    trig_pin : u16,
    echo_pin : u16
}

impl GpioHandler{
    pub fn new(trig: u16,echo : u16)->Self{
        Self{
            trig_pin:trig,
            echo_pin:echo
        }
    }

    pub fn begin_monitoring(
            &self,times_open:&Arc<Mutex<i8>>,
            running:&Arc<Mutex<bool>>,
            distance:&Arc<Mutex<i16>>){

        let mut gpio_echo_pin = gpio::sysfs::SysFsGpioInput::open(self.echo_pin).unwrap();
        let mut gpio_trig_pin = gpio::sysfs::SysFsGpioOutput::open(self.trig_pin).unwrap();
       
        //deference the mutex to get the state of the application
        println!("Sensor Pin({}) Activated!!\n",self.trig_pin);
        println!("Sensor Pin({}) Activated!!\n",self.echo_pin);
        while true{
            //setup ultra sonic HC-SR04
            gpio_trig_pin.set_value(true).unwrap();
            thread::sleep(Duration::from_millis(0.01));
            gpio_trig_pin.set_value(false).unwrap();

            let mut start = Utc::now();
            while gpio_echo_pin.read_value().unwrap() == GpioValue::Low{
                start = Utc::now();
            }

            let end = Utc::now();
            while gpio_echo_pin.read_value().unwrap() == GpioValue::High{
                end = Utc::now();
            }
            
            let pulse_time = end - start;
            let distance_data = distance.lock().unwrap();

            *distance_data =  pulse_time.num_seconds() * 17150;


        }
    }
}