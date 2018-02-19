# Proxy test suite

this tool is meant to provide an easy way to test and reproduce proxy bugs.

Every test case must implement the following trait:

```rust
pub trait Check {
  fn generate_request(&self) -> Vec<u8>;
  fn check_request(&self, req: &httparse::Request) -> Result<(), String>;
  fn generate_response(&self) -> Vec<u8>;
  fn check_response(&self, res: &httparse::Response) -> Result<(), String>;
}
```

The trait will allow us to make non standard compliant request and responses.

The tool will:

- set up a HTTP server (right now, hardcoded to address `127.0.0.1:1026`)
- set up a HTTP client
- the client will send the request to the proxy (right now, hardcoded to address `127.0.0.1:8080`)
- the server will parse it with httparse (we're explicitely not using sozu's parser to find behaviour differences)
- the server will verify that the received request matches the one we want (it's expected that the proxy adds or remove some headers)
- the server sends the response
- the client will parse the response with httparse
- the client will verify that the received response matche the one we want

## Planned ideas

- find a nice way to generically write test cases
- execute all of the test cases, or a specific one if we want
- add some random behaviour, that could be fixed with a seed:
  - insert spaces in the headers
  - reorder the headers
  - split the writes to the sockets to test partial parsing or chunking
  - split the body in various chunks
