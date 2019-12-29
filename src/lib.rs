use serde::Deserialize;
use serde_json;
use serde_json::error::Error as JsonError;

use std::collections::HashMap;
use std::io::Error as IoError;
use std::net::IpAddr;
use std::process::Command;
use std::str::Utf8Error;

#[derive(Deserialize, Debug, PartialEq)]
pub struct LlAddr(String);

#[derive(Deserialize, Debug)]
struct Neighbor {
    dst: IpAddr,
    dev: String,
    lladdr: LlAddr,
    state: Vec<Reachable>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Reachable {
    Delay,
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
    let neighbors_map = neighbors.into_iter().map(|n| (n.dst, n.lladdr)).collect();
    Ok(neighbors_map)
}
#[cfg(target_os = "linux")]
pub fn lookup(ip_addr: IpAddr) -> Result<Option<LlAddr>, LookupError> {
    let neighbors: Vec<Neighbor> = neighbors_raw()?;
    let lladdr = neighbors
        .into_iter()
        .filter(|n| n.dst == ip_addr)
        .nth(0)
        .map(|n| n.lladdr);
    Ok(lladdr)
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
