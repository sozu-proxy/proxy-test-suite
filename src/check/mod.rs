use data::*;
use httparse;
use std::collections::{HashMap,HashSet};
use std::sync::{Arc,Mutex};
use std::io::{Read, Write, ErrorKind,BufReader};
use nom::HexDisplay;
use std::time::Duration;
use tokio_core::net::{TcpListener,TcpStream,Incoming};
use tokio_core::reactor::{Core,Handle};
use futures::{Future,Stream};
use futures::stream;
use futures::IntoFuture;
use tokio_io::io;
use std::net::SocketAddr;

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

pub enum CheckType {
  Success,
  RequestFailure,
  ResponseFailure,
}

pub use self::success::Check1;
pub use self::request_failure::Check2;
pub use self::response_failure::Check3;


pub struct Runner {
  checks: HashMap<usize, Arc<Mutex<Check>>>,
}

pub fn create() -> Arc<Mutex<Check>> {
  Arc::new(Mutex::new(Check1::new()))
}

pub fn run_all_checks() {
  let c1 = Arc::new(Mutex::new(Check1::new()));
  let c2 = Arc::new(Mutex::new(Check2::new()));
  let c3 = Arc::new(Mutex::new(Check3::new()));

  let mut core = Core::new().unwrap();
  let h0 = core.handle();
  let h1 = core.handle();
  let h2 = core.handle();
  let h3 = core.handle();

  println!("launching listener");
  let listener_addr = "127.0.0.1:1026".parse().unwrap();
  let listener = TcpListener::bind(&listener_addr, &h0).unwrap();

  core.run(run_success(listener.incoming(), h1, c1).and_then(|listener| {
    run_request_failure(listener, h2, c2)
  }).and_then(|listener| {
    run_response_failure(listener, h3, c3)
  })).unwrap();
}

pub fn run_check(listener: Incoming, handle: Handle, c1: Arc<Mutex<Check>>) -> impl Future<Item = Incoming, Error = ()> {
  let (req_success, res_success) = {
    let c = c1.lock().unwrap();
    (c.expects_request_success(), c.expects_response_success())
  };

  if req_success && res_success {
    Box::new(run_success(listener, handle, c1)) as Box<Future<Item = Incoming, Error = ()>>
  } else if !req_success {
    Box::new(run_request_failure(listener, handle, c1)) as Box<Future<Item = Incoming, Error = ()>>
  } else if req_success && ! res_success {
    Box::new(run_response_failure(listener, handle, c1)) as Box<Future<Item = Incoming, Error = ()>>
  } else {
    panic!()
  }
}

impl Runner {
  pub fn new() -> Runner {
    let mut checks = HashMap::new();
    checks.insert(0,  self::success::Check1::create());
    checks.insert(1,  self::request_failure::Check2::create());
    checks.insert(2,  self::response_failure::Check3::create());

    Runner {
      checks
    }
  }

  pub fn run(&self) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let h1 = core.handle();
    let h2 = core.handle();
    let h3 = core.handle();

    let it = self.checks.values();

    println!("launching listener");
    let listener_addr = "127.0.0.1:1026".parse().unwrap();
    let listener = TcpListener::bind(&listener_addr, &handle.clone()).unwrap();


    let f = stream::iter_ok(it).fold((listener.incoming(), handle),
      |(listener, handle):(Incoming, Handle) , check: &Arc<Mutex<Check>>| {
      //|(listener, handle) , check| {
      let h: Handle = handle.clone();
      run_check(listener, h, check.clone()).map(|listener| (listener, handle))
    }).map_err(|e: ()| {
      println!("got error: {:?}",e);
      ()
    });

    let a: (Incoming, Handle) = core.run(f).unwrap();
  }
  /*
  pub fn create_check(&self, check_type: CheckType, id: usize) -> Arc<Mutex<Check>> {
    match (check_type, id) {
      (CheckType::Success, 0) => Arc::new(Mutex::new(Check1::new())),
      (CheckType::ResponseFailure, 0) => Arc::new(Mutex::new(self::response_failure::Check3::new())),
      _ => panic!(),
    }
  }
  */

  /*
  pub fn run(&self, id: usize) {// -> ScopedJoinHandle<()> {
    if let Some(ref check) = self.checks.get(&id) {
      let res = {
        let c = check.lock().unwrap();
        !c.expects_request_success() && !c.expects_response_success()
      };

      //if !check.expects_request_success() && !check.expects_response_success() {
      if res {
        return run_request_failure(*check.clone());
      }
    }

    panic!();
  }
    */
}

