use httparse;
use std::sync::{Arc,Mutex};

use super::Check;

pub struct Check2 {
}

impl Check2 {
  pub fn new() -> Check2 {
    Check2 {}
  }

  pub fn create() -> Arc<Mutex<Check>> {
    Arc::new(Mutex::new(Self::new()))
  }
}

impl Check for Check2 {
  fn generate_request(&self) -> Vec<u8> {
    Vec::from("GET\r\n")
  }

  fn expects_request_success(&self) -> bool {
    false
  }

  fn check_request(&self, req: &httparse::Request) -> Result<(), String> {
    Err("request should have failed".to_string())
  }

  fn generate_response(&self) -> Vec<u8> {
    Vec::new()
  }

  fn expects_response_success(&self) -> bool {
    false
  }

  fn check_response(&self, res: &httparse::Response) -> Result<(), String> {
    Err("request should have failed".to_string())
  }

}

