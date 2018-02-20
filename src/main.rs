#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate httparse;
extern crate nom;

mod data;
mod check;

use check::*;

fn main() {
  //run_success().join().unwrap();
  //run_request_failure().join().unwrap();
  //run_response_failure().join().unwrap();
  //r1();
  //r2();
  r3();
}
