# dynamixel.rs

> Interface for Robotis Dynamixel servos in Rust

## Features
This library is currently in development but is aiming to become a full featured dynamixel library in Rust. It should give a good user experience when used without the `std` library and be extended with nice features when `std` is used. It currently got the following features:
 - Type safe read/write register for protocol 1 and protocol 2 (If you try to write to a read only register your program will not compile)
 - Very basic support for MX28 servo
 - Very basic support for M42 servo
 - Enumeration of servos (when used with `std`)
 - A generic servo trait that allows you to treat all servos the same (can be used as a Boxed trait with `std`)
 
 ### `std`/`no_ std`
 - The `std` feature is not enabled by default, if you're using the `std` library you should enable this feature.
 - If you also enable the `serialport` feature `Interface` will be implemented by `Box<SerialPort>` from [serialport](https://crates.io/crates/serialport/2.0.0)

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
