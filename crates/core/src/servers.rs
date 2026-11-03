//! The Servers page: a list of well-known public servers with the region they
//! are hosted in, and the Minecraft server list ping (`ServerStatusPinger`)
//! that tells whether one is up, how many play on it and its message.
//! Partner servers will come from Patina itself; there are none yet.

use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{bail, Result};

const DEFAULT_PORT: u16 = 25565;
/// For the whole ping: a server that takes longer counts as offline.
const TIMEOUT: Duration = Duration::from_secs(5);
/// The status answer is a JSON text with an icon of a few kilobytes; more is not a server.
const MAX_PACKET: usize = 1 << 20;
/// Sent in the handshake. Servers answer the status whatever the client's version.
const PROTOCOL: i32 = 775;

/// Where a server is hosted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Eu,
    Na,
    Sa,
    As,
    Oc,
}

/// A server of the list.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub region: Region,
    /// What is played there, as short lowercase keys the page translates.
    pub tags: &'static [&'static str],
}

const fn server(id: &'static str, name: &'static str, address: &'static str, region: Region, tags: &'static [&'static str]) -> Server {
    Server { id, name, address, region, tags }
}

/// Well-known public Java Edition servers. Partner servers are not here: there are none yet.
pub const POPULAR: &[Server] = &[
    server("hypixel", "Hypixel", "mc.hypixel.net", Region::Na, &["minigames", "skyblock", "bedwars"]),
    server("cubecraft", "CubeCraft", "play.cubecraft.net", Region::Eu, &["minigames", "skyblock"]),
    server("wynncraft", "Wynncraft", "play.wynncraft.com", Region::Na, &["rpg"]),
    server("manacube", "ManaCube", "play.manacube.com", Region::Na, &["skyblock", "parkour", "survival"]),
    server("complex", "Complex Gaming", "hub.mc-complex.com", Region::Na, &["pixelmon", "survival", "skyblock"]),
    server("pika", "PikaNetwork", "play.pika-network.net", Region::Na, &["minigames", "bedwars", "survival"]),
    server("minemen", "Minemen Club", "minemen.club", Region::Na, &["pvp"]),
    server("jartex", "Jartex Network", "play.jartexnetwork.com", Region::Eu, &["minigames", "bedwars", "survival"]),
    server("earthmc", "EarthMC", "play.earthmc.net", Region::Eu, &["towny", "survival"]),
    server("minehut", "Minehut", "minehut.com", Region::Na, &["community"]),
    server("purpleprison", "Purple Prison", "purpleprison.org", Region::Na, &["prison"]),
    server("2b2t", "2b2t", "2b2t.org", Region::Na, &["anarchy", "survival"]),
];

/// What a server answered.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub online: bool,
    pub players: Option<u32>,
    pub max_players: Option<u32>,
    pub version: Option<String>,
    /// The message of the day as plain text, its formatting codes taken out.
    pub motd: String,
    pub latency_ms: Option<u32>,
    /// The server's icon as a `data:image/png` URL.
    pub favicon: Option<String>,
}

/// `host`, `host:port` or `[v6]:port`, the port 25565 when it is left out.
pub fn split_address(address: &str) -> Result<(String, Option<u16>)> {
    let address = address.trim();
    if address.is_empty() || address.contains(char::is_whitespace) {
        bail!("Niepoprawny adres serwera", "Invalid server address");
    }
    if let Some(rest) = address.strip_prefix('[') {
        let Some((host, port)) = rest.split_once(']') else { bail!("Niepoprawny adres serwera", "Invalid server address") };
        let port = port.strip_prefix(':').map(str::parse::<u16>).transpose().ok().flatten();
        return Ok((host.to_string(), port));
    }
    match address.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') => match port.parse() {
            Ok(port) => Ok((host.to_string(), Some(port))),
            Err(_) => bail!("Niepoprawny port serwera", "Invalid server port"),
        },
        _ => Ok((address.to_string(), None)),
    }
}

/// Ping `address` as the multiplayer list does. A server that does not answer is offline, not an error.
pub async fn ping(address: &str) -> Result<Status> {
    let (host, port) = split_address(address)?;
    Ok(tokio::time::timeout(TIMEOUT, status(&host, port)).await.ok().and_then(|s| s.ok()).unwrap_or_default())
}

