#!/usr/bin/env python3
"""Films the home screen's backdrop with the Rust engine: a real village at sunset.

The film is made from the player's own game, so it stays in the launcher's data
folder and is never committed or shipped. Needs ffmpeg, an X server (XWayland)
and an installed engine (scripts/install-engine.sh).

Screen capture of the game comes out black on NVIDIA, so the film is shot frame
by frame from the engine, with the world slowed down to match: `/tick rate` makes
one shot's worth of real time pass for each frame of the film.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

DATA = os.path.join(os.environ.get("XDG_DATA_HOME", os.path.expanduser("~/.local/share")), "amethyst")
ENGINE = os.environ.get("AMETHYST_ENGINE_DIR", os.path.join(DATA, "engine"))
OUT = os.path.join(DATA, "backdrop")
SOCKET = os.path.join(os.environ["XDG_RUNTIME_DIR"], "backdrop-ctl.sock")

SEED = "8675309"
TIME = 12230
CAMERA = (46, 84, 2, 84, 9)  # offset from the village, height, offset, yaw, pitch
FPS = 24
SECONDS = 24
FADE = 3
GRADE = "eq=contrast=1.06:saturation=1.16:gamma=1.04,colorbalance=rm=0.05:bm=-0.05:rh=0.05:bh=-0.04,vignette=PI/5"


def ctl(*args, timeout=60):
    command = [os.path.join(ENGINE, "patina-ctl"), "--socket", SOCKET, "--timeout", str(timeout), *map(str, args)]
    return subprocess.run(command, capture_output=True, text=True).stdout


def wait_for_socket():
    for _ in range(80):
        if os.path.exists(SOCKET):
            return
        time.sleep(0.5)
    sys.exit("The game did not start.")


def film(frames):
    """Writes every frame of the film as a PNG; the world is slowed to the speed of shooting."""
    probe = os.path.join(frames, "probe.png")
    started = time.time()
    for _ in range(8):
        ctl("screenshot", probe)
    per_shot = (time.time() - started) / 8
    rate = max(1.0, 20 / (FPS * per_shot))
    print(f"{per_shot * 1000:.0f} ms a frame, tick rate {rate:.1f}", flush=True)
    ctl("command", f"tick rate {rate:.1f}")
    time.sleep(2)
    for i in range(FPS * SECONDS):
        ctl("screenshot", os.path.join(frames, f"{i:05}.png"))
    os.remove(probe)


def main():
    work = tempfile.mkdtemp(prefix="backdrop-work-", dir=DATA)
    game = os.path.join(work, "game")
    os.makedirs(game)
    with open(os.path.join(game, "options.txt"), "w") as f:
        f.write('fullscreen:true\nrenderDistance:24\nsimulationDistance:12\nfov:-0.15\nenableVsync:false\nmaxFps:120\nparticles:0\ntutorialStep:none\npauseOnLostFocus:false\ninactivityFpsLimit:"minimized"\n')
    with open(os.path.join(game, "patina-graphics.txt"), "w") as f:
        f.write("lodDistance:256\ntaa:true\nfxaa:false\n")

    env = {k: v for k, v in os.environ.items() if k != "WAYLAND_DISPLAY"}
    env.update(PATINA_NO_AUDIO="1", PATINA_HOME=game)
    if os.path.exists(SOCKET):
        os.remove(SOCKET)
    log = open(os.path.join(work, "log.txt"), "w")
    client = subprocess.Popen(
        [os.path.join(ENGINE, "patina-client"), "--resources", os.path.join(ENGINE, ".local/patina-client/26.3"),
         "--game-dir", game, "--home", game, "--world", "Backdrop", "--seed", SEED, "--creative",
         "--render-distance", "24", "--control", "--control-socket", SOCKET],
        env=env, stdout=log, stderr=subprocess.STDOUT)
    try:
        wait_for_socket()
        ctl("wait-ready", timeout=120)
        ctl("command", "gamemode spectator")
        ctl("command", "gamerule minecraft:advance_time false")
        ctl("command", "weather clear")
        ctl("command", "locate structure #minecraft:village")
        time.sleep(4)
        found = re.search(r"\[(-?\d+), ?~, ?(-?\d+)\]", ctl("messages"))
        if not found:
            sys.exit("No village found.")
        ctl("key", "f1")
        dx, y, dz, yaw, pitch = CAMERA
        ctl("tp", int(found[1]) + dx, y, int(found[2]) + dz)
        ctl("look", yaw, pitch)
        ctl("time", TIME)
        ctl("wait-ready", timeout=120)
        time.sleep(25)
        frames = os.path.join(work, "frames")
        os.makedirs(frames)
        film(frames)
        ctl("quit")
        time.sleep(3)
    finally:
        client.kill()

    os.makedirs(OUT, exist_ok=True)
    raw = os.path.join(work, "raw.mp4")
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(FPS), "-i", os.path.join(frames, "%05d.png"),
                    "-c:v", "libx264", "-preset", "fast", "-crf", "14", "-pix_fmt", "yuv444p", raw], check=True)
    body = SECONDS - FADE
    graph = (f"[0:v]split[a][b];[a]trim=0:{body},setpts=PTS-STARTPTS[main];[b]trim={body}:{SECONDS},setpts=PTS-STARTPTS[tail];"
             f"[0:v]trim=0:{FADE},setpts=PTS-STARTPTS[head];[tail][head]xfade=transition=fade:duration={FADE}:offset=0[seam];"
             f"[seam][main]concat=n=2:v=1:a=0,trim=0:{body},setpts=PTS-STARTPTS,{GRADE},format=yuv420p")
    # JPEG frames, played by the launcher itself: `<video>` brings down WebKitGTK when
    # GStreamer's good plugins are missing.
    graded = os.path.join(work, "graded.mp4")
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", raw, "-filter_complex", graph, "-an", "-c:v", "libx264", "-crf", "14", graded], check=True)
    for old in os.listdir(OUT):
        os.remove(os.path.join(OUT, old))
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", graded, "-vf", "fps=12,scale=1280:-2:flags=lanczos", "-q:v", "4",
                    os.path.join(OUT, "%05d.jpg")], check=True)
    shutil.rmtree(work, ignore_errors=True)
    print(f"Backdrop written to {OUT}")


if __name__ == "__main__":
    main()
