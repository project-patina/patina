<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Check, FolderOpen, LoaderCircle, LogOut, Plus } from "@lucide/svelte";
  import Group from "../components/Group.svelte";
  import Head from "../components/Head.svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import NumberInput from "../components/NumberInput.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Select from "../components/Select.svelte";
  import ThemePicker from "../components/ThemePicker.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { api } from "../lib/api";
  import { bytes, relativeTime } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import type { Key } from "../lib/locales/pl";
  import { indicator, page } from "../lib/motion";
  import { ACCENTS, LOADERS } from "../lib/presets";
  import { app, flushSettings, refreshAccounts, toast, toastError, updateSettings, type SettingsSection } from "../lib/state.svelte";
  import type { EngineSettings, JavaInstall, Settings } from "../lib/types";
  import { PATINA_DEV } from "../lib/flavour";
  import { checkNow, install, RELEASES_URL, updates } from "../lib/updates.svelte";

  const groups: { title: Key | ""; items: { id: SettingsSection; label: Key }[] }[] = [
    { title: "settings.groupLauncher", items: [{ id: "general", label: "settings.general" }, { id: "look", label: "settings.look" }] },
    { title: "settings.groupGame", items: [{ id: "game", label: "settings.game" }, { id: "engine", label: "settings.engine" }, { id: "java", label: "settings.java" }, { id: "advanced", label: "settings.advanced" }] },
    { title: "settings.groupAccount", items: [{ id: "accounts", label: "settings.accounts" }] },
  ];
  const titles = Object.fromEntries(groups.flatMap((g) => g.items).map((i) => [i.id, i.label])) as Record<string, Key>;
  const section = $derived(app.settingsSection);
  const title = $derived(t(titles[section] ?? "settings.about"));

  const s = $derived(app.settings!);
  const set = <K extends keyof Settings>(k: K) => (v: Settings[K]) => updateSettings({ [k]: v } as Partial<Settings>);
  const e = $derived(s.rustEngine);
  const setEngine = (patch: Partial<EngineSettings>) => updateSettings({ rustEngine: { ...e, ...patch } });

  const onOff = $derived([{ value: "", label: t("engine.asGame") }, { value: "on", label: t("common.on") }, { value: "off", label: t("common.off") }]);
  const triple = (key: "hdr" | "batteryMode" | "fpsOverlay") => (e[key] === null ? "" : e[key] ? "on" : "off");
  const setTriple = (key: "hdr" | "batteryMode" | "fpsOverlay") => (v: string) => setEngine({ [key]: v === "" ? null : v === "on" });
  const numeric = (key: "lodDistance" | "resolutionScale") => (v: string) => setEngine({ [key]: v === "" ? null : +v });
  const lodOptions = $derived([
    { value: "", label: t("engine.asGame") }, { value: "0", label: t("common.off") },
    ...[64, 128, 256, 512, 1024].map((d) => ({ value: String(d), label: t("engine.chunks", { n: d }) })),
  ]);
  const scaleOptions = $derived([{ value: "", label: t("engine.asGame") }, ...[50, 75, 100, 125, 150, 200].map((p) => ({ value: String(p), label: `${p}%` }))]);

  // The start heap can never exceed the maximum: lowering one pulls the other along.
  const memoryOptions = $derived([512, 1024, 2048, 4096].filter((m) => m <= s.memoryMaxMb).map((m) => ({ value: String(m), label: m >= 1024 ? `${m / 1024} GB` : `${m} MB` })));
  const setMaxMemory = (v: number) => updateSettings(s.memoryMinMb > v ? { memoryMaxMb: v, memoryMinMb: v } : { memoryMaxMb: v });
  const minMemoryValue = $derived(memoryOptions.some((o) => o.value === String(s.memoryMinMb)) ? String(s.memoryMinMb) : memoryOptions[memoryOptions.length - 1]?.value ?? "512");

  let javas = $state<JavaInstall[] | null>(null);
  let scanning = $state(false);
  async function scanJava() {
    scanning = true;
    try {
      javas = await api.detectJava();
    } catch (error) {
      toastError(error);
    } finally {
      scanning = false;
    }
  }

  async function logout(uuid: string) {
    try {
      app.accounts = await api.removeAccount(uuid);
      await refreshAccounts();
    } catch (error) {
      toastError(error);
    }
  }

  const updateStatus = $derived.by(() => {
    if (updates.checking) return t("update.checking");
    if (updates.checkError) return updates.checkError;
    if (updates.info) return t("update.availableVersion", { version: updates.info.version, size: bytes(updates.info.sizeBytes) });
    return updates.checked ? t("update.upToDate") : t("update.notChecked");
  });
