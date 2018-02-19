use data::*;
use httparse;
use std::collections::HashSet;

pub trait Check {
  fn generate_request(&self) -> Vec<u8>;
  fn check_request(&self, req: &httparse::Request) -> Result<(), String>;
  fn generate_response(&self) -> Vec<u8>;
  fn check_response(&self, res: &httparse::Response) -> Result<(), String>;
}

pub struct Check1 {
  pub req: Req1,
  pub res: Res1,
}

impl Check1 {
  pub fn new() -> Check1 {
    Check1 {
      req: Req1 {
        request_line: Vec::from(&b"GET / HTTP/1.1"[..]),
        headers: vec![
          (Vec::from(&b"Host"[..]), Vec::from(&b"lolcatho.st"[..]))
        ],
        body: Vec::new(),
      },
      res: Res1 {
        status_line: Vec::from(&b"HTTP/1.1 200 OK"[..]),
        headers: vec![
          (Vec::from(&b"Server"[..]), Vec::from(&b"lolcatho.st"[..]))
        ],
        body: Vec::new(),
      }
    }
  }
}

impl Check for Check1 {
  fn generate_request(&self) -> Vec<u8> {
    self.req.serialize()
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
    self.res.serialize()
  }

  fn check_response(&self, res: &httparse::Response) -> Result<(), String> {
    if res.version != Some(1) {
      return Err(format!("invalid version: {:?}", res.version));
    }

    if res.code != Some(200) {
      return Err(format!("invalid status: {:?}", res.code));
    }

    if res.reason != Some("OK") {
      return Err(format!("invalid path: {:?}", res.reason));
    }

    let headers: HashSet<(Vec<u8>, Vec<u8>)> = res.headers.iter()
      .map(|h| (Vec::from(h.name.as_bytes()), Vec::from(h.value))).collect();

    let base_headers: HashSet<(Vec<u8>, Vec<u8>)> = self.res.headers.iter().cloned().collect();

    if ! headers.is_superset(&base_headers) {
      Err(format!("invalid headers: {:?}\nbase headers: {:?}", headers, base_headers))
    } else {
      Ok(())
    }

  }
}
