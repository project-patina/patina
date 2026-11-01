<script lang="ts">
  import { fly } from "svelte/transition";
  import { Check, FolderOpen, LoaderCircle, LogOut, Plus } from "@lucide/svelte";
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import Head from "../components/Head.svelte";
  import Modal from "../components/Modal.svelte";
  import LoginPanel from "../components/LoginPanel.svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import ThemePicker from "../components/ThemePicker.svelte";
  import { api } from "../lib/api";
  import { ACCENTS, LOADERS } from "../lib/presets";
  import { app, dur, flushSettings, refreshAccounts, toast, toastError, updateSettings } from "../lib/state.svelte";
  import type { EngineSettings, JavaInstall, Settings } from "../lib/types";

  const sections = [
    { id: "look", label: "Wygląd" },
    { id: "game", label: "Gra" },
    { id: "engine", label: "Silnik Rust" },
    { id: "java", label: "Java" },
    { id: "launcher", label: "Launcher" },
    { id: "linux", label: "Zaawansowane" },
    { id: "accounts", label: "Konta" },
    { id: "about", label: "O programie" },
  ] as const;
  let section = $state<(typeof sections)[number]["id"]>("look");

  const s = $derived(app.settings!);
  const set = <K extends keyof Settings>(k: K) => (v: Settings[K]) => updateSettings({ [k]: v } as Partial<Settings>);

  const e = $derived(s.rustEngine);
  const setEngine = (patch: Partial<EngineSettings>) => updateSettings({ rustEngine: { ...e, ...patch } });
  const num = (ev: { currentTarget: HTMLSelectElement }) => (ev.currentTarget.value === "" ? null : +ev.currentTarget.value);
  const flag = (v: string) => (v === "" ? null : v === "on");

  let javas = $state<JavaInstall[] | null>(null);
  let scanning = $state(false);
  async function scanJava() {
    scanning = true;
    try {
      javas = await api.detectJava();
    } catch (e) {
      toastError(e);
    } finally {
      scanning = false;
    }
  }

  let addAccount = $state(false);
  async function logout(uuid: string) {
    try {
      app.accounts = await api.removeAccount(uuid);
      await refreshAccounts();
    } catch (e) {
      toastError(e);
    }
  }
</script>