async fn status(host: &str, port: Option<u16>) -> std::io::Result<Status> {
    // ServerAddress: a name without a port may point elsewhere with an SRV record.
    let (target, port) = match port {
        Some(port) => (host.to_string(), port),
        None => srv::lookup(host).await.unwrap_or_else(|| (host.to_string(), DEFAULT_PORT)),
    };
    let mut stream = TcpStream::connect((target.as_str(), port)).await?;
    stream.set_nodelay(true)?;
    // Handshake (intention 1, status), then the status request.
    let mut handshake = Vec::new();
    write_varint(&mut handshake, 0);
    write_varint(&mut handshake, PROTOCOL);
    write_string(&mut handshake, host);
    handshake.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut handshake, 1);
    let mut out = Vec::new();
    write_packet(&mut out, &handshake);
    write_packet(&mut out, &[0]);
    let sent = Instant::now();
    stream.write_all(&out).await?;
    let response = read_packet(&mut stream).await?;
    let mut latency = sent.elapsed();
    let mut body = response.as_slice();
    if read_varint(&mut body)? != 0 {
        return Err(invalid("not a status response"));
    }
    let json: serde_json::Value = serde_json::from_slice(read_string(&mut body)?).map_err(|_| invalid("not JSON"))?;
    // The ping and its pong give the latency the list shows, when the server answers it.
    let mut ping = Vec::new();
    write_varint(&mut ping, 1);
    ping.extend_from_slice(&0x5061_7469_6e61_i64.to_be_bytes());
    let mut out = Vec::new();
    write_packet(&mut out, &ping);
    let pinged = Instant::now();
    if stream.write_all(&out).await.is_ok() && tokio::time::timeout(Duration::from_secs(2), read_packet(&mut stream)).await.is_ok_and(|p| p.is_ok()) {
        latency = pinged.elapsed();
    }
    Ok(parse_status(&json, latency))
}

fn parse_status(json: &serde_json::Value, latency: Duration) -> Status {
    let number = |v: &serde_json::Value| v.as_u64().map(|n| n.min(u64::from(u32::MAX)) as u32);
    Status {
        online: true,
        players: number(&json["players"]["online"]),
        max_players: number(&json["players"]["max"]),
        version: json["version"]["name"].as_str().map(|v| strip_codes(v).trim().to_string()).filter(|v| !v.is_empty()),
        // Servers line their message up with runs of spaces, which mean nothing in a list.
        motd: strip_codes(&text_of(&json["description"]))
            .lines()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        latency_ms: Some(latency.as_millis().min(u128::from(u32::MAX)) as u32),
        favicon: json["favicon"].as_str().filter(|f| f.starts_with("data:image/png;base64,")).map(str::to_string),
    }
}

/// A text component's text: a string, or an object with `text` and `extra`.
fn text_of(component: &serde_json::Value) -> String {
    match component {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(parts) => parts.iter().map(text_of).collect(),
        serde_json::Value::Object(o) => {
            let mut s = o.get("text").and_then(|t| t.as_str()).unwrap_or_default().to_string();
            if let Some(extra) = o.get("extra") {
                s.push_str(&text_of(extra));
            }
            s
        }
        _ => String::new(),
    }
}

/// Takes out the `§` formatting codes and the character after each.
fn strip_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

fn invalid(what: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, what.to_string())
}

fn write_varint(out: &mut Vec<u8>, value: i32) {
    let mut v = value as u32;
    loop {
        if v & !0x7f == 0 {
            out.push(v as u8);
            return;
        }
        out.push((v & 0x7f | 0x80) as u8);
        v >>= 7;
    }
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    write_varint(out, s.len() as i32);
    out.extend_from_slice(s.as_bytes());
}

fn write_packet(out: &mut Vec<u8>, body: &[u8]) {
    write_varint(out, body.len() as i32);
    out.extend_from_slice(body);
}

fn read_varint(buf: &mut &[u8]) -> std::io::Result<i32> {
    let mut value = 0u32;
    for i in 0..5 {
        let (&byte, rest) = buf.split_first().ok_or_else(|| invalid("short varint"))?;
        *buf = rest;
        value |= u32::from(byte & 0x7f) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(value as i32);
        }
    }
    Err(invalid("varint too long"))
}

