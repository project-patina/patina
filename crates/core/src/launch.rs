//! Turn a prepared install into a java command line and run it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::install::Prepared;
use crate::instance::{Instance, Overrides};
use crate::mojang::{self, Arg, Features};
use crate::settings::Settings;
use crate::{Ctx, Result};

/// Who is playing.
#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub xuid: String,
    /// `msa` for Microsoft accounts, `legacy` for offline test profiles.
    pub user_type: String,
}

impl Player {
    /// Offline profile, only used by the headless CLI to test launching.
    pub fn offline(name: &str) -> Self {
        let uuid = uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_OID, format!("OfflinePlayer:{name}").as_bytes());
        Self { name: name.into(), uuid: uuid.simple().to_string(), access_token: "0".into(), xuid: "0".into(), user_type: "legacy".into() }
    }
}

#[derive(Debug, Clone)]
pub struct LaunchSpec {
    /// argv[0..]: optional wrappers (gamemoderun, mangohud, …) then java.
    pub program: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: PathBuf,
    /// Strings to mask when echoing logs/commands.
    pub secrets: Vec<String>,
    /// Variables the program must not inherit.
    pub env_remove: Vec<String>,
    /// Written to the program's standard input, then closed.
    pub stdin: Option<String>,
}

impl LaunchSpec {
    pub fn display(&self) -> String {
        let mut s = self.program.iter().chain(&self.args).cloned().collect::<Vec<_>>().join(" ");
        for sec in &self.secrets {
            if sec.len() > 4 {
                s = s.replace(sec, "********");
            }
        }
        s
    }
}

pub const OPTIMIZED_JVM_ARGS: &[&str] = &[
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+UseG1GC",
    "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20",
    "-XX:MaxGCPauseMillis=50",
    "-XX:G1HeapRegionSize=32M",
    "-XX:+DisableExplicitGC",
    "-XX:+ParallelRefProcEnabled",
];

/// Split a user-supplied argument string, honouring simple quotes.
pub fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut has = false;
    for c in s.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                has = true;
            }
            (None, c) if c.is_whitespace() => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn parse_env(s: &str) -> Vec<(String, String)> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

pub fn find_in_path(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(bin)).find(|p| p.is_file()))
}

