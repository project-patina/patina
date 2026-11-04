//! The Discord activity ("Rich Presence"), through the local socket the Discord
//! app listens on: `discord-ipc-N` in the runtime folder on Unix, a named pipe
//! on Windows. Nothing goes over the network from here, and with Discord closed
//! nothing happens at all.
//!
//! A frame is the opcode and the JSON's length (both u32, little endian), then
//! the JSON. The handshake names the application; SET_ACTIVITY replaces the
//! activity, or clears it when there is none.

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

/// Patina's application in the Discord Developer Portal. Its name is what
/// Discord shows after "Playing".
pub const APPLICATION_ID: &str = "1502302426336858353";
const LOGO: &str = "https://project-patina.github.io/patina/assets/discord-icon.png";
const SITE: &str = "https://project-patina.github.io/patina/";

const HANDSHAKE: u32 = 0;
const FRAME: u32 = 1;
const CLOSE: u32 = 2;
const PING: u32 = 3;
const PONG: u32 = 4;
const MAX_FRAME: usize = 64 * 1024;

/// How often a lost Discord is looked for again, and the activity sent again,
/// which also notices a Discord that restarted.
const RETRY: Duration = Duration::from_secs(20);
const RESEND: Duration = Duration::from_secs(60);
/// Changes this close together go as one: Discord takes five updates in 20 s.
const SETTLE: Duration = Duration::from_millis(600);

/// What the profile shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    /// The first line, e.g. "Playing Minecraft 26.3".
    pub details: String,
    /// The second line, e.g. "On a server: mc.example.net".
    pub state: String,
    /// The image's tooltip.
    pub image_text: String,
    /// Unix seconds: Discord counts up from it.
    pub started: Option<i64>,
    /// The label of the link to Patina's site.
    pub button: String,
}

impl Activity {
    fn to_json(&self) -> Value {
        // Discord refuses lines under 2 or over 128 characters.
        let line = |s: &str| {
            let s: String = s.chars().take(128).collect();
            format!("{s:<2}")
        };
        let mut activity = json!({
            "details": line(&self.details),
            "state": line(&self.state),
            "assets": { "large_image": LOGO, "large_text": line(&self.image_text) },
            "buttons": [{ "label": line(&self.button), "url": SITE }],
        });
        if let Some(started) = self.started {
            activity["timestamps"] = json!({ "start": started });
        }
        activity
    }
}

/// The activity's owner: a thread that keeps one connection to Discord and
/// shows the latest wanted activity on it.
pub struct Presence {
    wanted: Sender<Option<Activity>>,
}

impl Presence {
    pub fn start(application_id: &str) -> Self {
        let (wanted, rx) = mpsc::channel();
        let application_id = application_id.to_owned();
        std::thread::Builder::new()
            .name("discord".into())
            .spawn(move || run(&application_id, rx))
            .expect("spawn the Discord thread");
        Self { wanted }
    }

    /// Shows `activity`, or clears it with `None`.
    pub fn set(&self, activity: Option<Activity>) {
        let _ = self.wanted.send(activity);
    }
}

