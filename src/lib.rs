use serde::Deserialize;
use serde_json;
use serde_json::error::Error as JsonError;

use std::collections::HashMap;
use std::io::Error as IoError;
use std::net::IpAddr;
use std::process::Command;
use std::str::Utf8Error;

#[derive(Clone, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct LlAddr(String);
impl LlAddr {
    pub fn from_string(mut s: String) -> Self {
        s = s.to_lowercase();
        LlAddr(s)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Deserialize, Debug)]
struct Neighbor {
    dst: IpAddr,
    dev: String,
    lladdr: Option<LlAddr>,
    state: Vec<State>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum State {
    Delay,
    Failed,
    Incomplete,
    Reachable,
    Stale,
}

#[derive(Debug)]
pub enum LookupError {
    IoError(IoError),
    Utf8Error(Utf8Error),
    JsonError(JsonError),
}
impl From<IoError> for LookupError {
    fn from(e: IoError) -> LookupError {
        LookupError::IoError(e)
    }
}
impl From<Utf8Error> for LookupError {
    fn from(e: Utf8Error) -> LookupError {
        LookupError::Utf8Error(e)
    }
}
impl From<JsonError> for LookupError {
    fn from(e: JsonError) -> LookupError {
        LookupError::JsonError(e)
    }
}

#[cfg(target_os = "linux")]
fn neighbors_raw() -> Result<Vec<Neighbor>, LookupError> {
    let json = Command::new("ip")
        .arg("--json")
        .arg("neighbor")
        .stderr(std::process::Stdio::inherit())
        .output()?
        .stdout;
    Ok(serde_json::from_str(std::str::from_utf8(json.as_ref())?)?)
}
#[cfg(target_os = "linux")]
pub fn neighbors() -> Result<HashMap<IpAddr, LlAddr>, LookupError> {
    let neighbors: Vec<Neighbor> = neighbors_raw()?;
    let neighbors_map = neighbors
        .into_iter()
        .filter_map(|n| match n {
            Neighbor {
                lladdr: Some(l), ..
            } => Some((n.dst, l)),
            Neighbor { lladdr: None, .. } => None,
        })
        .collect();
    Ok(neighbors_map)
}
#[cfg(target_os = "linux")]
pub fn lookup(ip_addr: IpAddr) -> Result<Option<LlAddr>, LookupError> {
    let ip_addr = normalize_ip_addr(ip_addr);
    let neighbors: Vec<Neighbor> = neighbors_raw()?;
    let lladdr = neighbors
        .into_iter()
        .filter(|n| n.dst == ip_addr)
        .filter_map(|n| n.lladdr)
        .nth(0);
    Ok(lladdr)
}

fn normalize_ip_addr(ip_addr: IpAddr) -> IpAddr {
    match ip_addr {
        IpAddr::V4(_) => ip_addr,
        IpAddr::V6(v6) => match v6.to_ipv4() {
            Some(converted_v4) => IpAddr::V4(converted_v4),
            None => ip_addr,
        },
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    #[test]
    fn test_neighbors() {
        assert!(neighbors().unwrap().keys().count() > 0);
    }

    #[test]
    fn test_lookup() {
        for (ip, lladdr) in neighbors().unwrap().into_iter() {
            assert_eq!(lookup(ip).unwrap().unwrap(), lladdr);
        }
    }
}