{#snippet triple(key: "hdr" | "batteryMode" | "fpsOverlay")}
  <select class="input pick" value={e[key] === null ? "" : e[key] ? "on" : "off"} onchange={(ev) => setEngine({ [key]: flag(ev.currentTarget.value) })}>
    <option value="">Jak w grze</option>
    <option value="on">Włączone</option>
    <option value="off">Wyłączone</option>
  </select>
{/snippet}

<div class="page">
  <header class="settings-head"><h1>Ustawienia</h1><div class="save-state" aria-live="polite">
    {#if app.settingsSave === "error"}<button class="retry" onclick={() => flushSettings().catch(() => {})}>Błąd zapisu · ponów</button>
    {:else if app.settingsSave === "pending"}Zapisywanie…{:else}Zapisano automatycznie{/if}
  </div></header>
  <nav class="tabs" aria-label="Kategorie ustawień">
    {#each sections as sec (sec.id)}
      <button class:on={section === sec.id} aria-current={section === sec.id ? "page" : undefined} onclick={() => (section = sec.id)}>{sec.label}</button>
    {/each}
  </nav>

  {#key section}
    <section class="content" in:fly={{ y: 8, duration: dur(220) }}>
      <div class="column">
      {#if section === "look"}
        <h2 class="look-title">Twój styl, mniej rozpraszaczy.</h2>
        <p class="look-desc muted">Wybierz paletę. Resztę dopasujesz, jeśli zechcesz.</p>
        <ThemePicker />
        <div class="card block">
          <Row title="Tryb kolorów">
            <div class="seg">
              {#each [["dark", "Ciemny"], ["light", "Jasny"], ["system", "Systemowy"]] as [k, l] (k)}
                <button class:on={s.theme === k} aria-pressed={s.theme === k} onclick={() => updateSettings({ theme: k as Settings["theme"] })}>{l}</button>
              {/each}
            </div>
          </Row>
        </div>
        <details class="appearance-details">
          <summary>Dostosuj interfejs</summary>
          <div class="card block">
          <Row title="Kolor akcentu">
            <div class="accents">
              {#each ACCENTS as c (c)}
                <button class="sw" class:on={s.accent === c} style:background={c} aria-label={c} onclick={() => updateSettings({ accent: c })}>
                  {#if s.accent === c}<Check size={13} />{/if}
                </button>
              {/each}
              <label class="sw custom" title="Własny kolor">
                <input type="color" aria-label="Własny kolor akcentu" value={s.accent} oninput={(e) => updateSettings({ accent: e.currentTarget.value })} />
                <Plus size={13} />
              </label>
            </div>
          </Row>
          <Row title="Animacje" desc="Płynne przejścia między ekranami i elementami."><Toggle label="Animacje" checked={s.animations} onchange={set("animations")} /></Row>
          <Row title="Efekt rozmycia" desc="Szkło pod menu i oknami. Wyłącz na słabszym sprzęcie."><Toggle label="Efekt rozmycia" checked={s.blur} onchange={set("blur")} /></Row>
          <Row title="Skala interfejsu">
            <div class="scale">
              <input type="range" min="0.8" max="1.25" step="0.05" value={s.uiScale} oninput={(e) => updateSettings({ uiScale: +e.currentTarget.value })} />
              <span class="mono">{Math.round(s.uiScale * 100)}%</span>
            </div>
          </Row>
          <Row title="Czcionka">
            <div class="seg">
              {#each [["inter", "Inter"], ["outfit", "Outfit"], ["system", "Systemowa"], ["mono", "Mono"]] as [k, l] (k)}
                <button class:on={s.font === k} onclick={() => updateSettings({ font: k as Settings["font"] })}>{l}</button>
              {/each}
            </div>
          </Row>
          <Row title="Zaokrąglenia">
            <div class="seg">
              {#each [["sharp", "Ostre"], ["normal", "Normalne"], ["round", "Okrągłe"]] as [k, l] (k)}
                <button class:on={s.radius === k} onclick={() => updateSettings({ radius: k as Settings["radius"] })}>{l}</button>
              {/each}
            </div>
          </Row>
          <Row title="Tryb kompaktowy" desc="Mniejsze odstępy i czcionka — więcej się mieści."><Toggle label="Tryb kompaktowy" checked={s.compact} onchange={set("compact")} /></Row>
        </div>

        </details>
        <details class="appearance-details">
          <summary>Ekran startowy i postać</summary>
        <div class="card block">
          <Row title="Kinowe tło" desc="Animowane tło wioski. Wyłącz, aby używać tapety i zmniejszyć obciążenie.">
            <Toggle label="Kinowe tło" checked={s.heroScene} onchange={set("heroScene")} />
          </Row>
          <Row title="Tapeta" desc="Tło za postacią na stronie startowej. „Baner instancji” bierze tło wybranej instancji." stacked>
            <ImagePicker value={s.heroWallpaper} noneLabel="Baner instancji" onchange={(v) => updateSettings({ heroWallpaper: v, heroScene: false })} />
          </Row>
          <Row title="Przyciemnienie tapety">
            <div class="scale">
              <input type="range" min="0" max="80" value={s.heroDim} oninput={(e) => updateSettings({ heroDim: +e.currentTarget.value })} />
              <span class="mono">{s.heroDim}%</span>
            </div>
          </Row>
          <Row title="Pozycja postaci">
            <div class="seg">
              {#each [["left", "Lewo"], ["center", "Środek"], ["right", "Prawo"]] as [k, l] (k)}
                <button class:on={s.modelPosition === k} onclick={() => updateSettings({ modelPosition: k as Settings["modelPosition"] })}>{l}</button>
              {/each}
            </div>
          </Row>
          <Row title="Głowa podąża za kursorem"><Toggle label="Głowa podąża za kursorem" checked={s.followCursor} onchange={set("followCursor")} /></Row>
          <Row title="Plakietka z nickiem"><Toggle label="Plakietka z nickiem" checked={s.showNametag} onchange={set("showNametag")} /></Row>
        </div>

        </details>
        <details class="appearance-details">
          <summary>Tapeta launchera</summary>
        <div class="card block">
          <Row title="Tapeta launchera" desc="Obraz za całym interfejsem." stacked>
            <ImagePicker value={s.appWallpaper} noneLabel="Brak" onchange={(v) => updateSettings({ appWallpaper: v })} />
          </Row>
          {#if s.appWallpaper}
            <Row title="Przyciemnienie" desc="Im więcej, tym czytelniejszy tekst.">
              <div class="scale">
                <input type="range" min="0" max="95" value={s.appWallpaperDim} oninput={(e) => updateSettings({ appWallpaperDim: +e.currentTarget.value })} />
                <span class="mono">{s.appWallpaperDim}%</span>
              </div>
            </Row>
          {/if}
        </div>
        </details>
      {:else if section === "game"}
        <div class="card block">
          <Row title="Rozdzielczość okna gry">
            <input class="input num" type="number" min="320" value={s.gameWidth} onchange={(e) => updateSettings({ gameWidth: +e.currentTarget.value })} />
            <span class="dim">×</span>
            <input class="input num" type="number" min="240" value={s.gameHeight} onchange={(e) => updateSettings({ gameHeight: +e.currentTarget.value })} />
          </Row>
          <Row title="Pełny ekran" desc="Uruchamiaj grę od razu na pełnym ekranie."><Toggle label="Pełny ekran" checked={s.fullscreen} onchange={set("fullscreen")} /></Row>
        </div>
        <h3 class="sub">Nowe instancje</h3>
        <div class="card block">
          <Row title="Domyślny loader">
            <div class="seg">
              {#each LOADERS as l (l.id)}
                <button class:on={s.defaultLoader === l.id} onclick={() => updateSettings({ defaultLoader: l.id })}>{l.label}</button>
              {/each}
            </div>
          </Row>
          <Row title="Instaluj mody optymalizacyjne" desc="Sodium, Lithium, FerriteCore, ImmediatelyFast, Entity Culling, Mod Menu i Fabric API przy pierwszym uruchomieniu instancji Fabric/Quilt.">
            <Toggle label="Instaluj mody optymalizacyjne" checked={s.installPerformanceMods} onchange={set("installPerformanceMods")} />
          </Row>
          <Row title="Pokazuj snapshoty"><Toggle label="Pokazuj snapshoty" checked={s.showSnapshots} onchange={set("showSnapshots")} /></Row>
          <Row title="Pokazuj stare wersje (beta/alpha)"><Toggle label="Pokazuj stare wersje (beta/alpha)" checked={s.showOldVersions} onchange={set("showOldVersions")} /></Row>
        </div>
      {:else if section === "engine"}
        <p class="muted lead">Dotyczy instancji z loaderem Rust. „Jak w grze” zostawia to, co ustawisz w menu gry. Launcher zapisuje wybrane wartości przy każdym uruchomieniu.</p>
        <h3 class="sub">Wydajność</h3>
        <div class="card block">
          <Row title="Uruchamiaj przez XWayland" desc="Znacznie więcej FPS na laptopach z NVIDIA i Intelem, na których Wayland spowalnia przekazywanie klatek do ekranu."><Toggle label="Uruchamiaj przez XWayland" checked={e.xwayland} onchange={(v) => setEngine({ xwayland: v })} /></Row>
        </div>
        <h3 class="sub">Grafika</h3>
        <div class="card block">
          <Row title="Daleki teren (LOD)" desc="Uproszczony teren poza zasięgiem renderowania.">
            <select class="input pick" value={String(e.lodDistance ?? "")} onchange={(ev) => setEngine({ lodDistance: num(ev) })}>
              <option value="">Jak w grze</option>
              <option value="0">Wyłączony</option>
              {#each [64, 128, 256, 512, 1024] as d (d)}<option value={String(d)}>{d} chunków</option>{/each}
            </select>
          </Row>
          <Row title="Skala rozdzielczości" desc="Poniżej 100% szybciej, powyżej ostrzej.">
            <select class="input pick" value={String(e.resolutionScale ?? "")} onchange={(ev) => setEngine({ resolutionScale: num(ev) })}>
              <option value="">Jak w grze</option>
              {#each [50, 75, 100, 125, 150, 200] as p (p)}<option value={String(p)}>{p}%</option>{/each}
            </select>
          </Row>
          <Row title="Wygładzanie krawędzi">
            <select class="input pick" value={e.antialiasing} onchange={(ev) => setEngine({ antialiasing: ev.currentTarget.value as EngineSettings["antialiasing"] })}>
              <option value="game">Jak w grze</option>
              <option value="off">Wyłączone</option>
              <option value="fxaa">FXAA</option>
              <option value="taa">TAA</option>
            </select>
          </Row>
          <Row title="API graficzne" desc="Na Linuksie domyślnie Vulkan, OpenGL 3.3 jako zapas.">
            <select class="input pick" value={e.graphicsApi} onchange={(ev) => setEngine({ graphicsApi: ev.currentTarget.value as EngineSettings["graphicsApi"] })}>
              <option value="game">Jak w grze</option>
              <option value="auto">Automatycznie</option>
              <option value="vulkan">Vulkan</option>
              <option value="opengl">OpenGL</option>
            </select>
          </Row>
          <Row title="HDR" desc="Wymaga monitora i kompozytora z HDR.">{@render triple("hdr")}</Row>
          <Row title="Tryb baterii" desc="Do 60 FPS w świecie i mniej wątków. Dla laptopów na baterii.">{@render triple("batteryMode")}</Row>
          <Row title="Licznik FPS" desc="Nakładka silnika z liczbą klatek.">{@render triple("fpsOverlay")}</Row>
        </div>
      {:else if section === "java"}
        <div class="card block">
          <Row title="Maksymalna pamięć (RAM)" desc={`Twój komputer ma ${((app.info?.totalMemoryMb ?? 0) / 1024).toFixed(1)} GB. Dla modów zwykle wystarcza 4–6 GB.`}>
            <MemorySlider value={s.memoryMaxMb} onchange={(v) => updateSettings({ memoryMaxMb: v })} />
          </Row>
          <Row title="Pamięć początkowa">
            <select class="input" style="width: 140px" value={String(s.memoryMinMb)} onchange={(e) => updateSettings({ memoryMinMb: +e.currentTarget.value })}>
              {#each [512, 1024, 2048, 4096] as m (m)}<option value={String(m)}>{m >= 1024 ? `${m / 1024} GB` : `${m} MB`}</option>{/each}
            </select>
          </Row>
          <Row title="Zoptymalizowane argumenty JVM" desc="G1GC dostrojony pod Minecrafta — mniej przycięć przy zbieraniu śmieci."><Toggle label="Zoptymalizowane argumenty JVM" checked={s.optimizedJvmArgs} onchange={set("optimizedJvmArgs")} /></Row>
          <Row title="Własne argumenty JVM" stacked>
            <input class="input mono" placeholder="np. -XX:+UseZGC" value={s.jvmArgs} onchange={(e) => updateSettings({ jvmArgs: e.currentTarget.value })} />
          </Row>
          <Row title="Java" desc="Domyślnie launcher sam pobiera właściwą Javę (8/17/21/25) dla każdej wersji gry." stacked>
            <div class="java">
              <input class="input mono" placeholder="Automatycznie (zalecane)" value={s.javaPath} onchange={(e) => updateSettings({ javaPath: e.currentTarget.value.trim() })} />
              <button class="btn" onclick={scanJava} disabled={scanning}>{#if scanning}<LoaderCircle size={15} class="spin" />{/if} Wykryj</button>
              {#if s.javaPath}<button class="btn ghost" onclick={() => updateSettings({ javaPath: "" })}>Automatycznie</button>{/if}
            </div>
            {#if javas}
              <div class="javas">
                {#each javas as j (j.path)}
                  <button class="jv" class:on={s.javaPath === j.path} onclick={() => updateSettings({ javaPath: j.path })}>
                    <span class="chip accent">Java {j.major}</span>
                    <span class="mono path">{j.path}</span>
                    <span class="dim">{j.source === "bundled" ? "pobrana" : "systemowa"}</span>
                  </button>
                {:else}
                  <div class="dim">Nie znaleziono Javy w systemie — to nic, launcher pobierze ją sam.</div>
                {/each}
              </div>
            {/if}
          </Row>
        </div>
      {:else if section === "launcher"}
        <div class="card block">
          <Row title="Po uruchomieniu gry" desc="Co ma zrobić launcher, gdy gra wystartuje.">
            <select class="input" style="width: 220px" value={s.onLaunch} onchange={(e) => updateSettings({ onLaunch: e.currentTarget.value as Settings["onLaunch"] })}>
              <option value="hide">Ukryj launcher</option>
              <option value="minimize">Zminimalizuj</option>
              <option value="keep">Zostaw otwarty</option>
              <option value="close">Zamknij launcher</option>
            </select>
          </Row>
          <Row title="Pokaż launcher po zamknięciu gry"><Toggle label="Pokaż launcher po zamknięciu gry" checked={s.reopenOnExit} onchange={set("reopenOnExit")} /></Row>
          <Row title="Otwieraj konsolę przy starcie gry"><Toggle label="Otwieraj konsolę przy starcie gry" checked={s.openConsoleOnLaunch} onchange={set("openConsoleOnLaunch")} /></Row>
          <Row title="Równoległe pobieranie" desc="Ile plików pobierać naraz.">
            <div class="scale">
              <input type="range" min="2" max="48" step="1" value={s.maxConcurrentDownloads} oninput={(e) => updateSettings({ maxConcurrentDownloads: +e.currentTarget.value })} />
              <span class="mono">{s.maxConcurrentDownloads}</span>
            </div>
          </Row>
        </div>
      {:else if section === "linux"}
        <div class="card block">
          <Row title="Feral GameMode" desc="Uruchamia grę przez gamemoderun (pakiet gamemode)."><Toggle label="Feral GameMode" checked={s.gamemode} onchange={set("gamemode")} /></Row>
          <Row title="MangoHud" desc="Nakładka z FPS i obciążeniem (pakiet mangohud)."><Toggle label="MangoHud" checked={s.mangohud} onchange={set("mangohud")} /></Row>
          <Row title="Dedykowana karta graficzna" desc="PRIME offload: na laptopach z dwiema kartami gra użyje mocniejszej (NVIDIA/AMD)."><Toggle label="Dedykowana karta graficzna" checked={s.discreteGpu} onchange={set("discreteGpu")} /></Row>
          <Row title="Komenda opakowująca" desc="Doklejana przed java, np. prime-run albo obs-gamecapture.">
            <input class="input mono" style="width: 260px" placeholder="brak" value={s.wrapperCommand} onchange={(e) => updateSettings({ wrapperCommand: e.currentTarget.value })} />
          </Row>
          <Row title="Zmienne środowiskowe" desc="Jedna na linię: KLUCZ=wartość." stacked>
            <textarea class="input" rows="3" placeholder="__GL_THREADED_OPTIMIZATIONS=0" value={s.envVars} onchange={(e) => updateSettings({ envVars: e.currentTarget.value })}></textarea>
          </Row>
          <Row title="Komenda przed uruchomieniem" desc="Wykonywana w folderze instancji (sh -c). Błąd przerywa start." stacked>
            <input class="input mono" placeholder="brak" value={s.preLaunchCommand} onchange={(e) => updateSettings({ preLaunchCommand: e.currentTarget.value })} />
          </Row>
          <Row title="Komenda po zamknięciu gry" stacked>
            <input class="input mono" placeholder="brak" value={s.postExitCommand} onchange={(e) => updateSettings({ postExitCommand: e.currentTarget.value })} />
          </Row>
        </div>
      {:else if section === "accounts"}
        <div class="card block">
          {#each app.accounts as a (a.uuid)}
            <div class="acc">
              <Head skinUrl={a.skinUrl} name={a.name} size={36} radius={9} />
              <div class="acc-text">
                <b>{a.name}</b>
                <small class="dim mono">{a.uuid}</small>
              </div>
              {#if a.active}
                <span class="chip accent">Aktywne</span>
              {:else}
                <button class="btn sm" onclick={() => api.setActiveAccount(a.uuid).then((l) => (app.accounts = l)).catch(toastError)}>Ustaw aktywne</button>
              {/if}
              <button class="btn sm ghost danger" onclick={() => logout(a.uuid)}><LogOut size={14} /> Wyloguj</button>
            </div>
          {/each}
          <div class="acc add">
            <button class="btn" onclick={() => (addAccount = true)}><Plus size={15} /> Dodaj konto Microsoft</button>
          </div>
        </div>
      {:else}
        <div class="card block about">
          <img src="/icon.svg" alt="" width="56" height="56" />
          <div>
            <h3>Patina <span class="dim">{app.info?.version}</span></h3>
            <p class="muted">Minimalistyczny launcher Minecraft. {app.info?.os} / {app.info?.arch}</p>
          </div>
        </div>
        <div class="card block">
          <p class="muted credit">Silnik Rust ma otwarty kod na licencji MIT (plik LICENSE w folderze silnika). Patina jest nieoficjalna, niezwiązana z Mojang ani Microsoft.</p>
        </div>
        <div class="card block">
          <Row title="Folder danych" desc={app.info?.dataDir}>
            <button class="btn" onclick={() => api.openFolder().catch(toastError)}><FolderOpen size={15} /> Otwórz</button>
          </Row>
          <Row title="Przewodnik powitalny" desc="Pokaż ponownie ekran pierwszego uruchomienia.">
            <button class="btn" onclick={() => { updateSettings({ onboarded: false }); toast("Przewodnik pokaże się teraz"); }}>Pokaż</button>
          </Row>
        </div>
      {/if}
      </div>
    </section>
  {/key}
</div>

{#if addAccount}
  <Modal onclose={() => { api.cancelLogin(); addAccount = false; }} width={420}>
    <h3 style="margin-bottom: 16px">Dodaj konto</h3>
    <LoginPanel onsuccess={(a) => { addAccount = false; toast(`Zalogowano jako ${a.name}`, "success"); }} />
  </Modal>
{/if}

<style>
  .settings-head { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 10px 24px 14px; }
  .settings-head h1 { font-size: 1.55rem; font-weight: 600; }
  .save-state { color: var(--text-3); font-size: 0.78rem; }
  .retry { color: var(--danger); text-decoration: underline; text-underline-offset: 3px; }
  .look-title { font-size: 1.45rem; font-weight: 560; letter-spacing: -0.035em; margin: 10px 0 6px; }
  .look-desc { margin: 0 0 22px; font-size: 0.9rem; }
  .appearance-details { border-top: 1px solid var(--border); }
  .appearance-details summary { padding: 19px 0; cursor: pointer; color: var(--text-2); font-size: 0.92rem; font-weight: 550; }
  .appearance-details summary:hover, .appearance-details[open] summary { color: var(--text); }
  .appearance-details[open] { padding-bottom: 18px; }
  .page {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .tabs {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 4px 22px;
    padding: 8px 12px 0;
    border-bottom: 1px solid var(--border);
  }
  .tabs button {
    padding: 8px 2px 11px;
    margin-bottom: -1px;
    border-bottom: 2px solid transparent;
    font-size: 0.9rem;
    font-weight: 560;
    color: var(--text-3);
    transition: color 0.16s, border-color 0.16s;
  }
  .tabs button:hover {
    color: var(--text);
  }
  .tabs button.on {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
  .content {
    flex: 1;
    overflow: auto;
    min-width: 0;
    padding: 18px 20px 40px;
  }
  .column {
    max-width: 640px;
    margin: 0 auto;
  }
  /* Rows sit straight on the page: hairlines instead of cards. */
  .column :global(.card) {
    background: none;
    border: 0;
    border-radius: 0;
  }
  .pick {
    width: 170px;
  }
  .lead {
    margin: -6px 0 14px;
    line-height: 1.45;
  }
  .credit {
    margin-top: 12px;
    font-size: 0.85rem;
    line-height: 1.45;
  }
  .card .credit {
    margin: 0;
    padding: 14px 0;
  }
  .sub {
    font-size: 0.72rem;
    font-weight: 650;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin: 26px 0 2px;
    color: var(--text-3);
  }
  .block {
    padding: 0;
  }
  .seg {
    display: flex;
    padding: 3px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }
  .seg button {
    padding: 6px 12px;
    border-radius: 7px;
    font-weight: 600;
    font-size: 0.88rem;
    color: var(--text-2);
    transition: background 0.2s, color 0.2s;
  }
  .seg button.on {
    background: var(--panel-3);
    color: var(--text);
    box-shadow: 0 1px 3px rgb(0 0 0 / 0.15);
  }
  .accents {
    display: flex;
    gap: 7px;
  }
  .sw {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    border: 2px solid var(--panel);
    box-shadow: 0 0 0 1px var(--border-2);
    transition: transform 0.2s var(--ease-spring);
    cursor: pointer;
    position: relative;
  }
  .sw:hover {
    transform: scale(1.15);
  }
  .sw.on {
    box-shadow: 0 0 0 2px var(--text);
  }
  .custom {
    background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red);
  }
  .custom input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .scale {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 260px;
  }
  .scale input {
    flex: 1;
    accent-color: var(--accent);
  }
  .scale span {
    width: 44px;
    text-align: right;
  }
  .num {
    width: 96px;
  }
  .java {
    display: flex;
    gap: 8px;
  }
  .javas {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }
  .jv {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 8px;
    text-align: left;
    background: var(--panel-2);
    border: 1px solid transparent;
  }
  .jv:hover {
    border-color: var(--border-2);
  }
  .jv.on {
    border-color: var(--accent);
  }
  .path {
    flex: 1;
    font-size: 0.8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .acc {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
  }
  .acc.add {
    border-bottom: 0;
  }
  .acc-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .acc-text small {
    font-size: 0.75rem;
    margin-top: 2px;
  }
  .about {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 18px 20px;
    margin-bottom: 12px;
  }
  .about p {
    margin: 4px 0 0;
  }
</style>
