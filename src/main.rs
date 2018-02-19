#[macro_use]
extern crate may;
extern crate httparse;
extern crate nom;

use may::net::TcpListener;
use std::io::{Read, Write};
use std::sync::{Arc,Mutex};
use may::net::TcpStream;
use nom::HexDisplay;

mod data;
mod check;

use check::*;

fn main() {
  run_success().join().unwrap();
  run_request_failure().join().unwrap();
  run_response_failure().join().unwrap();
}