use std::convert::{AsMut,AsRef};

pub struct Buffer {
  pub vec: Vec<u8>,
  pub index: usize,
}

impl Buffer {
  pub fn new(sz: usize) -> Buffer {
    Buffer {
      vec:   vec![0; sz],
      index: 0,
    }
  }


  pub fn advance(&mut self, sz: usize) {
    self.index += sz;
    assert!(self.index < self.vec.len());
  }
}

impl AsMut<[u8]> for Buffer {
  fn as_mut(&mut self) -> &mut [u8] {
    &mut self.vec[self.index..]
  }
}

impl AsRef<[u8]> for Buffer {
  fn as_ref(&self) -> &[u8] {
    &self.vec[..self.index]
  }
}



use futures::future::{Loop,loop_fn};

/*
pub fn r1() {
  run_success(Arc::new(Mutex::new(self::success::Check1::new())));
}
*/

pub fn run_success(listener: Incoming, handle: Handle, c1: Arc<Mutex<Check>>) -> impl Future<Item = Incoming, Error = ()> {
  let c2 = c1.clone();
  let c3 = c1.clone();

  let server = listener.into_future().and_then(move |(opt_stream, listener)| {
    let (tcp, addr) = opt_stream.expect("could not accept listener");
    let buf = Buffer::new(16384);

    loop_fn((tcp, buf, c1), |(tcp, buf, c1)| {
      io::read(tcp, buf).and_then(|(tcp, mut buf, sz)| {
        {
          buf.advance(sz);
          let mut headers = [httparse::EMPTY_HEADER; 16];
          let mut req = httparse::Request::new(&mut headers);
          let status = req.parse(buf.as_ref()).unwrap();
          if status.is_complete() {
            println!("received request:\n{:#?}", req);
            {
              let checker = c1.lock().unwrap();
              checker.check_request(&req).unwrap();
            }

            return Ok(Loop::Break(tcp));
          }
        }
        Ok(Loop::Continue((tcp, buf, c1)))
      })
    }).and_then(move |stream| {
      let buffer: Vec<u8> = {
        let mut checker = c2.lock().unwrap();
        checker.generate_response()
      };

      io::write_all(stream, buffer)
    }).then(|r| {
      println!("got result: {:?}", r);
      Ok(listener)
    })
  });

  let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
  let buffer = {
    let mut checker = c3.lock().unwrap();
    checker.generate_request()
  };
  let client = TcpStream::connect(&addr, &handle).and_then(move |tcp| {

    tcp.set_keepalive(Some(Duration::from_millis(1)));

    io::write_all(tcp, buffer)
  }).and_then(move |(tcp, _)| {
    let buf = Buffer::new(16384);

    loop_fn((tcp, buf, c3), |(tcp, buf, c3)| {
      io::read(tcp, buf).and_then(|(tcp, mut buf, sz)| {
        {
          buf.advance(sz);
          let mut headers = [httparse::EMPTY_HEADER; 16];
          let mut res = httparse::Response::new(&mut headers);
          let status = res.parse(buf.as_ref()).unwrap();
          if status.is_complete() {
            println!("received response:\n{:#?}", res);
            {
              let checker = c3.lock().unwrap();
              checker.check_response(&res).unwrap();
            }

            return Ok(Loop::Break(tcp));
          }
        }
        Ok(Loop::Continue((tcp, buf, c3)))
      })
    })
  });

  server
    .or_else(|(_, listener)| Ok(listener))
    .join(client.map_err(|e| {
      println!("got error: {:?}", e);
      ()
    }))
    .map(|(listener, _)| listener)
}


/*
pub fn r2() {
  run_request_failure(Arc::new(Mutex::new(self::request_failure::Check2::new())));
}
*/