</script>

<div class="settings">
  <nav aria-label={t("settings.title")} use:indicator={section}>
    <span class="nav-indicator" data-indicator></span>
    {#each groups as group (group.title)}
      <div class="nav-label">{t(group.title as Key)}</div>
      {#each group.items as item (item.id)}
        <button class="nav-item" aria-current={section === item.id ? "page" : undefined} onclick={() => (app.settingsSection = item.id)}>{t(item.label)}</button>
      {/each}
    {/each}
    <button class="nav-item about-link" aria-current={section === "about" ? "page" : undefined} onclick={() => (app.settingsSection = "about")}>{t("settings.about")}</button>
  </nav>

  {#key section}
    <div class="pane" in:page>
      <div class="column">
        <header>
          <h1 class="page-title">{title}</h1>
          {#if app.settingsSave === "error"}
            <button class="save error" onclick={() => flushSettings().catch(() => {})}>{t("settings.saveFailed")}</button>
          {:else if app.settingsSave === "pending"}
            <span class="save" aria-live="polite">{t("settings.saving")}</span>
          {/if}
        </header>

        {#if section === "general"}
          <Group title={t("general.language")}>
            <Row title={t("general.languageRow")}>
              <Segmented label={t("general.languageRow")} value={s.language === "pl" || s.language === "en" ? s.language : ""}
                options={[{ value: "pl", label: "Polski" }, { value: "en", label: "English" }, { value: "", label: t("general.system") }]}
                onchange={(language) => updateSettings({ language })} />
            </Row>
          </Group>
          <Group title={t("general.launch")}>
            <Row title={t("general.afterStart")}>
              <Select label={t("general.afterStart")} value={s.onLaunch} onchange={set("onLaunch")} width={200} options={[
                { value: "hide", label: t("general.onLaunchHide") }, { value: "minimize", label: t("general.onLaunchMinimize") },
                { value: "keep", label: t("general.onLaunchKeep") }, { value: "close", label: t("general.onLaunchClose") },
              ]} />
            </Row>
            <Row title={t("general.reopen")}><Toggle label={t("general.reopen")} checked={s.reopenOnExit} onchange={set("reopenOnExit")} /></Row>
            <Row title={t("general.console")}><Toggle label={t("general.console")} checked={s.openConsoleOnLaunch} onchange={set("openConsoleOnLaunch")} /></Row>
          </Group>
          <Group title={t("general.discord")}>
            <Row title={t("general.discordPresence")} desc={t("general.discordPresenceDesc")}><Toggle label={t("general.discordPresence")} checked={s.discordPresence} onchange={set("discordPresence")} /></Row>
            {#if s.discordPresence}
              <Row title={t("general.discordServer")} desc={t("general.discordServerDesc")}><Toggle label={t("general.discordServer")} checked={s.discordShowServer} onchange={set("discordShowServer")} /></Row>
            {/if}
          </Group>
          <Group title={t("general.downloads")}>
            <Row title={t("general.parallel")} desc={t("general.parallelDesc")}>
              <div class="range">
                <input type="range" min="2" max="48" step="1" aria-label={t("general.parallel")} value={s.maxConcurrentDownloads} oninput={(ev) => updateSettings({ maxConcurrentDownloads: +ev.currentTarget.value })} />
                <span class="mono">{s.maxConcurrentDownloads}</span>
              </div>
            </Row>
          </Group>
          <Group title={t("update.title")}>
            <Row title={t("update.version")}><span class="value mono">{app.info?.version}</span></Row>
            {#if PATINA_DEV}
              <Row title={t("update.status")}><span class="value">{t("update.devOff")}</span></Row>
            {:else}
              <Row title={t("update.lastCheck")}><span class="value">{updates.lastCheck ? relativeTime(updates.lastCheck) : t("time.never")}</span></Row>
              <Row title={t("update.status")} desc={updateStatus}>
                {#if updates.info?.installable}
                  <button class="btn primary" onclick={install}>{t("update.install")}</button>
                {:else if updates.info}
                  <button class="btn" onclick={() => openUrl(RELEASES_URL).catch(toastError)}>{t("update.openReleases")}</button>
                {/if}
                <button class="btn" disabled={updates.checking} onclick={checkNow}>
                  {#if updates.checking}<LoaderCircle size={14} class="spin" />{/if} {t("update.check")}
                </button>
              </Row>
            {/if}
          </Group>
        {:else if section === "look"}
          <Group title={t("look.palette")}>
            <ThemePicker />
            <Row title={t("look.mode")}>
              <Segmented label={t("look.mode")} value={s.theme} onchange={set("theme")} options={[{ value: "dark", label: t("look.dark") }, { value: "light", label: t("look.light") }, { value: "system", label: t("general.system") }]} />
            </Row>
            <Row title={t("look.accent")} stacked>
              <div class="accents">
                {#each ACCENTS as colour (colour)}
                  <button class="sw" style:background={colour} aria-label={colour} aria-pressed={s.accent === colour} onclick={() => updateSettings({ accent: colour })}>
                    {#if s.accent === colour}<Check size={14} />{/if}
                  </button>
                {/each}
                <label class="sw custom" class:on={!ACCENTS.includes(s.accent)} title={t("look.customAccent")}>
                  <input type="color" aria-label={t("look.customAccent")} value={s.accent} oninput={(ev) => updateSettings({ accent: ev.currentTarget.value })} />
                  <Plus size={14} />
                </label>
              </div>
            </Row>
          </Group>
          <Group title={t("look.interface")}>
            <Row title={t("look.animations")}><Toggle label={t("look.animations")} checked={s.animations} onchange={set("animations")} /></Row>
            <Row title={t("look.scale")}>
              <div class="range">
                <input type="range" min="0.8" max="1.25" step="0.05" aria-label={t("look.scale")} value={s.uiScale} oninput={(ev) => updateSettings({ uiScale: +ev.currentTarget.value })} />
                <span class="mono">{Math.round(s.uiScale * 100)}%</span>
              </div>
            </Row>
            <Row title={t("look.font")}>
              <Segmented label={t("look.font")} value={s.font} onchange={set("font")} options={[{ value: "inter", label: "Inter" }, { value: "outfit", label: "Outfit" }, { value: "system", label: t("general.system") }, { value: "mono", label: "Mono" }]} />
            </Row>
            <Row title={t("look.radius")}>
              <Segmented label={t("look.radius")} value={s.radius} onchange={set("radius")} options={[{ value: "sharp", label: t("look.radiusSharp") }, { value: "normal", label: t("look.radiusNormal") }, { value: "round", label: t("look.radiusRound") }]} />
            </Row>
            <Row title={t("look.compact")}><Toggle label={t("look.compact")} checked={s.compact} onchange={set("compact")} /></Row>
          </Group>
          <Group title={t("look.home")}>
            <Row title={t("look.followCursor")}><Toggle label={t("look.followCursor")} checked={s.followCursor} onchange={set("followCursor")} /></Row>
            <Row title={t("look.nametag")}><Toggle label={t("look.nametag")} checked={s.showNametag} onchange={set("showNametag")} /></Row>
            <Row title={t("look.homeWallpaper")} stacked>
              <ImagePicker value={s.heroWallpaper} noneLabel={t("look.instanceBanner")} onchange={(v) => updateSettings({ heroWallpaper: v })} />
            </Row>
          </Group>
          <Group title={t("look.appWallpaper")}>
            <Row title={t("look.appWallpaperRow")} stacked>
              <ImagePicker value={s.appWallpaper} onchange={(v) => updateSettings({ appWallpaper: v })} />
            </Row>
          </Group>
        {:else if section === "game"}
          <Group title={t("game.window")}>
            <Row title={t("game.resolution")}>
              <NumberInput label={t("game.width")} min={320} max={15360} value={s.gameWidth} onchange={set("gameWidth")} />
              <span class="dim">×</span>
              <NumberInput label={t("game.height")} min={240} max={8640} value={s.gameHeight} onchange={set("gameHeight")} />
            </Row>
            <Row title={t("game.fullscreen")}><Toggle label={t("game.fullscreen")} checked={s.fullscreen} onchange={set("fullscreen")} /></Row>
          </Group>
          <Group title={t("game.newInstances")}>
            <Row title={t("game.defaultLoader")}>
              <Segmented label={t("game.defaultLoader")} value={s.defaultLoader} onchange={set("defaultLoader")} options={LOADERS.map((l) => ({ value: l.id, label: l.label }))} />
            </Row>
            <Row title={t("game.perfMods")} desc={t("game.perfModsDesc")}><Toggle label={t("game.perfMods")} checked={s.installPerformanceMods} onchange={set("installPerformanceMods")} /></Row>
            <Row title={t("game.snapshots")}><Toggle label={t("game.snapshots")} checked={s.showSnapshots} onchange={set("showSnapshots")} /></Row>
            <Row title={t("game.oldVersions")}><Toggle label={t("game.oldVersions")} checked={s.showOldVersions} onchange={set("showOldVersions")} /></Row>
          </Group>
        {:else if section === "engine"}
          <p class="lead">{t("engine.lead")}</p>
          <Group title={t("engine.performance")}>
            <Row title={t("engine.xwayland")} desc={t("engine.xwaylandDesc")}><Toggle label={t("engine.xwayland")} checked={e.xwayland} onchange={(v) => setEngine({ xwayland: v })} /></Row>
          </Group>
          <Group title={t("engine.graphics")}>
            <Row title={t("engine.lod")} desc={t("engine.lodDesc")}><Select label={t("engine.lod")} value={String(e.lodDistance ?? "")} options={lodOptions} onchange={numeric("lodDistance")} /></Row>
            <Row title={t("engine.resolution")} desc={t("engine.resolutionDesc")}><Select label={t("engine.resolution")} value={String(e.resolutionScale ?? "")} options={scaleOptions} onchange={numeric("resolutionScale")} /></Row>
            <Row title={t("engine.aa")}>
              <Select label={t("engine.aa")} value={e.antialiasing} onchange={(antialiasing) => setEngine({ antialiasing })} options={[{ value: "game", label: t("engine.asGame") }, { value: "off", label: t("common.off") }, { value: "fxaa", label: "FXAA" }, { value: "taa", label: "TAA" }]} />
            </Row>
            <Row title={t("engine.api")} desc={t("engine.apiDesc")}>
              <Select label={t("engine.api")} value={e.graphicsApi} onchange={(graphicsApi) => setEngine({ graphicsApi })} options={[{ value: "game", label: t("engine.asGame") }, { value: "auto", label: t("engine.auto") }, { value: "vulkan", label: "Vulkan" }, { value: "opengl", label: "OpenGL" }]} />
            </Row>
            <Row title="HDR" desc={t("engine.hdrDesc")}><Select label="HDR" value={triple("hdr")} options={onOff} onchange={setTriple("hdr")} /></Row>
            <Row title={t("engine.battery")} desc={t("engine.batteryDesc")}><Select label={t("engine.battery")} value={triple("batteryMode")} options={onOff} onchange={setTriple("batteryMode")} /></Row>
            <Row title={t("engine.fps")} desc={t("engine.fpsDesc")}><Select label={t("engine.fps")} value={triple("fpsOverlay")} options={onOff} onchange={setTriple("fpsOverlay")} /></Row>
          </Group>
        {:else if section === "java"}
          <Group title={t("java.memory")}>
            <Row title={t("java.maxMemory")} desc={t("java.maxMemoryDesc", { gb: ((app.info?.totalMemoryMb ?? 0) / 1024).toFixed(1) })}>
              <MemorySlider value={s.memoryMaxMb} onchange={setMaxMemory} />
            </Row>
            <Row title={t("java.minMemory")}>
              <Select label={t("java.minMemory")} width={120} value={minMemoryValue} options={memoryOptions} onchange={(v) => updateSettings({ memoryMinMb: +v })} />
            </Row>
          </Group>
          <Group title={t("java.runtime")}>
            <Row title={t("java.optimized")} desc={t("java.optimizedDesc")}><Toggle label={t("java.optimized")} checked={s.optimizedJvmArgs} onchange={set("optimizedJvmArgs")} /></Row>
            <Row title={t("java.args")} stacked>
              <input class="input mono" aria-label={t("java.args")} placeholder="-XX:+UseZGC" value={s.jvmArgs} onchange={(ev) => updateSettings({ jvmArgs: ev.currentTarget.value })} />
            </Row>
            <Row title={t("java.path")} desc={t("java.pathDesc")} stacked>
              <div class="java">
                <input class="input mono" aria-label={t("java.path")} placeholder={t("java.auto")} value={s.javaPath} onchange={(ev) => updateSettings({ javaPath: ev.currentTarget.value.trim() })} />
                <button class="btn" onclick={scanJava} disabled={scanning}>{#if scanning}<LoaderCircle size={14} class="spin" />{/if} {t("java.detect")}</button>
                {#if s.javaPath}<button class="btn ghost" onclick={() => updateSettings({ javaPath: "" })}>{t("java.useAuto")}</button>{/if}
              </div>
              {#if javas}
                <div class="javas">
                  {#each javas as j (j.path)}
                    <button class="jv" class:on={s.javaPath === j.path} onclick={() => updateSettings({ javaPath: j.path })}>
                      <span class="major">Java {j.major}</span>
                      <span class="mono path">{j.path}</span>
                      <span class="dim">{t(j.source === "bundled" ? "java.bundled" : "java.system")}</span>
                    </button>
                  {:else}
                    <div class="dim">{t("java.none")}</div>
                  {/each}
                </div>
              {/if}
            </Row>
          </Group>
        {:else if section === "advanced"}
          <Group title={t("advanced.tools")}>
            <Row title="Feral GameMode" desc={t("advanced.gamemodeDesc")}><Toggle label="Feral GameMode" checked={s.gamemode} onchange={set("gamemode")} /></Row>
            <Row title="MangoHud" desc={t("advanced.mangohudDesc")}><Toggle label="MangoHud" checked={s.mangohud} onchange={set("mangohud")} /></Row>
            <Row title={t("advanced.gpu")} desc={t("advanced.gpuDesc")}><Toggle label={t("advanced.gpu")} checked={s.discreteGpu} onchange={set("discreteGpu")} /></Row>
          </Group>
          <Group title={t("advanced.commands")}>
            <Row title={t("advanced.wrapper")} desc={t("advanced.wrapperDesc")} stacked>
              <input class="input mono" aria-label={t("advanced.wrapper")} placeholder={t("common.none")} value={s.wrapperCommand} onchange={(ev) => updateSettings({ wrapperCommand: ev.currentTarget.value })} />
            </Row>
            <Row title={t("advanced.env")} desc={t("advanced.envDesc")} stacked>
              <textarea class="input" aria-label={t("advanced.env")} rows="3" placeholder="__GL_THREADED_OPTIMIZATIONS=0" value={s.envVars} onchange={(ev) => updateSettings({ envVars: ev.currentTarget.value })}></textarea>
            </Row>
            <Row title={t("advanced.pre")} desc={t("advanced.preDesc")} stacked>
              <input class="input mono" aria-label={t("advanced.pre")} placeholder={t("common.none")} value={s.preLaunchCommand} onchange={(ev) => updateSettings({ preLaunchCommand: ev.currentTarget.value })} />
            </Row>
            <Row title={t("advanced.post")} stacked>
              <input class="input mono" aria-label={t("advanced.post")} placeholder={t("common.none")} value={s.postExitCommand} onchange={(ev) => updateSettings({ postExitCommand: ev.currentTarget.value })} />
            </Row>
          </Group>
        {:else if section === "accounts"}
          <Group>
            {#each app.accounts as a (a.uuid)}
              <div class="acc">
                <Head skinUrl={a.skinUrl} name={a.name} size={32} radius={6} />
                <div class="acc-text">
                  <b>{a.name}</b>
                  <small class="dim mono">{a.uuid}</small>
                </div>
                {#if a.active}
                  <span class="dim active">{t("account.active")}</span>
                {:else}
                  <button class="btn sm" onclick={() => api.setActiveAccount(a.uuid).then((l) => (app.accounts = l)).catch(toastError)}>{t("account.setActive")}</button>
                {/if}
                <button class="btn sm ghost danger" onclick={() => logout(a.uuid)}><LogOut size={14} /> {t("account.logout")}</button>
              </div>
            {/each}
            <div class="add"><button class="btn" onclick={() => (app.addAccountOpen = true)}><Plus size={14} /> {t("account.addMicrosoft")}</button></div>
          </Group>
        {:else}
          <div class="about">
            <img src="/icon.svg" alt="" width="48" height="48" />
            <div>
              <b>Patina {app.info?.version}</b>
              <div class="dim">{app.info?.os} / {app.info?.arch}</div>
            </div>
          </div>
          <p class="lead">{t("about.credit")}</p>
          <Group>
            <Row title={t("about.dataDir")} desc={app.info?.dataDir}>
              <button class="btn" onclick={() => api.openFolder().catch(toastError)}><FolderOpen size={14} /> {t("common.open")}</button>
            </Row>
            <Row title={t("about.guide")}>
              <button class="btn" onclick={() => { updateSettings({ onboarded: false }); toast(t("about.guideShown")); }}>{t("common.show")}</button>
            </Row>
          </Group>
        {/if}
      </div>
    </div>
  {/key}
</div>

<style>
  .settings {
    height: 100%;
    display: flex;
    min-height: 0;
  }
  nav {
    position: relative;
    width: 188px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px;
    overflow-y: auto;
    border-right: 1px solid var(--border);
  }
  .nav-label {
    padding: 12px 8px 4px;
    color: var(--text-3);
    font-size: var(--fs-12);
    font-weight: 500;
  }
  .nav-label:nth-of-type(1) { padding-top: 4px; }
  .about-link { margin-top: auto; }
  .pane {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 24px 24px 48px;
  }
  .column { max-width: 640px; margin: 0 auto; }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    min-height: 28px;
    margin-bottom: 16px;
  }
  .save { color: var(--text-3); font-size: var(--fs-12); }
  .save.error { color: var(--danger); text-decoration: underline; text-underline-offset: 3px; }
  .lead { margin: 0 0 24px; color: var(--text-2); font-size: var(--fs-13); line-height: 1.5; }
  .value { color: var(--text-2); font-size: var(--fs-13); }
  .range {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 240px;
  }
  .range input { flex: 1; min-width: 0; }
  .range span { width: 40px; font-size: var(--fs-12); text-align: right; }
  .accents { display: flex; flex-wrap: wrap; gap: 8px; }
  .sw {
    position: relative;
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    color: #101214;
    box-shadow: inset 0 0 0 1px rgb(128 128 128 / 0.3);
    cursor: pointer;
  }
  .sw[aria-pressed="true"], .sw.on { outline: 2px solid var(--text); outline-offset: 2px; }
  .custom { color: var(--text-2); border: 1px dashed var(--border-2); box-shadow: none; }
  .custom input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .java { display: flex; gap: 8px; }
  .java .input { flex: 1; min-width: 0; }
  .javas { display: flex; flex-direction: column; gap: 4px; width: 100%; }
  .jv {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 4px 8px;
    border-radius: var(--radius-s);
    text-align: left;
    font-size: var(--fs-12);
    border: 1px solid var(--border);
    transition: border-color 0.12s;
  }
  .jv:hover { border-color: var(--border-2); }
  .jv.on { border-color: var(--accent); }
  .major { font-weight: 600; white-space: nowrap; }
  .path { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .acc {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 56px;
    border-bottom: 1px solid var(--border);
  }
  .acc-text { flex: 1; display: flex; flex-direction: column; min-width: 0; }
  .acc-text b { font-size: var(--fs-13); font-weight: 500; }
  .acc-text small { font-size: var(--fs-12); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .active { font-size: var(--fs-12); }
  .add { padding-top: 12px; }
  .about { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
  .about b { font-weight: 600; }
  .about .dim { font-size: var(--fs-12); }
</style>
