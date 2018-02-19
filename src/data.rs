#[derive(Clone,Debug,PartialEq)]
pub enum Request {
  Raw(Vec<u8>),
  Req1(Req1),
  //Req2(Req2),
}

#[derive(Clone,Debug,PartialEq)]
pub struct Req1 {
  pub request_line: Vec<u8>,
  pub headers:      Vec<(Vec<u8>, Vec<u8>)>,
  pub body:         Vec<u8>,
}

#[derive(Clone,Debug,PartialEq)]
pub struct Req2 {
  pub method:  Vec<u8>,
  pub path:    Vec<u8>,
  pub version: u8,
  pub host:    Vec<u8>,
  pub headers: Vec<(Vec<u8>, Vec<u8>)>,
  pub body:    Vec<u8>,
}

impl Request {
  pub fn serialize(&self) -> Vec<u8> {
    match self {
      &Request::Raw(ref v) => v.clone(),
      &Request::Req1(ref r) => r.serialize(),
    }
  }
}

impl Req1 {
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend(&self.request_line);
    v.extend(b"\r\n");
    for h in self.headers.iter() {
      v.extend(&h.0);
      v.extend(b": ");
      v.extend(&h.1);
      v.extend(b"\r\n");
    }
    v.extend(b"\r\n");
    v.extend(&self.body);

    v
  }
}

#[derive(Clone,Debug,PartialEq)]
pub enum Response {
  Raw(Vec<u8>),
  Res1(Res1),
}

#[derive(Clone,Debug,PartialEq)]
pub struct Res1 {
  pub status_line:  Vec<u8>,
  pub headers:      Vec<(Vec<u8>, Vec<u8>)>,
  pub body:         Vec<u8>,
}

impl Response {
  pub fn serialize(&self) -> Vec<u8> {
    match self {
      &Response::Raw(ref v) => v.clone(),
      &Response::Res1(ref r) => r.serialize(),
    }
  }
}

impl Res1 {
  pub fn serialize(&self) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend(&self.status_line);
    v.extend(b"\r\n");
    for h in self.headers.iter() {
      v.extend(&h.0);
      v.extend(b": ");
      v.extend(&h.1);
      v.extend(b"\r\n");
    }
    v.extend(b"\r\n");
    v.extend(&self.body);

    v
  }
}