fn substitute(s: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(v) => out.push_str(v),
                    None => {
                        out.push_str("${");
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn eval_args(args: &[Arg], features: &Features, vars: &HashMap<&str, String>) -> Vec<String> {
    let mut out = Vec::new();
    for a in args {
        match a {
            Arg::Plain(s) => out.push(substitute(s, vars)),
            Arg::Ruled { rules, value } => {
                if mojang::rules_allow(Some(rules), features) {
                    out.extend(value.to_vec().iter().map(|s| substitute(s, vars)));
                }
            }
        }
    }
    out
}

/// Drop `--flag ${unresolved}` pairs (quick play etc. we did not fill in).
fn drop_unresolved(args: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    for a in args {
        if a.contains("${") {
            if out.last().is_some_and(|p| p.starts_with("--")) {
                out.pop();
            }
            continue;
        }
        out.push(a);
    }
    out
}

fn has_nvidia() -> bool {
    Path::new("/proc/driver/nvidia/version").exists()
}

pub fn build(ctx: &Ctx, inst: &Instance, prep: &Prepared, player: &Player, s: &Settings) -> LaunchSpec {
    let o = &inst.overrides;
    let game_dir = ctx.paths.game_dir(&inst.id);
    let sep = if cfg!(windows) { ";" } else { ":" };
    let classpath = prep.classpath.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>().join(sep);
    let width = o.game_width.unwrap_or(s.game_width);
    let height = o.game_height.unwrap_or(s.game_height);
    let fullscreen = o.fullscreen.unwrap_or(s.fullscreen);
    let assets_dir = prep.legacy_assets_dir.clone().unwrap_or_else(|| prep.assets_root.clone());

    let vars: HashMap<&str, String> = HashMap::from([
        ("auth_player_name", player.name.clone()),
        ("version_name", prep.version.id.clone()),
        ("game_directory", game_dir.to_string_lossy().into()),
        ("assets_root", assets_dir.to_string_lossy().into()),
        ("game_assets", assets_dir.to_string_lossy().into()),
        ("assets_index_name", prep.assets_index.clone()),
        ("auth_uuid", player.uuid.clone()),
        ("auth_access_token", player.access_token.clone()),
        ("auth_session", player.access_token.clone()),
        ("auth_xuid", player.xuid.clone()),
        ("clientid", crate::auth::CLIENT_ID.to_string()),
        ("user_type", player.user_type.clone()),
        ("user_properties", "{}".into()),
        ("version_type", prep.version.kind.clone().unwrap_or_else(|| "release".into())),
        ("natives_directory", prep.natives_dir.to_string_lossy().into()),
        ("launcher_name", crate::LAUNCHER_NAME.to_string()),
        ("launcher_version", crate::LAUNCHER_VERSION.to_string()),
        ("classpath", classpath),
        ("classpath_separator", sep.into()),
        ("library_directory", ctx.paths.libraries().to_string_lossy().into()),
        ("resolution_width", width.to_string()),
        ("resolution_height", height.to_string()),
    ]);
    let features = Features { custom_resolution: !fullscreen };

    // JVM
    let mut args: Vec<String> = Vec::new();
    let min = o.memory_min_mb.unwrap_or(s.memory_min_mb);
    let max = o.memory_max_mb.unwrap_or(s.memory_max_mb).max(min);
    args.push(format!("-Xms{min}M"));
    args.push(format!("-Xmx{max}M"));
    if s.optimized_jvm_args {
        args.extend(OPTIMIZED_JVM_ARGS.iter().map(|a| a.to_string()));
    }
    args.extend(split_args(&s.jvm_args));
    if let Some(extra) = &o.jvm_args {
        args.extend(split_args(extra));
    }
    match &prep.version.arguments {
        Some(a) if !a.jvm.is_empty() => args.extend(eval_args(&a.jvm, &features, &vars)),
        _ => {
            args.push(substitute("-Djava.library.path=${natives_directory}", &vars));
            args.push("-cp".into());
            args.push(vars["classpath"].clone());
        }
    }
    if let Some(l) = &prep.logging_arg {
        args.push(l.clone());
    }
    args.push(prep.version.main_class.clone().unwrap_or_else(|| "net.minecraft.client.main.Main".into()));

    // Game
    let mut game = match (&prep.version.arguments, &prep.version.minecraft_arguments) {
        (Some(a), _) if !a.game.is_empty() => eval_args(&a.game, &features, &vars),
        (_, Some(legacy)) => {
            let mut g: Vec<String> = legacy.split_whitespace().map(|a| substitute(a, &vars)).collect();
            if !fullscreen {
                g.extend(["--width".into(), width.to_string(), "--height".into(), height.to_string()]);
            }
            g
        }
        _ => vec![],
    };
    game = drop_unresolved(game);
    if fullscreen && !game.iter().any(|a| a == "--fullscreen") {
        game.push("--fullscreen".into());
    }
    if let Some(server) = o.join_server.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let quick_play = serde_json::to_string(&prep.version.arguments).unwrap_or_default().contains("quickPlayMultiplayer");
        if quick_play {
            game.extend(["--quickPlayMultiplayer".into(), server.into()]);
        } else {
            let (host, port) = server.split_once(':').unwrap_or((server, "25565"));
            game.extend(["--server".into(), host.into(), "--port".into(), port.into()]);
        }
    }
    args.extend(game);

    let mut program = wrappers(o, s);
    program.push(prep.java.to_string_lossy().into_owned());
    let env = base_env(o, s);

    LaunchSpec { program, args, env, cwd: game_dir, secrets: vec![player.access_token.clone()], env_remove: vec![], stdin: None }
}

/// What goes in front of the game's program: gamemode, MangoHud, the player's own wrapper.
pub fn wrappers(o: &Overrides, s: &Settings) -> Vec<String> {
    let mut program = Vec::new();
    if s.gamemode {
        if let Some(p) = find_in_path("gamemoderun") {
            program.push(p.to_string_lossy().into_owned());
        }
    }
    if s.mangohud {
        if let Some(p) = find_in_path("mangohud") {
            program.push(p.to_string_lossy().into_owned());
        }
    }
    program.extend(split_args(o.wrapper_command.as_deref().unwrap_or(&s.wrapper_command)));
    program
}

/// The GPU choice and the player's own variables.
pub fn base_env(o: &Overrides, s: &Settings) -> Vec<(String, String)> {
    let mut env = Vec::new();
    if s.discrete_gpu {
        env.push(("DRI_PRIME".into(), "1".into()));
        if has_nvidia() {
            env.push(("__NV_PRIME_RENDER_OFFLOAD".into(), "1".into()));
            env.push(("__GLX_VENDOR_LIBRARY_NAME".into(), "nvidia".into()));
            env.push(("__VK_LAYER_NV_optimus".into(), "NVIDIA_only".into()));
        }
    }
    env.extend(parse_env(&s.env_vars));
    if let Some(e) = &o.env_vars {
        env.extend(parse_env(e));
    }
    env
}

/// A launcher with no console opens a window for every console program it
/// starts on Windows; CREATE_NO_WINDOW keeps them out of sight.
pub fn hide_console(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    #[cfg(not(windows))]
    let _ = command;
}

/// Run a shell hook (pre-launch / post-exit) in the game directory.
pub async fn run_hook(cmd: &str, cwd: &Path) -> Result<()> {
    if cmd.trim().is_empty() {
        return Ok(());
    }
    let (shell, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
    let mut command = tokio::process::Command::new(shell);
    command.arg(flag).arg(cmd).current_dir(cwd).kill_on_drop(true);
    hide_console(&mut command);
    let status = command.status().await?;
    if !status.success() {
        crate::bail!("Komenda „{cmd}” zakończyła się błędem ({status})");
    }
    Ok(())
}

/// `capture = false` sends output nowhere; used when the launcher exits right
/// after starting the game, so the game never writes into a closed pipe.
pub fn spawn(spec: &LaunchSpec, capture: bool) -> Result<tokio::process::Child> {
    std::fs::create_dir_all(&spec.cwd)?;
    let out = || if capture { Stdio::piped() } else { Stdio::null() };
    let mut cmd = tokio::process::Command::new(&spec.program[0]);
    cmd.args(&spec.program[1..])
        .args(&spec.args)
        .envs(spec.env.iter().map(|(k, v)| (k, v)))
        .current_dir(&spec.cwd)
        .stdin(if spec.stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(out())
        .stderr(out());
    for key in &spec.env_remove {
        cmd.env_remove(key);
    }
    hide_console(&mut cmd);
    // The game must survive the launcher closing (on_launch = close).
    #[cfg(unix)]
    cmd.process_group(0);
    Ok(cmd.spawn()?)
}

/// Hand over `spec.stdin` and close the pipe, so the game reads to the end.
pub async fn feed_stdin(child: &mut tokio::process::Child, spec: &LaunchSpec) -> Result<()> {
    if let (Some(input), Some(mut pipe)) = (&spec.stdin, child.stdin.take()) {
        pipe.write_all(input.as_bytes()).await?;
    }
    Ok(())
}

/// The vanilla log config prints log4j XML events; turn them back into
/// readable `[time] [thread/LEVEL]: message` lines. Plain lines pass through.
#[derive(Default)]
pub struct Log4jParser {
    in_event: bool,
    level: String,
    thread: String,
    time: String,
    text: String,
    in_cdata: bool,
}

fn attr(tag: &str, name: &str) -> String {
    let key = format!("{name}=\"");
    tag.find(&key).map(|i| &tag[i + key.len()..]).and_then(|r| r.split('"').next()).unwrap_or("").to_string()
}

impl Log4jParser {
    /// Returns finished `(line, level)` pairs.
    pub fn feed(&mut self, line: &str) -> Vec<(String, String)> {
        let t = line.trim_start();
        if !self.in_event {
            if t.starts_with("<log4j:Event") {
                self.in_event = true;
                self.level = attr(t, "level");
                self.thread = attr(t, "thread");
                self.time = attr(t, "timestamp")
                    .parse::<i64>()
                    .ok()
                    .and_then(chrono::DateTime::from_timestamp_millis)
                    .map(|d| d.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
                    .unwrap_or_default();
                self.text.clear();
                return vec![];
            }
            let level = if line.contains("ERROR") || line.contains("Exception") { "ERROR" } else if line.contains("WARN") { "WARN" } else { "INFO" };
            return vec![(line.to_string(), level.to_string())];
        }
        if t.starts_with("</log4j:Event>") {
            self.in_event = false;
            let msg = std::mem::take(&mut self.text);
            let head = format!("[{}] [{}/{}]: ", self.time, self.thread, self.level);
            return msg.trim_end().lines().enumerate().map(|(i, l)| (if i == 0 { format!("{head}{l}") } else { l.to_string() }, self.level.clone())).collect();
        }
        let mut rest = line;
        loop {
            if self.in_cdata {
                match rest.find("]]>") {
                    Some(end) => {
                        self.text.push_str(&rest[..end]);
                        self.in_cdata = false;
                        rest = &rest[end + 3..];
                    }
                    None => {
                        self.text.push_str(rest);
                        self.text.push('\n');
                        break;
                    }
                }
            } else {
                match rest.find("<![CDATA[") {
                    Some(start) => {
                        if !self.text.is_empty() && !self.text.ends_with('\n') {
                            self.text.push('\n');
                        }
                        self.in_cdata = true;
                        rest = &rest[start + 9..];
                    }
                    None => break,
                }
            }
        }
        vec![]
    }
}

/// Forward child stdout/stderr line by line as `(line, level)`, secrets masked.
pub fn pipe_output(child: &mut tokio::process::Child, secrets: Vec<String>, on_line: std::sync::Arc<dyn Fn(String, String) + Send + Sync>) {
    let mask = std::sync::Arc::new(move |mut l: String| {
        for s in &secrets {
            if s.len() > 4 {
                l = l.replace(s.as_str(), "********");
            }
        }
        l
    });
    if let Some(out) = child.stdout.take() {
        let (f, m) = (on_line.clone(), mask.clone());
        tokio::spawn(async move {
            let mut parser = Log4jParser::default();
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                for (line, level) in parser.feed(&l) {
                    f(m(line), level);
                }
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let (f, m) = (on_line, mask);
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                f(m(l), "ERROR".into());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitting() {
        assert_eq!(split_args(r#"-Dfoo=bar  -Dx="a b" ''"#), vec!["-Dfoo=bar", "-Dx=a b", ""]);
        assert!(split_args("   ").is_empty());
    }

    #[test]
    fn substitution() {
        let vars = HashMap::from([("a", "1".to_string())]);
        assert_eq!(substitute("x${a}y${b}z", &vars), "x1y${b}z");
        assert_eq!(drop_unresolved(vec!["--x".into(), "${q}".into(), "--y".into(), "2".into()]), vec!["--y", "2"]);
    }

    #[test]
    fn log4j_xml() {
        let mut p = Log4jParser::default();
        let mut out = Vec::new();
        for l in [
            r#"<log4j:Event logger="net.minecraft.client.Minecraft" timestamp="1793305041035" level="INFO" thread="Render thread">"#,
            "  <log4j:Message><![CDATA[Backend library: LWJGL]]></log4j:Message>",
            "</log4j:Event>",
            r#"<log4j:Event logger="x" timestamp="1" level="ERROR" thread="main">"#,
            "  <log4j:Message><![CDATA[Boom]]></log4j:Message>",
            "  <log4j:Throwable><![CDATA[java.lang.Exception: bad",
            "\tat a.b(c)",
            "]]></log4j:Throwable>",
            "</log4j:Event>",
            "plain line",
        ] {
            out.extend(p.feed(l));
        }
        assert!(out[0].0.ends_with("[Render thread/INFO]: Backend library: LWJGL"), "{:?}", out[0]);
        assert!(out[1].0.ends_with("[main/ERROR]: Boom"));
        assert_eq!(out[2].0, "java.lang.Exception: bad");
        assert_eq!(out[3].0, "\tat a.b(c)");
        assert_eq!(out[1].1, "ERROR");
        assert_eq!(out.last().unwrap().0, "plain line");
    }

    #[test]
    fn env_parsing() {
        assert_eq!(parse_env("A=1\n# c\n\nB = two=2"), vec![("A".into(), "1".into()), ("B".into(), "two=2".into())]);
    }
}
