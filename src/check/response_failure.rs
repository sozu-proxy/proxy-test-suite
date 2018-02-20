use data::*;
use httparse;
use std::collections::HashSet;
use std::sync::{Arc,Mutex};

use super::Check;

pub struct Check3 {
  pub req: Req1,
}

impl Check3 {
  pub fn new() -> Check3 {
    Check3 {
      req: Req1 {
        request_line: Vec::from(&b"GET / HTTP/1.1"[..]),
        headers: vec![
          (Vec::from(&b"Host"[..]), Vec::from(&b"lolcatho.st"[..]))
        ],
        body: Vec::new(),
      },
    }
  }

  pub fn create() -> Arc<Mutex<Check>> {
    Arc::new(Mutex::new(Self::new()))
  }
}

impl Check for Check3 {
  fn generate_request(&self) -> Vec<u8> {
    self.req.serialize()
  }

  fn expects_request_success(&self) -> bool {
    true
  }

  fn expects_response_success(&self) -> bool {
    false
  }

  fn check_request(&self, req: &httparse::Request) -> Result<(), String> {
    if req.method != Some("GET") {
      return Err(format!("invalid method: {:?}", req.method));
    }

    if req.path != Some("/") {
      return Err(format!("invalid path: {:?}", req.path));
    }

    if req.version != Some(1) {
      return Err(format!("invalid version: {:?}", req.version));
    }

    let headers: HashSet<(Vec<u8>, Vec<u8>)> = req.headers.iter()
      .map(|h| (Vec::from(h.name.as_bytes()), Vec::from(h.value))).collect();

    let base_headers: HashSet<(Vec<u8>, Vec<u8>)> = self.req.headers.iter().cloned().collect();

    if ! headers.is_superset(&base_headers) {
      Err(format!("invalid headers: {:?}\nbase headers: {:?}", headers, base_headers))
    } else {
      Ok(())
    }
  }

  fn generate_response(&self) -> Vec<u8> {
    Vec::from("BLAH\r\n")
  }

  fn check_response(&self, res: &httparse::Response) -> Result<(), String> {
    Err("response should have failed".to_string())
  }
}