fn run(application_id: &str, rx: Receiver<Option<Activity>>) {
    // A build without Patina's application id never talks to Discord.
    if application_id.is_empty() || !application_id.chars().all(|c| c.is_ascii_digit()) {
        while rx.recv().is_ok() {}
        return;
    }
    let mut wanted: Option<Activity> = None;
    let mut conn: Option<Conn> = None;
    // What Discord shows, as far as we know, and when it was sent.
    let mut shown: Option<(Option<Activity>, Instant)> = None;
    let mut next_try = Instant::now();
    loop {
        match rx.recv_timeout(RETRY) {
            Ok(activity) => {
                wanted = activity;
                std::thread::sleep(SETTLE);
                while let Ok(activity) = rx.try_recv() {
                    wanted = activity;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(conn) = conn.as_mut() {
                    let _ = conn.set_activity(None);
                }
                return;
            }
        }
        let current = shown.as_ref().is_some_and(|(s, at)| *s == wanted && at.elapsed() < RESEND);
        if current || (conn.is_none() && wanted.is_none()) {
            continue;
        }
        if conn.is_none() {
            if Instant::now() < next_try {
                continue;
            }
            match Conn::open(application_id) {
                Ok(c) => conn = Some(c),
                Err(e) => {
                    log::debug!("discord: {e}");
                    next_try = Instant::now() + RETRY;
                    continue;
                }
            }
        }
        let Some(c) = conn.as_mut() else { continue };
        match c.set_activity(wanted.as_ref()) {
            Ok(()) => {
                shown = Some((wanted.clone(), Instant::now()));
                // Cleared: nothing to keep the connection for.
                if wanted.is_none() {
                    conn = None;
                }
            }
            Err(e) => {
                log::debug!("discord: {e}");
                conn = None;
                shown = None;
                next_try = Instant::now();
            }
        }
    }
}

#[cfg(unix)]
type Stream = std::os::unix::net::UnixStream;
#[cfg(windows)]
type Stream = std::fs::File;

/// The socket names Discord, its Flatpak and Snap packages and Vesktop listen on.
#[cfg(unix)]
fn candidates() -> Vec<std::path::PathBuf> {
    let mut bases: Vec<std::path::PathBuf> = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .filter_map(std::env::var_os)
        .map(Into::into)
        .collect();
    bases.push("/tmp".into());
    let folders = ["", "app/com.discordapp.Discord", "app/com.discordapp.DiscordCanary", "app/dev.vencord.Vesktop", "snap.discord", "snap.discord-canary", ".flatpak/dev.vencord.Vesktop/xdg-run"];
    let mut out = Vec::new();
    for base in &bases {
        for folder in folders {
            for n in 0..10 {
                out.push(base.join(folder).join(format!("discord-ipc-{n}")));
            }
        }
    }
    out
}

#[cfg(windows)]
fn candidates() -> Vec<std::path::PathBuf> {
    (0..10).map(|n| format!(r"\\.\pipe\discord-ipc-{n}").into()).collect()
}

#[cfg(unix)]
fn connect(path: &std::path::Path) -> io::Result<Stream> {
    let stream = Stream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    Ok(stream)
}

#[cfg(windows)]
fn connect(path: &std::path::Path) -> io::Result<Stream> {
    std::fs::OpenOptions::new().read(true).write(true).open(path)
}

struct Conn {
    stream: Stream,
}

impl Conn {
    fn open(application_id: &str) -> io::Result<Self> {
        let mut last = io::Error::new(io::ErrorKind::NotFound, "Discord is not running");
        for path in candidates() {
            let stream = match connect(&path) {
                Ok(stream) => stream,
                Err(e) => {
                    if e.kind() != io::ErrorKind::NotFound {
                        last = e;
                    }
                    continue;
                }
            };
            let mut conn = Self { stream };
            conn.send(HANDSHAKE, &json!({ "v": 1, "client_id": application_id }))?;
            conn.reply()?;
            return Ok(conn);
        }
        Err(last)
    }

    fn send(&mut self, op: u32, value: &Value) -> io::Result<()> {
        let body = serde_json::to_vec(value)?;
        let mut frame = Vec::with_capacity(8 + body.len());
        frame.extend_from_slice(&op.to_le_bytes());
        frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
        frame.extend_from_slice(&body);
        self.stream.write_all(&frame)?;
        self.stream.flush()
    }

    /// The next answer, after any pings. An error event or a closed socket is an error.
    fn reply(&mut self) -> io::Result<Value> {
        loop {
            let mut header = [0; 8];
            self.stream.read_exact(&mut header)?;
            let op = u32::from_le_bytes(header[..4].try_into().expect("4 bytes"));
            let len = u32::from_le_bytes(header[4..].try_into().expect("4 bytes")) as usize;
            if len > MAX_FRAME {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "oversized frame from Discord"));
            }
            let mut body = vec![0; len];
            self.stream.read_exact(&mut body)?;
            let value: Value = serde_json::from_slice(&body)?;
            match op {
                PING => self.send(PONG, &value)?,
                CLOSE => return Err(io::Error::new(io::ErrorKind::ConnectionAborted, format!("Discord closed the connection: {}", value["message"]))),
                _ if value["evt"] == "ERROR" => return Err(io::Error::other(format!("Discord refused: {}", value["data"]["message"]))),
                _ => return Ok(value),
            }
        }
    }

    fn set_activity(&mut self, activity: Option<&Activity>) -> io::Result<()> {
        let nonce = uuid::Uuid::new_v4().to_string();
        let args = json!({ "pid": std::process::id(), "activity": activity.map(Activity::to_json) });
        self.send(FRAME, &json!({ "cmd": "SET_ACTIVITY", "args": args, "nonce": nonce }))?;
        self.reply().map(drop)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    fn read_frame(stream: &mut Stream) -> (u32, Value) {
        let mut header = [0; 8];
        stream.read_exact(&mut header).unwrap();
        let len = u32::from_le_bytes(header[4..].try_into().unwrap()) as usize;
        let mut body = vec![0; len];
        stream.read_exact(&mut body).unwrap();
        (u32::from_le_bytes(header[..4].try_into().unwrap()), serde_json::from_slice(&body).unwrap())
    }

    fn write_frame(stream: &mut Stream, op: u32, value: Value) {
        let body = serde_json::to_vec(&value).unwrap();
        stream.write_all(&op.to_le_bytes()).unwrap();
        stream.write_all(&(body.len() as u32).to_le_bytes()).unwrap();
        stream.write_all(&body).unwrap();
    }

    #[test]
    fn handshakes_answers_pings_and_sets_then_clears_the_activity() {
        let dir = std::env::temp_dir().join(format!("patina-discord-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("discord-ipc-0");
        let listener = UnixListener::bind(&path).unwrap();
        let fake = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let (op, hello) = read_frame(&mut s);
            assert_eq!((op, hello["client_id"].as_str()), (HANDSHAKE, Some("123")));
            write_frame(&mut s, FRAME, json!({ "cmd": "DISPATCH", "evt": "READY" }));
            let (_, set) = read_frame(&mut s);
            // A ping before the answer must be answered and skipped.
            write_frame(&mut s, PING, json!({ "n": 1 }));
            assert_eq!(read_frame(&mut s), (PONG, json!({ "n": 1 })));
            write_frame(&mut s, FRAME, json!({ "cmd": "SET_ACTIVITY", "nonce": set["nonce"] }));
            let (_, cleared) = read_frame(&mut s);
            write_frame(&mut s, FRAME, json!({ "evt": "ERROR", "data": { "message": "nope" } }));
            (set, cleared)
        });
        let mut conn = Conn { stream: connect(&path).unwrap() };
        conn.send(HANDSHAKE, &json!({ "v": 1, "client_id": "123" })).unwrap();
        conn.reply().unwrap();
        let activity = Activity { details: "Playing Minecraft 26.3".into(), state: "x".into(), image_text: "Patina".into(), started: Some(1_700_000_000), button: "Get Patina".into() };
        conn.set_activity(Some(&activity)).unwrap();
        let refused = conn.set_activity(None).unwrap_err();
        assert!(refused.to_string().contains("nope"), "{refused}");
        let (set, cleared) = fake.join().unwrap();
        assert_eq!(set["cmd"], "SET_ACTIVITY");
        let shown = &set["args"]["activity"];
        assert_eq!(shown["details"], "Playing Minecraft 26.3");
        assert_eq!(shown["state"], "x ", "short lines are padded to Discord's minimum");
        assert_eq!(shown["timestamps"]["start"], 1_700_000_000);
        assert_eq!(shown["buttons"][0]["url"], SITE);
        assert!(cleared["args"]["activity"].is_null());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
