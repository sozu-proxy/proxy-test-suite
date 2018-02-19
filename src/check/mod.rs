use data::*;
use httparse;
use std::collections::HashSet;
use may::coroutine::JoinHandle;
use std::sync::{Arc,Mutex};
use may::net::TcpListener;
use std::io::{Read, Write, ErrorKind};
use may::net::TcpStream;
use nom::HexDisplay;
use std::time::Duration;
use may::coroutine;

mod success;
mod request_failure;
mod response_failure;

pub trait Check {
  fn generate_request(&self) -> Vec<u8>;
  fn expects_request_success(&self) -> bool;
  fn check_request(&self, req: &httparse::Request) -> Result<(), String>;
  fn generate_response(&self) -> Vec<u8>;
  fn expects_response_success(&self) -> bool;
  fn check_response(&self, res: &httparse::Response) -> Result<(), String>;
}

pub use self::success::Check1;
pub use self::request_failure::Check2;
pub use self::response_failure::Check3;

pub fn run_success() -> JoinHandle<()> {
  let c1 = Arc::new(Mutex::new(Check1::new()));
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
  go!(move || {
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
  })
}

pub fn run_request_failure() -> JoinHandle<()> {
  let c1 = Arc::new(Mutex::new(Check2{}));

  println!("launching client");
  go!(move || {
    let mut tcp = TcpStream::connect("127.0.0.1:8080").unwrap();
    tcp.set_nodelay(true);

    let buffer = {
      let mut checker = c1.lock().unwrap();
      checker.generate_request()
    };

    //tcp.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    println!("sending:\n{}", &buffer.to_hex(16));
    tcp.write_all(&buffer).unwrap();

    let mut counter = 0usize;
    loop {
      let mut buf = vec![0; 10];
      let sz = tcp.read(&mut buf).unwrap();
      println!("[{}] read {} bytes", counter, sz);
      coroutine::sleep(Duration::from_millis(500));
      counter += 1;
      match tcp.write_all(&b"hello"[..]) {
        Err(ref e) if e.kind() == ErrorKind::BrokenPipe => {
          break;
        },
        _ => {}
      }
    }

    println!("got error: {:?}", tcp.take_error().unwrap().unwrap());
    println!("END");
  })
}

pub fn run_response_failure() -> JoinHandle<()> {
  let c1 = Arc::new(Mutex::new(Check3::new()));
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
          println!("sending:\n{:?}", &buffer.to_hex(16));
          stream.write_all(&buffer).unwrap();

          break;
        }
      }

      let mut counter = 0usize;
      loop {
        let mut buf = vec![0; 10];
        let sz = stream.read(&mut buf).unwrap();
        println!("[{}] read {} bytes", counter, sz);
        coroutine::sleep(Duration::from_millis(500));
        counter += 1;
        match stream.write_all(&b"hello"[..]) {
          Err(ref e) if e.kind() == ErrorKind::BrokenPipe => {
            break;
          },
          _ => {}
        }
      }

      println!("got error: {:?}", stream.take_error().unwrap().unwrap());
      println!("END");

    }
  });

  println!("launching client");
  go!(move || {
    let mut tcp = TcpStream::connect("127.0.0.1:8080").unwrap();
    let buffer = {
      let mut checker = c2.lock().unwrap();
      checker.generate_request()
    };
    println!("sending:\n{}", &buffer.to_hex(16));
    tcp.write_all(&buffer).unwrap();

    let mut buf = vec![0; 16384];
    let mut index = 0usize;

    let mut counter = 0usize;
    loop {
      let mut buf = vec![0; 10];
      let sz = tcp.read(&mut buf).unwrap();
      println!("[{}] read {} bytes", counter, sz);
      coroutine::sleep(Duration::from_millis(500));
      counter += 1;
      match tcp.write_all(&b"hello"[..]) {
        Err(ref e) if e.kind() == ErrorKind::BrokenPipe => {
          break;
        },
        _ => {}
      }
    }

    println!("got error: {:?}", tcp.take_error().unwrap().unwrap());
    println!("END");

  })
}
