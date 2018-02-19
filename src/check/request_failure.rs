use data::*;
use httparse;
use std::collections::HashSet;

use super::Check;

pub struct Check2 {
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