fn read_string<'a>(buf: &mut &'a [u8]) -> std::io::Result<&'a [u8]> {
    let len = usize::try_from(read_varint(buf)?).map_err(|_| invalid("negative length"))?;
    if len > buf.len() {
        return Err(invalid("short string"));
    }
    let (s, rest) = buf.split_at(len);
    *buf = rest;
    Ok(s)
}

async fn read_packet(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut len = 0u32;
    for i in 0..5 {
        let byte = stream.read_u8().await?;
        len |= u32::from(byte & 0x7f) << (7 * i);
        if byte & 0x80 == 0 {
            break;
        }
        if i == 4 {
            return Err(invalid("varint too long"));
        }
    }
    let len = len as usize;
    if len == 0 || len > MAX_PACKET {
        return Err(invalid("bad packet length"));
    }
    let mut body = vec![0; len];
    stream.read_exact(&mut body).await?;
    Ok(body)
}

/// `_minecraft._tcp` SRV records through the system's resolver, as `ServerNameResolver` looks them up.
mod srv {
    use std::time::Duration;

    /// The record's target and port, if the name has one.
    pub async fn lookup(host: &str) -> Option<(String, u16)> {
        if host.parse::<std::net::IpAddr>().is_ok() || !host.contains('.') {
            return None;
        }
        let server = nameserver()?;
        let query = query(&format!("_minecraft._tcp.{host}"));
        let socket = tokio::net::UdpSocket::bind(if server.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" }).await.ok()?;
        socket.send_to(&query, (server, 53)).await.ok()?;
        let mut answer = vec![0; 1500];
        let len = tokio::time::timeout(Duration::from_secs(2), socket.recv(&mut answer)).await.ok()?.ok()?;
        parse(&answer[..len], &query[..2])
    }

    /// The first `nameserver` of `/etc/resolv.conf`; other systems resolve names without SRV.
    fn nameserver() -> Option<std::net::IpAddr> {
        let conf = std::fs::read_to_string("/etc/resolv.conf").ok()?;
        conf.lines().find_map(|l| l.trim().strip_prefix("nameserver")).and_then(|s| s.trim().parse().ok())
    }

    pub(super) fn query(name: &str) -> Vec<u8> {
        let id = (std::process::id() as u16) ^ 0x5a5a;
        let mut q = Vec::with_capacity(32 + name.len());
        q.extend_from_slice(&id.to_be_bytes());
        // Recursion desired; one question.
        q.extend_from_slice(&[0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0]);
        for label in name.trim_end_matches('.').split('.') {
            q.push(label.len().min(63) as u8);
            q.extend_from_slice(&label.as_bytes()[..label.len().min(63)]);
        }
        // QTYPE SRV, QCLASS IN.
        q.extend_from_slice(&[0, 0, 33, 0, 1]);
        q
    }

    /// Skips a name, following nothing: compression pointers end it.
    fn skip_name(m: &[u8], mut at: usize) -> Option<usize> {
        loop {
            let len = *m.get(at)? as usize;
            if len & 0xc0 == 0xc0 {
                return Some(at + 2);
            }
            at += 1;
            if len == 0 {
                return Some(at);
            }
            at += len;
        }
    }

    fn read_name(m: &[u8], mut at: usize) -> Option<String> {
        let mut labels = Vec::new();
        for _ in 0..32 {
            let len = *m.get(at)? as usize;
            if len & 0xc0 == 0xc0 {
                at = (len & 0x3f) << 8 | *m.get(at + 1)? as usize;
                continue;
            }
            if len == 0 {
                return Some(labels.join("."));
            }
            labels.push(String::from_utf8_lossy(m.get(at + 1..at + 1 + len)?).into_owned());
            at += 1 + len;
        }
        None
    }

    pub(super) fn parse(m: &[u8], id: &[u8]) -> Option<(String, u16)> {
        if m.len() < 12 || &m[..2] != id || m[3] & 0x0f != 0 {
            return None;
        }
        let questions = u16::from_be_bytes([m[4], m[5]]);
        let answers = u16::from_be_bytes([m[6], m[7]]);
        let mut at = 12;
        for _ in 0..questions {
            at = skip_name(m, at)? + 4;
        }
        // The lowest priority wins; among equals the first, as the game takes the first answer.
        let mut best: Option<(u16, String, u16)> = None;
        for _ in 0..answers {
            at = skip_name(m, at)?;
            let kind = u16::from_be_bytes([*m.get(at)?, *m.get(at + 1)?]);
            let len = u16::from_be_bytes([*m.get(at + 8)?, *m.get(at + 9)?]) as usize;
            let data = at + 10;
            if kind == 33 && len >= 7 {
                let priority = u16::from_be_bytes([m[data], m[data + 1]]);
                let port = u16::from_be_bytes([*m.get(data + 4)?, *m.get(data + 5)?]);
                let target = read_name(m, data + 6)?;
                if !target.is_empty() && best.as_ref().is_none_or(|b| priority < b.0) {
                    best = Some((priority, target, port));
                }
            }
            at = data + len;
        }
        best.map(|(_, target, port)| (target, port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_split_into_host_and_port() {
        assert_eq!(split_address("mc.hypixel.net").unwrap(), ("mc.hypixel.net".into(), None));
        assert_eq!(split_address(" play.example.com:25570 ").unwrap(), ("play.example.com".into(), Some(25570)));
        assert_eq!(split_address("[::1]:25566").unwrap(), ("::1".into(), Some(25566)));
        assert!(split_address("").is_err());
        assert!(split_address("a b").is_err());
        assert!(split_address("host:port").is_err());
    }

    #[test]
    fn a_status_answer_reads_as_the_list_shows_it() {
        let json = serde_json::json!({
            "version": {"name": "§cVelocity 1.8-26.3", "protocol": 775},
            "players": {"max": 200000, "online": 41234},
            "description": {"text": "§aHypixel   Network ", "extra": [{"text": "§c[1.8-26.3]\n  §eSKYBLOCK"}]},
            "favicon": "data:image/png;base64,AAAA"
        });
        let s = parse_status(&json, Duration::from_millis(42));
        assert!(s.online);
        assert_eq!((s.players, s.max_players, s.latency_ms), (Some(41234), Some(200000), Some(42)));
        assert_eq!(s.version.as_deref(), Some("Velocity 1.8-26.3"));
        assert_eq!(s.motd, "Hypixel Network [1.8-26.3]\nSKYBLOCK");
        assert!(s.favicon.is_some());
    }

    #[test]
    fn varints_round_trip() {
        for v in [0, 1, 127, 128, 25565, 775, i32::MAX, -1] {
            let mut out = Vec::new();
            write_varint(&mut out, v);
            assert_eq!(read_varint(&mut out.as_slice()).unwrap(), v);
        }
    }

    #[test]
    fn an_srv_answer_gives_its_target_and_port() {
        let query = srv::query("_minecraft._tcp.example.com");
        let mut answer = query.clone();
        // A response, one answer: a pointer to the question's name, SRV IN, 10 1 25570 mc.example.com.
        answer[2] = 0x81;
        answer[3] = 0x80;
        answer[7] = 1;
        answer.extend_from_slice(&[0xc0, 12, 0, 33, 0, 1, 0, 0, 1, 0]);
        let target: &[u8] = b"\x02mc\x07example\x03com\x00";
        answer.extend_from_slice(&((6 + target.len()) as u16).to_be_bytes());
        answer.extend_from_slice(&[0, 10, 0, 1]);
        answer.extend_from_slice(&25570u16.to_be_bytes());
        answer.extend_from_slice(target);
        assert_eq!(srv::parse(&answer, &query[..2]), Some(("mc.example.com".into(), 25570)));
        assert_eq!(srv::parse(&answer, &[0, 0]), None);
    }

    /// Over the network: `cargo test -p amethyst-core servers -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore]
    async fn the_listed_servers_answer() {
        for server in POPULAR {
            let s = ping(server.address).await.unwrap();
            println!("{:<16} online {} players {:?}/{:?} {:?} ms {:?}", server.name, s.online, s.players, s.max_players, s.latency_ms, s.motd.lines().next());
        }
    }

    #[test]
    fn the_list_has_unique_ids_and_valid_addresses() {
        let mut ids: Vec<_> = POPULAR.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), POPULAR.len());
        assert!(POPULAR.iter().all(|s| split_address(s.address).is_ok()));
    }
}