pub fn run_request_failure(listener: Incoming, handle: Handle, c1: Arc<Mutex<Check>>) -> impl Future<Item = Incoming, Error = ()> {

  println!("launching client");

  let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
  let buffer = {
    let mut checker = c1.lock().unwrap();
    checker.generate_request()
  };
  TcpStream::connect(&addr, &handle).and_then(|tcp| {

    tcp.set_keepalive(Some(Duration::from_millis(1)));
    println!("sending:\n{}", &buffer.to_hex(16));
    io::write_all(tcp, buffer).and_then(|(tcp,res)| {
      println!("res: {:?}", res);
      io::flush(tcp)
    }).and_then(|tcp| {
      let buf = vec![42; 10];
      io::read_to_end(tcp, buf)
        .then(|r| {
          match r {
            Ok((tcp, sl)) => {
              let mut buf = vec![42u8; 10];
              if sl == buf {
                println!("buffer not modified, socket probably closed");
                Ok(())
              } else {
                panic!("buffer should not have changed:\n{}", sl.to_hex(16));
              }
            },
            Err(e) => {
              println!("got error: {:?}", e);
              Ok(())
            }
          }
        })
    }).then(|_| Ok(()))
  }).then(|_| Ok(listener))
}

/*
pub fn r3() {
  run_response_failure(Arc::new(Mutex::new(self::response_failure::Check3::new())));
}
*/
pub fn run_response_failure(listener: Incoming, handle: Handle, c1: Arc<Mutex<Check>>) -> impl Future<Item = Incoming, Error = ()> {

  let c2 = c1.clone();
  let c3 = c1.clone();

  /*
  let mut core = Core::new().unwrap();
  let handle = core.handle();

  println!("launching listener");
  let listener_addr = "127.0.0.1:1026".parse().unwrap();
  let listener = TcpListener::bind(&listener_addr, &handle).unwrap();
  */

  let server = listener.into_future().and_then(|(opt_stream, listener)| {
    let (tcp, addr) = opt_stream.expect("could not accept listener");
    let buf = Buffer::new(16384);

    loop_fn((tcp, buf, c1), |(tcp, buf, c1)| {
      io::read(tcp, buf).and_then(|(tcp, mut buf, sz)| {
        {
          buf.advance(sz);
          let mut headers = [httparse::EMPTY_HEADER; 16];
          let mut req = httparse::Request::new(&mut headers);
          let status = req.parse(buf.as_ref()).unwrap();
          if status.is_complete() {
            println!("received request:\n{:#?}", req);
            {
              let checker = c1.lock().unwrap();
              checker.check_request(&req).unwrap();
            }

            return Ok(Loop::Break(tcp));
          }
        }
        Ok(Loop::Continue((tcp, buf, c1)))
      })
    }).and_then(move |stream| {
      let buffer: Vec<u8> = {
        let mut checker = c2.lock().unwrap();
        checker.generate_response()
      };

      io::write_all(stream, buffer)
    }).and_then(|(stream, _)| {
      let buf = vec![42; 10];
      io::read_to_end(stream, buf).then(|r| {
        match r {
          Ok((_, sl)) => {
            let mut buf = vec![42u8; 10];
            if sl == buf {
              println!("buffer not modified, socket probably closed");
              Ok(())
            } else {
              panic!("buffer should not have changed:\n{}", sl.to_hex(16));
            }
          },
          Err(e) => {
            println!("got error: {:?}", e);
            Ok(())
          }
        }
      })
    }).then(|r| {
      println!("got result: {:?}", r);
      Ok(listener)
    })
  });

  let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
  let buffer = {
    let mut checker = c3.lock().unwrap();
    checker.generate_request()
  };
  let client = TcpStream::connect(&addr, &handle).and_then(|tcp| {

    tcp.set_keepalive(Some(Duration::from_millis(1)));

    io::write_all(tcp, buffer)
  }).and_then(|(tcp, _)| {
      let buf = vec![42; 10];
      io::read_to_end(tcp, buf).then(|r| {
        match r {
          Ok((_, sl)) => {
            let mut buf = vec![42u8; 10];
            if sl == buf {
              println!("buffer not modified, socket probably closed");
              Ok(())
            } else {
              panic!("buffer should not have changed:\n{}", sl.to_hex(16));
            }
          },
          Err(e) => {
            println!("got error: {:?}", e);
            Ok(())
          }
        }
      })
  });

  server
    .or_else(|(_, listener)| Ok(listener))
    .join(client.map_err(|e| {
      println!("got error: {:?}", e);
      ()
    }))
    .map(|(listener, _)| listener)
  //core.run(server.or_else(|(_, listener)| Ok(listener)).join(client).map(|(listener, _)| listener)).unwrap();
}
