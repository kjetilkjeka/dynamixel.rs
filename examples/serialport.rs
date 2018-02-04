extern crate dynamixel;
extern crate serialport;
extern crate badlog;

use std::io::{
    self
};
use std::str::FromStr;


use dynamixel::BaudRate;
use dynamixel::Interface;

fn main() {
    badlog::init_from_env("LOG_LEVEL");
    
    let ports = serialport::available_ports().unwrap();

    println!("Choose a serial device (0 - {})", ports.len() - 1);
    for (i, port) in ports.iter().enumerate() {
        println!("({}) {:?}", i, port);
    }
    
    let reader = io::stdin();
    let mut buffer = String::new();
    reader.read_line(&mut buffer).ok().unwrap();
    buffer.pop().unwrap(); // remove new-line
    
    let index = usize::from_str(&buffer).unwrap();
    
    let mut serial = serialport::open(&ports[index].port_name).unwrap();
    serial.set_baud_rate(BaudRate::Baud1000000).unwrap();

    let interfaces = dynamixel::enumerate(&mut serial).unwrap();
    println!("Found following servos:");
    for (i, port) in interfaces.iter().enumerate() {
        println!("({}) {:?}", i, port);
    }

    let mut buffer = String::new();
    reader.read_line(&mut buffer).ok().unwrap();
    buffer.pop().unwrap(); // remove new-line
    
    let index = usize::from_str(&buffer).unwrap();
        
    let mut servo = dynamixel::connect(&mut serial, interfaces[index].clone()).unwrap();

    
    let pos = servo.get_position(&mut serial).unwrap();
    let mut target_pos = pos + 3.14/2.0;
    if target_pos >= 3.14 {
        target_pos -= 6.28;
    }
    
    
    servo.set_enable_torque(&mut serial, true).unwrap();
    servo.set_position(&mut serial, target_pos).unwrap();
    
}
