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

use check::Check;

fn main() {

    let c1 = Arc::new(Mutex::new(check::Check1::new()));
    let c2 = c1.clone();

    println!("launching listener");
    let listener = TcpListener::bind("127.0.0.1:1026").unwrap();
    go!(move || {
      while let Ok((mut stream, _)) = listener.accept() {
        let mut buf = vec![0; 16384];
        let mut index = 0usize;

        loop {
          let sz = stream.read(&mut buf[index..]).unwrap();
          index += sz;

          let mut headers = [httparse::EMPTY_HEADER; 16];
          let mut req = httparse::Request::new(&mut headers);
          let status = req.parse(&buf[..index]).unwrap();

          if status.is_complete() {
            println!("received request:\n{:#?}", req);
            let buffer: Vec<u8> = {
              let mut checker = c1.lock().unwrap();
              checker.check_request(&req).unwrap();
              checker.generate_response()
            };
            stream.write_all(&buffer).unwrap();

            break;
          }
        }

      }
    });

    println!("launching client");
    let h = go!(move || {
      let mut tcp = TcpStream::connect("127.0.0.1:8080").unwrap();
      let buffer = {
        let mut checker = c2.lock().unwrap();
        checker.generate_request()
      };
      println!("sending:\n{}", &buffer.to_hex(16));
      tcp.write_all(&buffer).unwrap();

      let mut buf = vec![0; 16384];
      let mut index = 0usize;

      loop {
        let sz = { tcp.read(&mut buf[index..]).unwrap() };
        index += sz;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut res = httparse::Response::new(&mut headers);
        let status = { res.parse(&buf[..index]).unwrap() };

        if status.is_complete() {
          println!("received response:\n{:#?}", res);
          let checker = c2.lock().unwrap();
          checker.check_response(&res).unwrap();
          break;
        }
      }
    });

    h.join().unwrap();
}
