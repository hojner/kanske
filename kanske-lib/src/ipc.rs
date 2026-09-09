//! Shared text protocol for the `kanske` <-> `kanskectl` control socket.
//!
//! One request/response per connection: the client writes a single line,
//! the daemon writes a single line back, and the connection is closed.

use crate::error::KanskeError;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// Report the currently applied profile and connected head count.
    Status,
    /// Apply the named profile, regardless of whether it matches the
    /// currently connected heads.
    Switch(String),
}

impl Request {
    pub fn to_line(&self) -> String {
        match self {
            Request::Status => "STATUS".to_string(),
            Request::Switch(name) => format!("SWITCH {}", name),
        }
    }

    pub fn from_line(line: &str) -> Result<Self, KanskeError> {
        let line = line.trim();
        if line == "STATUS" {
            return Ok(Request::Status);
        }
        if let Some(name) = line.strip_prefix("SWITCH ") {
            let name = name.trim();
            if name.is_empty() {
                return Err(KanskeError::IpcError(
                    "SWITCH requires a profile name".to_string(),
                ));
            }
            return Ok(Request::Switch(name.to_string()));
        }
        Err(KanskeError::IpcError(format!(
            "Unrecognized request: '{}'",
            line
        )))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Ok(String),
    Err(String),
}

impl Response {
    pub fn to_line(&self) -> String {
        match self {
            Response::Ok(msg) => format!("OK {}", msg),
            Response::Err(msg) => format!("ERR {}", msg),
        }
    }

    pub fn from_line(line: &str) -> Result<Self, KanskeError> {
        let line = line.trim();
        if let Some(msg) = line.strip_prefix("OK ") {
            return Ok(Response::Ok(msg.to_string()));
        }
        if line == "OK" {
            return Ok(Response::Ok(String::new()));
        }
        if let Some(msg) = line.strip_prefix("ERR ") {
            return Ok(Response::Err(msg.to_string()));
        }
        if line == "ERR" {
            return Ok(Response::Err(String::new()));
        }
        Err(KanskeError::IpcError(format!(
            "Unrecognized response: '{}'",
            line
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_status_roundtrip() {
        let req = Request::Status;
        assert_eq!(req.to_line(), "STATUS");
        assert_eq!(Request::from_line("STATUS").unwrap(), Request::Status);
    }

    #[test]
    fn test_request_switch_roundtrip() {
        let req = Request::Switch("docked".to_string());
        assert_eq!(req.to_line(), "SWITCH docked");
        assert_eq!(
            Request::from_line("SWITCH docked").unwrap(),
            Request::Switch("docked".to_string())
        );
    }

    #[test]
    fn test_request_switch_missing_name() {
        assert!(Request::from_line("SWITCH").is_err());
        assert!(Request::from_line("SWITCH ").is_err());
    }

    #[test]
    fn test_request_unrecognized() {
        assert!(Request::from_line("BOGUS").is_err());
    }

    #[test]
    fn test_response_ok_roundtrip() {
        let resp = Response::Ok("profile=docked heads=2".to_string());
        assert_eq!(resp.to_line(), "OK profile=docked heads=2");
        assert_eq!(
            Response::from_line("OK profile=docked heads=2").unwrap(),
            resp
        );
    }

    #[test]
    fn test_response_err_roundtrip() {
        let resp = Response::Err("Profile 'x' not found".to_string());
        assert_eq!(resp.to_line(), "ERR Profile 'x' not found");
        assert_eq!(
            Response::from_line("ERR Profile 'x' not found").unwrap(),
            resp
        );
    }

    #[test]
    fn test_response_unrecognized() {
        assert!(Response::from_line("WAT").is_err());
    }
}
