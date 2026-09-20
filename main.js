const { app, BrowserWindow, ipcMain, shell, dialog } = require('electron');
const path = require('path');
const fs = require('fs');
const os = require('os');
const namesLib = require('./names');
const discord = require('./discord');

const USER_DIR = app.getPath('userData');
const ACCOUNTS_FILE = path.join(USER_DIR, 'accounts.json');
const SETTINGS_FILE = path.join(USER_DIR, 'settings.json');
const VERSIONS_CACHE_FILE = path.join(USER_DIR, 'versions-cache.json');
const VERSIONS_CACHE_TTL = 6 * 60 * 60 * 1000; // 6h

function readJSON(file, fallback) {
  try {
    if (!fs.existsSync(file)) return fallback;
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch { return fallback; }
}
function writeJSON(file, data) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(data, null, 2));
}

function listAllJavas() {
  const found = [];
  const push = (p) => { try { if (p && fs.existsSync(p) && !found.includes(p)) found.push(p); } catch {} };
  // JAVA_HOME / JDK_HOME
  try {
    for (const env of ['JAVA_HOME', 'JDK_HOME']) {
      const jh = process.env[env];
      if (jh) {
        push(path.join(jh, 'bin', 'javaw.exe'));
        push(path.join(jh, 'bin', 'java.exe'));
        push(path.join(jh, 'bin', 'java'));
      }
    }
  } catch {}
  try {
    // PATH lookup (portable, no hardcoded usernames/versions)
    const cmd = process.platform === 'win32' ? 'where java' : 'which -a java';
    const out = require('child_process').execSync(cmd, { stdio: ['ignore', 'pipe', 'ignore'] }).toString();
    for (const line of out.split(/\r?\n/)) {
      const p = line.trim();
      if (p) push(p);
    }
  } catch {}
  try {
    const bases = process.platform === 'win32'
      ? ['C:\\Program Files\\Eclipse Adoptium', 'C:\\Program Files\\Microsoft', 'C:\\Program Files\\Java', 'C:\\Program Files (x86)\\Java', 'C:\\Program Files\\Temurin', 'C:\\Program Files\\BellSoft', 'C:\\Program Files\\Zulu']
      : ['/usr/lib/jvm', '/opt', '/Library/Java/JavaVirtualMachines'];
    for (const base of bases) {
      if (!fs.existsSync(base)) continue;
      for (const d of fs.readdirSync(base)) {
        push(path.join(base, d, 'bin', 'javaw.exe'));
        push(path.join(base, d, 'bin', 'java.exe'));
        push(path.join(base, d, 'bin', 'java'));
      }
    }
  } catch {}
  try {
    // Mojang-bundled runtimes from previous installs (any game dir + vanilla default)
    const appData = app.getPath('appData');
    for (const gd of [readJSON(SETTINGS_FILE, {}).gameDir, path.join(appData, '.cinder-minecraft'), path.join(appData, '.minecraft')]) {
      if (!gd) continue;
      const rt = path.join(gd, 'runtime');
      if (!fs.existsSync(rt)) continue;
      const walk = (dir, depth) => {
        if (depth > 4) return;
        let entries = [];
        try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
        for (const e of entries) {
          const p = path.join(dir, e.name);
          if (e.isFile() && /^java(w)?(\.exe)?$/i.test(e.name)) push(p);
          else if (e.isDirectory()) walk(p, depth + 1);
        }
      };
      walk(rt, 0);
    }
  } catch {}
  const majorOf = (p) => {
    const m = p.match(/jdk-(\d+)/i) || p.match(/java[_-]?(\d+)/i) || p.match(/(\d+)[.\-]/);
    return m ? parseInt(m[1], 10) : 0;
  };
  return found
    .map(p => ({ path: p, major: majorOf(p) }))
    .sort((a, b) => b.major - a.major);
}

function detectJava() {
  const all = listAllJavas();
  if (all.length) return all[0].path;
  return 'java';
}

function requiredJavaForVersion(version) {
  const v = String(version || '');
  const m = v.match(/^(\d+)\.(\d+)(?:\.(\d+))?/);
  if (m) {
    const major = parseInt(m[1], 10), minor = parseInt(m[2], 10), patch = parseInt(m[3] || '0', 10);
    if (major >= 26 || major === 25) return 25;
    if (major >= 24) return 21;
    if (major === 1) {
      if (minor > 20 || (minor === 20 && patch >= 5)) return 21;
      if (minor >= 18) return 17;
      if (minor === 17) return 16;
      return 8;
    }
    return 21;
  }
  return 17;
}

function getJavaForVersion(version, preferred, autoJava = true) {
  const need = requiredJavaForVersion(version);
  const all = listAllJavas();
  const majorOfPath = (p) => {
    const it = all.find(j => j.path === p);
    return it ? it.major : 0;
  };
  if (!autoJava && preferred && preferred !== 'java' && fs.existsSync(preferred)) return preferred;
  if (preferred && preferred !== 'java' && fs.existsSync(preferred) && majorOfPath(preferred) >= need) return preferred;
  const ok = all.filter(j => j.major >= need).sort((a, b) => a.major - b.major);
  if (ok.length) return ok[0].path;
  if (all.length) return all[0].path;
  return preferred || 'java';
}

function getDefaultSettings() {
  return {
    gameDir: path.join(app.getPath('appData'), '.cinder-minecraft'),
    javaPath: detectJava(),
    autoJava: true,
    minRam: '2G',
    maxRam: '4G',
    width: 854,
    height: 480,
    fullscreen: false,
    jvmArgs: '',
    serverHost: '',
    recentServers: [],
    lastAccountId: null,
    lastVersion: null,
    loader: 'vanilla',
    loaderProfiles: {},
    discordPresence: true,
    discordClientId: '',
    discordDownloadUrl: 'https://github.com/radoslavgeme/cinder',
    launchAction: 'minimize', // 'minimize' | 'hide' | 'nothing'
    launchTimeoutSec: 30,
    accent: 'indigo', // 'indigo' | 'mint' | 'ember' | 'violet' | 'glacier' | 'rose'
    reduceMotion: false,
    logsFollow: true,
    showDebugLogs: true,
    jvmPreset: 'balanced', // 'balanced' | 'performance' | 'latency'
    showSnow: true,
    checkUpdates: true
  };
}

function sanitizeSettings(s) {
  const out = { ...s };
  if (out.width !== undefined) out.width = Math.max(640, Math.min(3840, parseInt(out.width, 10) || 854));
  if (out.height !== undefined) out.height = Math.max(480, Math.min(2160, parseInt(out.height, 10) || 480));
  const ramOk = (v) => /^[0-9]+(\.[0-9]+)?[GM]$/i.test(String(v || '').trim());
  if (out.minRam !== undefined && !ramOk(out.minRam)) out.minRam = '2G';
  if (out.maxRam !== undefined && !ramOk(out.maxRam)) out.maxRam = '4G';
  if (out.minRam) out.minRam = String(out.minRam).trim().toUpperCase();
  if (out.maxRam) out.maxRam = String(out.maxRam).trim().toUpperCase();
  if (out.jvmArgs !== undefined) out.jvmArgs = String(out.jvmArgs || '').slice(0, 500);
  if (out.serverHost !== undefined) {
    let h = String(out.serverHost || '').trim().slice(0, 255);
    h = h.replace(/^(https?:\/\/|mc:\/\/)/i, '').replace(/\/+$/, '').replace(/\s+/g, '');
    out.serverHost = h;
  }
  if (out.gameDir !== undefined) out.gameDir = String(out.gameDir || '').slice(0, 500);
  if (out.javaPath !== undefined) out.javaPath = String(out.javaPath || '').slice(0, 500);
  if (!Array.isArray(out.recentServers)) out.recentServers = [];
  out.recentServers = out.recentServers.map(String).map(x => x.trim()).filter(Boolean).slice(0, 8);
  if (out.discordPresence !== undefined) out.discordPresence = !!out.discordPresence;
  if (out.discordClientId !== undefined) out.discordClientId = String(out.discordClientId || '').trim().slice(0, 64);
  if (out.discordDownloadUrl !== undefined) {
    const u = String(out.discordDownloadUrl || '').trim().slice(0, 200);
    out.discordDownloadUrl = /^https?:\/\/.+/i.test(u) ? u : '';
  }
  // Migrate the old minimize checkbox to the launch-action select.
  if (!['minimize', 'hide', 'nothing'].includes(out.launchAction)) {
    out.launchAction = out.closeOnLaunch === false ? 'nothing' : 'minimize';
  }
  delete out.closeOnLaunch;
  const t = parseInt(out.launchTimeoutSec, 10);
  out.launchTimeoutSec = Math.max(10, Math.min(300, Number.isFinite(t) ? t : 30));
  if (!['indigo', 'mint', 'ember', 'violet', 'glacier', 'rose'].includes(out.accent)) out.accent = 'indigo';
  if (out.reduceMotion !== undefined) out.reduceMotion = !!out.reduceMotion;
  if (out.logsFollow !== undefined) out.logsFollow = !!out.logsFollow;
  if (out.showDebugLogs !== undefined) out.showDebugLogs = !!out.showDebugLogs;
  if (!['balanced', 'performance', 'latency'].includes(out.jvmPreset)) out.jvmPreset = 'balanced';
  if (out.showSnow !== undefined) out.showSnow = !!out.showSnow;
  if (out.checkUpdates !== undefined) out.checkUpdates = !!out.checkUpdates;
  return out;
}

function createWindow() {
  let icon;
  try {
    // Real .ico on Windows (taskbar + shortcuts), SVG elsewhere.
    const ico = path.join(__dirname, 'assets', 'icon.ico');
    const svg = path.join(__dirname, 'brand', 'logo.svg');
    if (process.platform === 'win32' && fs.existsSync(ico)) icon = ico;
    else if (fs.existsSync(svg)) icon = svg;
  } catch {}
  const win = new BrowserWindow({
    width: 1180,
    height: 800,
    minWidth: 920,
    minHeight: 640,
    backgroundColor: '#070b0e',
    title: 'Cinder',
    icon,
    autoHideMenuBar: true,
    frame: false,
    titleBarStyle: 'hidden',
    trafficLightPosition: { x: 14, y: 14 },
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });
  win.loadFile('index.html');
  if (process.argv.includes('--dev')) win.webContents.openDevTools();
  return win;
}

// ---- Custom frameless titlebar ----
ipcMain.handle('window:min', () => { const w = BrowserWindow.getAllWindows()[0]; if (w) w.minimize(); });
ipcMain.handle('window:max', () => {
  const w = BrowserWindow.getAllWindows()[0];
  if (!w) return false;
  if (w.isMaximized()) w.unmaximize(); else w.maximize();
  return w.isMaximized();
});
ipcMain.handle('window:close', () => { const w = BrowserWindow.getAllWindows()[0]; if (w) w.close(); });

app.whenReady().then(() => {
  createWindow();
  try {
    const s = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
    discord.init({ settings: s, log: (m) => { const w = BrowserWindow.getAllWindows()[0]; if (w) w.webContents.send('game:log', m); } });
    // Show launcher presence once UI is up (renderer also pushes screen updates).
    setTimeout(() => {
      try {
        discord.setLauncher({
          account: (readJSON(ACCOUNTS_FILE, []).find(a => a.id === s.lastAccountId) || {}).name || null,
          version: s.lastVersion || null,
          screen: 'play'
        });
      } catch {}
    }, 2500);
  } catch {}
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
app.on('window-all-closed', () => { try { discord.shutdown(); } catch {} if (process.platform !== 'darwin') app.quit(); });

// ---- Accounts ----
ipcMain.handle('accounts:list', () => readJSON(ACCOUNTS_FILE, []));
ipcMain.handle('accounts:add', (_, name) => {
  name = String(name || '').trim().slice(0, 16);
  if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) throw new Error('Name must be 3-16 chars: A-Z, 0-9, _');
  const list = readJSON(ACCOUNTS_FILE, []);
  if (list.find(a => a.name.toLowerCase() === name.toLowerCase())) throw new Error('Account already exists');
  const acc = { id: Date.now().toString(), name, type: 'offline', created: new Date().toISOString() };
  list.push(acc);
  writeJSON(ACCOUNTS_FILE, list);
  return list;
});
ipcMain.handle('accounts:microsoft-login', async () => {
  const win = BrowserWindow.getAllWindows()[0];
  const send = (msg) => win && win.webContents.send('game:log', msg);
  try {
    const { Auth } = require('msmc');
    send('> Opening Microsoft login...');
    const authManager = new Auth('select_account');
    const xboxManager = await authManager.launch('electron', { width: 520, height: 680 });
    send('> Microsoft OK, getting Minecraft profile...');
    const token = await xboxManager.getMinecraft();
    const profile = token.profile;
    if (!profile || !profile.name) throw new Error('No Minecraft profile found on this Microsoft account. Buy Minecraft Java or check account.');
    const list = readJSON(ACCOUNTS_FILE, []);
    const mcToken = token.mclc(true);
    let acc = list.find(a => a.type === 'microsoft' && a.uuid === profile.id);
    if (acc) {
      acc.name = profile.name;
      acc.mcToken = mcToken;
    } else {
      acc = { id: Date.now().toString(), name: profile.name, uuid: profile.id, type: 'microsoft', mcToken, created: new Date().toISOString() };
      list.push(acc);
    }
    writeJSON(ACCOUNTS_FILE, list);
    send(`> Logged in as ${profile.name} (premium)`);
    return list;
  } catch (err) {
    const msg = String(err.message || err);
    if (msg.includes('Window was closed') || msg.includes('closed')) throw new Error('Login cancelled');
    throw new Error('Microsoft login failed: ' + msg.slice(0, 300));
  }
});
ipcMain.handle('accounts:remove', (_, id) => {
  const list = readJSON(ACCOUNTS_FILE, []).filter(a => a.id !== id);
  writeJSON(ACCOUNTS_FILE, list);
  return list;
});
// ---- 3D skin texture (fetched + cached in main, no CORS issues for WebGL) ----
ipcMain.handle('skin:get', async (_, name) => {
  const safe = String(name || 'Steve').replace(/[^A-Za-z0-9_]/g, '').slice(0, 16) || 'Steve';
  const dir = path.join(USER_DIR, 'skins');
  const file = path.join(dir, safe.toLowerCase() + '.png');
  try {
    const st = fs.existsSync(file) ? fs.statSync(file) : null;
    if (!st || Date.now() - st.mtimeMs > 24 * 60 * 60 * 1000) {
      const res = await fetch(`https://minotar.net/skin/${encodeURIComponent(safe)}.png`);
      if (!res.ok) throw new Error('skin fetch ' + res.status);
      const buf = Buffer.from(await res.arrayBuffer());
      if (buf.length < 100) throw new Error('bad skin bytes');
      fs.mkdirSync(dir, { recursive: true });
      fs.writeFileSync(file, buf);
      try {
        const files = fs.readdirSync(dir)
          .map((f) => { try { return { f, t: fs.statSync(path.join(dir, f)).mtimeMs }; } catch { return null; } })
          .filter(Boolean)
          .sort((a, b) => b.t - a.t);
        for (const old of files.slice(200)) {
          try { fs.unlinkSync(path.join(dir, old.f)); } catch {}
        }
      } catch {}
    }
    return 'data:image/png;base64,' + fs.readFileSync(file).toString('base64');
  } catch {
    return null;
  }
});
ipcMain.handle('accounts:suggest-names', (_, opts = {}) => {  const taken = readJSON(ACCOUNTS_FILE, []).map(a => a.name);
  const count = Math.max(1, Math.min(12, parseInt(opts.count, 10) || 6));
  const style = ['ember', 'hero', 'dark', 'cute', 'random', 'mixed'].includes(opts.style) ? opts.style : 'mixed';
  return namesLib.suggestNames(count, { taken, style });
});
ipcMain.handle('presence:update', async (_, p = {}) => {
  try {
    const s = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
    // (Re)connect first: boot-time login may have failed (Discord was closed)
    // or the ID changed since. init() is a no-op when already connected.
    try { await discord.init({ settings: s }); } catch {}
    if (p.inGame) return discord.setGame(p);
    return discord.setScreen(p.screen || 'play', { account: p.account, version: p.version });
  } catch { return false; }
});
ipcMain.handle('presence:status', () => discord.status());

async function getMicrosoftAuth(account) {
  const { Auth, tokenUtils } = require('msmc');
  const list = readJSON(ACCOUNTS_FILE, []);
  const stored = list.find(a => a.id === account.id) || account;
  if (!stored.mcToken) throw new Error('No Microsoft token saved — please login again');
  const auth = new Auth('select_account');
  let mc = tokenUtils.fromMclcToken(auth, stored.mcToken);
  try {
    mc = await mc.refresh(true);
  } catch (e) {
    throw new Error('Microsoft session expired — please login again, then retry. (' + (e.message || e).toString().slice(0, 120) + ')');
  }
  const idx = list.findIndex(a => a.id === stored.id);
  if (idx >= 0) {
    list[idx].mcToken = mc.mclc(true);
    list[idx].name = mc.profile?.name || list[idx].name;
    try { writeJSON(ACCOUNTS_FILE, list); } catch {}
  }
  return mc.mclc();
}

// ---- Settings / system ----
ipcMain.handle('settings:get', () => ({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) }));
ipcMain.handle('settings:save', (_, s) => {
  const cur = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}), ...s });
  writeJSON(SETTINGS_FILE, cur);
  // Re-init Discord presence if its settings changed (cheap, no-ops when disabled).
  try { discord.init({ settings: cur }); } catch {}
  return cur;
});
ipcMain.handle('java:list', (_, version) => {
  const javas = listAllJavas();
  return { required: version ? requiredJavaForVersion(version) : null, javas };
});
ipcMain.handle('system:info', () => ({
  totalGB: +(os.totalmem() / 1024 ** 3).toFixed(1),
  freeGB: +(os.freemem() / 1024 ** 3).toFixed(1),
  platform: process.platform
}));
ipcMain.handle('dialog:pick-dir', async (_, current) => {
  const win = BrowserWindow.getAllWindows()[0];
  const r = await dialog.showOpenDialog(win, {
    title: 'Choose game directory',
    defaultPath: current || app.getPath('appData'),
    properties: ['openDirectory', 'createDirectory']
  });
  if (r.canceled || !r.filePaths.length) return null;
  return r.filePaths[0];
});
ipcMain.handle('dialog:pick-java', async () => {
  const win = BrowserWindow.getAllWindows()[0];
  const r = await dialog.showOpenDialog(win, {
    title: 'Choose java.exe / javaw.exe',
    filters: [{ name: 'Java', extensions: ['exe'] }],
    properties: ['openFile']
  });
  if (r.canceled || !r.filePaths.length) return null;
  return r.filePaths[0];
});

// ---- Versions (Mojang piston-meta, cached) ----
function installedVersions(gameDir) {
  try {
    const dir = path.join(gameDir || getDefaultSettings().gameDir, 'versions');
    if (!fs.existsSync(dir)) return new Set();
    return new Set(fs.readdirSync(dir).filter(d => {
      try { return fs.existsSync(path.join(dir, d, d + '.json')); } catch { return false; }
    }));
  } catch { return new Set(); }
}

ipcMain.handle('versions:list', async (_, opts = {}) => {
  const gameDir = opts.gameDir || readJSON(SETTINGS_FILE, {}).gameDir || getDefaultSettings().gameDir;
  const useCache = !opts.force;
  if (useCache) {
    try {
      const cached = readJSON(VERSIONS_CACHE_FILE, null);
      if (cached && cached.time && (Date.now() - cached.time < VERSIONS_CACHE_TTL) && cached.data) {
        const installed = installedVersions(gameDir);
        return {
          ...cached.data,
          fromCache: true,
          versions: cached.data.versions.map(v => ({ ...v, installed: installed.has(v.id) }))
        };
      }
    } catch {}
  }
  const res = await fetch('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json');
  if (!res.ok) {
    // fall back to stale cache on failure
    const cached = readJSON(VERSIONS_CACHE_FILE, null);
    if (cached && cached.data) {
      const installed = installedVersions(gameDir);
      return { ...cached.data, fromCache: true, stale: true, versions: cached.data.versions.map(v => ({ ...v, installed: installed.has(v.id) })) };
    }
    throw new Error('Failed to fetch version list: ' + res.status);
  }
  const data = await res.json();
  const payload = {
    latest: data.latest,
    versions: data.versions.slice(0, 100).map(v => ({ id: v.id, type: v.type, url: v.url, time: v.releaseTime }))
  };
  try { writeJSON(VERSIONS_CACHE_FILE, { time: Date.now(), data: payload }); } catch {}
  const installed = installedVersions(gameDir);
  return { ...payload, fromCache: false, versions: payload.versions.map(v => ({ ...v, installed: installed.has(v.id) })) };
});

ipcMain.handle('versions:uninstall', async (_, { gameDir, id }) => {
  const safe = String(id || '').replace(/[^A-Za-z0-9._\-+]/g, '');
  if (!safe || safe === '.' || safe === '..') throw new Error('Invalid version');
  const dir = path.join(gameDir, 'versions', safe);
  if (!fs.existsSync(dir)) throw new Error('Version not installed');
  fs.rmSync(dir, { recursive: true, force: true });
  return true;
});

// ---- Mods folder ----
function readMods(gameDir) {
  const modsDir = path.join(gameDir || getDefaultSettings().gameDir, 'mods');
  if (!fs.existsSync(modsDir)) return [];
  return fs.readdirSync(modsDir)
    .filter(f => f.endsWith('.jar') || f.endsWith('.jar.disabled') || f.endsWith('.disabled'))
    .map(f => {
      const st = fs.statSync(path.join(modsDir, f));
      const enabled = f.endsWith('.jar');
      const display = enabled ? f : f.replace(/\.disabled$/, '');
      return { name: f, display, size: st.size, enabled };
    });
}
ipcMain.handle('mods:list', (_, gameDir) => readMods(gameDir));
ipcMain.handle('mods:toggle', (_, { gameDir, name }) => {
  const modsDir = path.join(gameDir, 'mods');
  let src = path.join(modsDir, path.basename(String(name || '')));
  // Tolerate stale UI name after a prior toggle: try .disabled variants.
  if (!fs.existsSync(src)) {
    const base = path.basename(String(name || ''));
    const alts = [];
    if (base.endsWith('.jar')) alts.push(base + '.disabled');
    if (base.endsWith('.jar.disabled')) alts.push(base.slice(0, -'.disabled'.length));
    if (base.endsWith('.disabled')) alts.push(base.slice(0, -'.disabled'.length));
    for (const a of alts) {
      const p = path.join(modsDir, a);
      if (fs.existsSync(p)) { src = p; break; }
    }
  }
  if (!fs.existsSync(src)) return readMods(gameDir); // already changed elsewhere — just refresh
  let dest;
  if (src.endsWith('.jar')) dest = src + '.disabled';
  else if (src.endsWith('.disabled')) dest = src.slice(0, -'.disabled'.length);
  else throw new Error('Invalid file');
  if (fs.existsSync(dest)) throw new Error('Target already exists');
  fs.renameSync(src, dest);
  return readMods(gameDir);
});
ipcMain.handle('folder:open', (_, dir) => {
  const target = path.resolve(String(dir || ''));
  return shell.openPath(target);
});
ipcMain.handle('link:open', (_, url) => {
  const u = String(url || '').trim().slice(0, 500);
  if (!/^https:\/\/[A-Za-z0-9._\-/()?=&%#]+$/.test(u)) throw new Error('Only https links can be opened');
  return shell.openExternal(u);
});

// ---- Mod loader install + compat check ----
const loaderLib = require('./loader');

ipcMain.handle('loader:install', async (_, { loader, mc, gameDir }) => {
  const win = BrowserWindow.getAllWindows()[0];
  const send = (m) => win && win.webContents.send('game:log', m);
  const { profileId, loaderVersion } = await loaderLib.installLoader({ loader, mc, gameDir, log: send });
  const cur = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
  cur.loader = loader;
  cur.loaderProfiles = { ...(cur.loaderProfiles || {}), [`${loader}:${mc}`]: profileId };
  writeJSON(SETTINGS_FILE, cur);
  return { profileId, loaderVersion, settings: cur };
});

ipcMain.handle('mods:check', async (_, { gameDir, mc, launcherLoader }) => {
  return loaderLib.checkInstalledMods(path.join(gameDir, 'mods'), { launcherLoader, mc });
});

// ---- Mods: Modrinth browser + local .jar management ----
const MODRINTH_HEADERS = { 'User-Agent': 'Cinder/1.0 (github:cinder-launcher)' };

ipcMain.handle('mods:search', async (_, query) => {
  query = String(query || '').trim().slice(0, 100);
  if (!query) return [];
  const facets = encodeURIComponent(JSON.stringify([['project_type:mod']]));
  const res = await fetch(`https://api.modrinth.com/v2/search?query=${encodeURIComponent(query)}&limit=20&index=relevance&facets=${facets}`, { headers: MODRINTH_HEADERS });
  if (!res.ok) throw new Error('Modrinth search failed: ' + res.status);
  const data = await res.json();
  return (data.hits || []).map(h => ({
    id: h.project_id, slug: h.slug, title: h.title,
    desc: h.description, icon: h.icon_url, downloads: h.downloads
  }));
});

ipcMain.handle('mods:project-versions', async (_, { projectId, gameVersion, loader }) => {
  const q = (gv) => {
    const p = new URLSearchParams();
    if (loader) p.set('loaders', JSON.stringify([loader]));
    if (gv) p.set('game_versions', JSON.stringify([gv]));
    return p.toString();
  };
  let res = await fetch(`https://api.modrinth.com/v2/project/${projectId}/version?${q(gameVersion)}`, { headers: MODRINTH_HEADERS });
  if (!res.ok) throw new Error('Failed to get mod versions: ' + res.status);
  let list = await res.json();
  let fallback = false;
  if (!list.length && gameVersion) {
    res = await fetch(`https://api.modrinth.com/v2/project/${projectId}/version?${q(null)}`, { headers: MODRINTH_HEADERS });
    if (res.ok) { list = await res.json(); fallback = list.length > 0; }
  }
  return {
    fallback,
    versions: list.slice(0, 10).map(v => {
      const f = (v.files || []).find(f => f.primary) || (v.files || [])[0];
      return {
        id: v.id, number: v.version_number,
        game: v.game_versions || [], loaders: v.loaders || [],
        file: f ? { url: f.url, name: f.filename, size: f.size } : null
      };
    }).filter(v => v.file)
  };
});

ipcMain.handle('mods:download', async (_, { fileUrl, fileName, gameDir }) => {
  const win = BrowserWindow.getAllWindows()[0];
  const send = (m) => win && win.webContents.send('game:log', m);
  const safe = path.basename(String(fileName || '').replace(/[^A-Za-z0-9._\-+]/g, '_'));
  if (!safe.endsWith('.jar')) throw new Error('Not a .jar file');
  const modsDir = path.join(gameDir, 'mods');
  fs.mkdirSync(modsDir, { recursive: true });
  send(`> Downloading mod ${safe}...`);
  const res = await fetch(fileUrl, { headers: MODRINTH_HEADERS });
  if (!res.ok) throw new Error('Download failed: ' + res.status);
  const buf = Buffer.from(await res.arrayBuffer());
  fs.writeFileSync(path.join(modsDir, safe), buf);
  send(`> Installed ${safe} (${(buf.length / 1024).toFixed(0)} KB) — loader auto-installs on Launch if needed.`);
  return { ok: true, name: safe };
});

ipcMain.handle('mods:delete', async (_, { gameDir, name }) => {
  const safe = path.basename(String(name || ''));
  if (!safe.includes('.jar')) throw new Error('Invalid file');
  const modsDir = path.join(gameDir, 'mods');
  // Try exact name first, then .disabled variants (handles stale UI after toggle / double-click).
  const candidates = [safe];
  if (safe.endsWith('.jar')) candidates.push(safe + '.disabled');
  if (safe.endsWith('.jar.disabled')) candidates.push(safe.slice(0, -'.disabled'.length));
  if (safe.endsWith('.disabled')) candidates.push(safe.slice(0, -'.disabled'.length));
  for (const c of candidates) {
    const p = path.join(modsDir, c);
    if (fs.existsSync(p)) {
      fs.unlinkSync(p);
      return true;
    }
  }
  // Already gone (double-click / refreshed list) — treat as success so UI just refreshes.
  return true;
});

ipcMain.handle('mods:import-jar', async (_, gameDir) => {
  const win = BrowserWindow.getAllWindows()[0];
  const r = await dialog.showOpenDialog(win, {
    title: 'Pick .jar mod files',
    filters: [{ name: 'Minecraft mods', extensions: ['jar'] }],
    properties: ['openFile', 'multiSelections']
  });
  if (r.canceled || !r.filePaths.length) return [];
  const modsDir = path.join(gameDir, 'mods');
  fs.mkdirSync(modsDir, { recursive: true });
  const added = [];
  for (const src of r.filePaths) {
    const dest = path.join(modsDir, path.basename(src));
    fs.copyFileSync(src, dest);
    added.push(path.basename(src));
  }
  return added;
});

// ---- Worlds (singleplayer saves: list / backup / restore / delete) ----
function safeWorldName(name) {
  const raw = String(name == null ? '' : name);
  const base = path.basename(raw);
  if (!base || base === '.' || base === '..') throw new Error('Invalid world name');
  if (raw !== base || raw.includes('/') || raw.includes('\\')) throw new Error('Invalid world name');
  if (base.length > 64) throw new Error('Invalid world name');
  if (/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\.|$)/i.test(base)) throw new Error('Invalid world name');
  return base;
}

function worldsAssertInside(savesDir, p) {
  const root = path.resolve(savesDir);
  const target = path.resolve(p);
  if (target !== root && !target.startsWith(root + path.sep)) throw new Error('Invalid world path');
}

ipcMain.handle('worlds:list', (_, gameDir) => {
  try {
    const saves = path.join(String(gameDir || ''), 'saves');
    if (!gameDir || !path.isAbsolute(saves) || !fs.existsSync(saves)) return [];
    const out = [];
    let truncated = false;
    for (const entry of fs.readdirSync(saves, { withFileTypes: true })) {
      try {
        if (!entry.isDirectory()) continue;
        const dir = path.join(saves, entry.name);
        if (!fs.existsSync(path.join(dir, 'level.dat'))) continue;
        let size = 0, files = 0;
        const walk = (d, depth) => {
          if (depth > 8 || files > 50000) { truncated = true; return; }
          let entries = [];
          try { entries = fs.readdirSync(d, { withFileTypes: true }); } catch { return; }
          for (const e of entries) {
            if (files > 50000) { truncated = true; return; }
            const p = path.join(d, e.name);
            let st = null;
            try { st = fs.lstatSync(p); } catch { continue; }
            if (st.isSymbolicLink()) continue;
            if (st.isDirectory()) walk(p, depth + 1);
            else if (st.isFile()) { size += st.size; files += 1; }
          }
        };
        walk(dir, 0);
        let mtime = 0;
        try { mtime = fs.statSync(path.join(dir, 'level.dat')).mtimeMs; } catch {}
        out.push({ name: entry.name, size, mtime, truncated });
      } catch {}
    }
    out.sort((a, b) => a.name.localeCompare(b.name));
    return out;
  } catch { return []; }
});

ipcMain.handle('worlds:backup', (_, p = {}) => {
  const gameDir = String(p.gameDir || '');
  const safe = safeWorldName(p.name);
  if (!gameDir) throw new Error('No game directory set');
  const src = path.join(gameDir, 'saves', safe);
  worldsAssertInside(path.join(gameDir, 'saves'), src);
  if (!fs.existsSync(src)) throw new Error('World not found: ' + safe);
  const AdmZip = require('adm-zip');
  const backups = path.join(gameDir, 'backups');
  fs.mkdirSync(backups, { recursive: true });
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  const stamp = d.getFullYear() + pad(d.getMonth() + 1) + pad(d.getDate()) + '-' + pad(d.getHours()) + pad(d.getMinutes()) + pad(d.getSeconds());
  let file = safe + '-' + stamp + '.zip';
  for (let i = 1; fs.existsSync(path.join(backups, file)) && i < 100; i++) {
    file = safe + '-' + stamp + '-' + i + '.zip';
  }
  if (fs.existsSync(path.join(backups, file))) throw new Error('Backup slot busy — try again in a moment');
  const zip = new AdmZip();
  zip.addLocalFolder(src);
  zip.writeZip(path.join(backups, file));
  return { file, count: zip.getEntries().length };
});

ipcMain.handle('worlds:restore', (_, p = {}) => {
  const gameDir = String(p.gameDir || '');
  const safe = safeWorldName(p.name);
  if (!gameDir) throw new Error('No game directory set');
  const backups = path.join(gameDir, 'backups');
  if (!fs.existsSync(backups)) throw new Error('No backups found for "' + safe + '"');
  const zips = fs.readdirSync(backups).filter((f) => f.startsWith(safe + '-') && f.endsWith('.zip')).sort().reverse();
  if (!zips.length) throw new Error('No backups found for "' + safe + '"');
  const saves = path.join(gameDir, 'saves');
  const dest = path.join(saves, safe);
  worldsAssertInside(saves, dest);
  fs.mkdirSync(saves, { recursive: true });
  // 1) extract to temp dir + validate containment (zip-slip guard)
  const AdmZip = require('adm-zip');
  const zipPath = path.join(backups, zips[0]);
  const zip = new AdmZip(zipPath);
  const destReal = path.resolve(dest);
  for (const e of zip.getEntries()) {
    const target = path.resolve(dest, e.entryName);
    if (target !== destReal && !target.startsWith(destReal + path.sep)) {
      throw new Error('Backup contains unsafe paths — restore aborted');
    }
  }
  const tmp = path.join(saves, safe + '.restore-' + Date.now());
  fs.mkdirSync(tmp, { recursive: true });
  try {
    zip.extractAllTo(tmp, true);
    let root = tmp;
    if (!fs.existsSync(path.join(tmp, 'level.dat'))) {
      // tolerate zips wrapped in a single top-level folder
      let lifted = false;
      for (const k of fs.readdirSync(tmp, { withFileTypes: true })) {
        if (!k.isDirectory()) continue;
        if (!fs.existsSync(path.join(tmp, k.name, 'level.dat'))) continue;
        const inner = path.join(tmp, k.name);
        const liftedDir = path.join(saves, safe + '.lift-' + Date.now());
        fs.renameSync(inner, liftedDir);
        fs.rmSync(tmp, { recursive: true, force: true });
        fs.mkdirSync(tmp, { recursive: true });
        for (const f of fs.readdirSync(liftedDir)) fs.renameSync(path.join(liftedDir, f), path.join(tmp, f));
        fs.rmdirSync(liftedDir);
        lifted = true;
        break;
      }
      if (!lifted || !fs.existsSync(path.join(tmp, 'level.dat'))) {
        throw new Error('Backup is corrupt (no level.dat) — original untouched');
      }
      root = tmp;
    }
    void root;
    // 2) rotate: current -> .bak (keep max 1), tmp -> dest
    if (fs.existsSync(dest)) {
      for (const k of fs.readdirSync(saves)) {
        if (k !== safe && k.startsWith(safe + '.bak-')) {
          try { fs.rmSync(path.join(saves, k), { recursive: true, force: true }); } catch {}
        }
      }
      fs.renameSync(dest, path.join(saves, safe + '.bak-' + Date.now()));
    }
    fs.renameSync(tmp, dest);
  } catch (e) {
    try { fs.rmSync(tmp, { recursive: true, force: true }); } catch {}
    throw e;
  }
  return true;
});

ipcMain.handle('worlds:delete', (_, p = {}) => {
  const gameDir = String(p.gameDir || '');
  const safe = safeWorldName(p.name);
  if (!gameDir) throw new Error('No game directory set');
  const dest = path.join(gameDir, 'saves', safe);
  worldsAssertInside(path.join(gameDir, 'saves'), dest);
  if (!fs.existsSync(dest)) throw new Error('World not found: ' + safe);
  fs.rmSync(dest, { recursive: true, force: true });
  return true;
});

// ---- Live server status (Rust slp-probe + pure-Node SLP fallback) ----
const serverStatusCache = new Map();

function serverStatusParseHost(rawArg) {
  let raw = rawArg;
  if (raw && typeof raw === 'object' && raw.host !== undefined) raw = raw.host;
  let h = String(raw == null ? '' : raw).trim().slice(0, 255);
  h = h.replace(/^(https?:\/\/|mc:\/\/)/i, '').replace(/\/+$/, '').replace(/\s+/g, '');
  const slash = h.indexOf('/');
  if (slash >= 0) h = h.slice(0, slash);
  if (!h) return { error: 'empty host' };
  let hostname = h;
  let port = 25565;
  const m = h.match(/:(\d+)$/);
  if (m) {
    const p = parseInt(m[1], 10);
    if (!Number.isFinite(p) || p <= 0 || p > 65535) return { error: 'bad port' };
    port = p;
    hostname = h.slice(0, h.length - m[0].length);
  }
  hostname = hostname.replace(/\.+$/, '');
  if (!hostname || hostname.length > 253) return { error: 'bad host' };
  if (!/^[A-Za-z0-9._\-]+$/.test(hostname)) return { error: 'bad host' };
  return { hostname, port, key: hostname.toLowerCase() + ':' + port };
}

function serverStatusFlattenDescription(desc) {
  if (desc == null) return '';
  if (typeof desc === 'string') return desc;
  if (Array.isArray(desc)) return desc.map(serverStatusFlattenDescription).join('');
  if (typeof desc === 'object') {
    let s = typeof desc.text === 'string' ? desc.text : '';
    if (Array.isArray(desc.extra)) s += desc.extra.map(serverStatusFlattenDescription).join('');
    return s;
  }
  return String(desc);
}

function serverStatusCleanMotd(raw) {
  return String(raw == null ? '' : raw)
    .replace(/§[0-9a-fk-or]/gi, '')
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 120);
}

function serverStatusWriteVarInt(v) {
  v = Number(v) >>> 0;
  const out = [];
  do {
    let b = v & 0x7f;
    v >>>= 7;
    if (v) b |= 0x80;
    out.push(b);
  } while (v);
  return Buffer.from(out);
}

function serverStatusReadVarInt(buf, off) {
  let num = 0, shift = 0, size = 0, b = 0;
  do {
    if (off + size >= buf.length) return null;
    b = buf[off + size];
    size += 1;
    num |= (b & 0x7f) << shift;
    shift += 7;
    if (shift > 35) throw new Error('varint too big');
  } while (b & 0x80);
  return { value: num >>> 0, size };
}

function serverStatusPackString(str) {
  const body = Buffer.from(String(str), 'utf8');
  return Buffer.concat([serverStatusWriteVarInt(body.length), body]);
}

function serverStatusTryBinary(hostname, port) {
  return new Promise((resolve) => {
    try {
      const bin = require('path').join(__dirname, 'bin', process.platform === 'win32' ? 'slp-probe.exe' : 'slp-probe');
      const execFile = require('child_process').execFile;
      execFile(bin, [hostname, String(port), '6000'], { timeout: 8000, windowsHide: true, maxBuffer: 64 * 1024 }, (err, stdout) => {
        if (err) return resolve(null);
        try {
          const text = String(stdout || '');
          const lines = text.split(/\r?\n/);
          let line = null;
          for (const l of lines) {
            if (l.trim().charAt(0) === '{') { line = l; break; }
          }
          if (!line) line = text.trim();
          resolve(JSON.parse(line));
        } catch (e) { resolve(null); }
      });
    } catch (e) { resolve(null); }
  });
}

function serverStatusNodePing(hostname, port, timeoutMs) {
  return new Promise((resolve, reject) => {
    const net = require('net');
    const start = Date.now();
    let buf = Buffer.alloc(0);
    let done = false;
    let sock = null;
    let timer = null;
    const cleanup = () => { if (timer) { clearTimeout(timer); timer = null; } };
    const fail = (e) => {
      if (done) return;
      done = true;
      cleanup();
      try { if (sock) sock.destroy(); } catch (err) {}
      reject(e instanceof Error ? e : new Error(String(e || 'offline')));
    };
    const ok = (data) => {
      if (done) return;
      done = true;
      cleanup();
      try { if (sock) sock.destroy(); } catch (err) {}
      resolve(data);
    };
    try {
      sock = net.createConnection({ host: hostname, port });
    } catch (e) { fail(e); return; }
    timer = setTimeout(() => fail(new Error('timeout')), timeoutMs + 500);
    if (timer && typeof timer.unref === 'function') { try { timer.unref(); } catch (err) {} }
    sock.setTimeout(timeoutMs);
    sock.on('timeout', () => fail(new Error('timeout')));
    sock.on('error', fail);
    sock.on('connect', () => {
      try {
        const portBuf = Buffer.allocUnsafe(2);
        portBuf.writeUInt16BE(port, 0);
        const body = Buffer.concat([
          serverStatusWriteVarInt(0),
          serverStatusWriteVarInt(767),
          serverStatusPackString(hostname),
          portBuf,
          serverStatusWriteVarInt(1)
        ]);
        const handshake = Buffer.concat([serverStatusWriteVarInt(body.length), body]);
        const request = Buffer.concat([serverStatusWriteVarInt(1), serverStatusWriteVarInt(0)]);
        sock.write(Buffer.concat([handshake, request]));
      } catch (e) { fail(e); }
    });
    sock.on('data', (chunk) => {
      if (done) return;
      try {
        buf = Buffer.concat([buf, chunk]);
        if (buf.length > 2 * 1024 * 1024) { fail(new Error('response too large')); return; }
        const pLen = serverStatusReadVarInt(buf, 0);
        if (!pLen) return;
        if (buf.length < pLen.size + pLen.value) return;
        let off = pLen.size;
        const pid = serverStatusReadVarInt(buf, off);
        if (!pid) return;
        off += pid.size;
        const sLen = serverStatusReadVarInt(buf, off);
        if (!sLen) return;
        off += sLen.size;
        if (buf.length < off + sLen.value) return;
        const info = JSON.parse(buf.slice(off, off + sLen.value).toString('utf8'));
        ok({ info, pingMs: Date.now() - start });
      } catch (e) { fail(e); }
    });
  });
}

function serverStatusCachePut(key, data) {
  if (serverStatusCache.size > 300) serverStatusCache.delete(serverStatusCache.keys().next().value);
  serverStatusCache.set(key, { time: Date.now(), data });
}

ipcMain.handle('server:status', async (_evt, arg) => {
  const parsed = serverStatusParseHost(arg);
  if (parsed.error) {
    return { ok: false, players: 0, max: 0, motd: '', pingMs: 0, version: '', error: parsed.error, cached: false };
  }
  const hit = serverStatusCache.get(parsed.key);
  if (hit && Date.now() - hit.time < 60000) return Object.assign({}, hit.data, { cached: true });
  try {
    const bj = await serverStatusTryBinary(parsed.hostname, parsed.port);
    if (bj && bj.ok === true && Number.isFinite(Number(bj.players))) {
      const bRes = {
        ok: true,
        players: Math.max(0, parseInt(bj.players, 10) || 0),
        max: Math.max(0, parseInt(bj.max, 10) || 0),
        motd: serverStatusCleanMotd(bj.motd || ''),
        pingMs: Math.max(0, parseInt(bj.pingMs, 10) || 0),
        version: String(bj.version || '').slice(0, 40),
        error: '',
        cached: false
      };
      serverStatusCachePut(parsed.key, bRes);
      return bRes;
    }
  } catch (e) {}
  try {
    const r = await serverStatusNodePing(parsed.hostname, parsed.port, 6000);
    const info = r.info || {};
    const nRes = {
      ok: true,
      players: Math.max(0, parseInt((info.players && info.players.online) || 0, 10) || 0),
      max: Math.max(0, parseInt((info.players && info.players.max) || 0, 10) || 0),
      motd: serverStatusCleanMotd(serverStatusFlattenDescription(info.description)),
      pingMs: Math.max(0, r.pingMs || 0),
      version: String((info.version && info.version.name) || '').slice(0, 40),
      error: '',
      cached: false
    };
    serverStatusCachePut(parsed.key, nRes);
    return nRes;
  } catch (e) {
    return { ok: false, players: 0, max: 0, motd: '', pingMs: 0, version: '', error: String((e && e.message) || e || 'offline').slice(0, 120), cached: false };
  }
});

ipcMain.handle('update:check', async () => {
  const cmpSemver = (a, b) => {
    const pa = String(a || '0').split('.').map((x) => parseInt(x, 10) || 0);
    const pb = String(b || '0').split('.').map((x) => parseInt(x, 10) || 0);
    const n = Math.max(pa.length, pb.length);
    for (let i = 0; i < n; i++) {
      const x = pa[i] || 0;
      const y = pb[i] || 0;
      if (x > y) return 1;
      if (x < y) return -1;
    }
    return 0;
  };
  let current = '0.0.0';
  try { current = require('./package.json').version || current; } catch {}
  const fallbackUrl = 'https://github.com/radoslavgeme/cinder/releases/latest';
  try {
    const res = await fetch('https://api.github.com/repos/radoslavgeme/cinder/releases/latest', {
      headers: { 'User-Agent': 'Cinder', Accept: 'application/vnd.github+json' },
      signal: AbortSignal.timeout(10000)
    });
    if (!res.ok) return { current, latest: current, newer: false, url: fallbackUrl, error: 'GitHub HTTP ' + res.status };
    const j = await res.json();
    const latest = String((j && j.tag_name) || '').replace(/^v/i, '').trim() || current;
    const url = (j && j.html_url) || fallbackUrl;
    const newer = cmpSemver(latest, current) > 0;
    return { current, latest, newer, url, error: null };
  } catch (e) {
    return { current, latest: current, newer: false, url: fallbackUrl, error: String((e && e.message) || e).slice(0, 300) };
  }
});
ipcMain.handle('diag:run', async () => {
  try {
    const platform = process.platform;
    const totalGB = +(os.totalmem() / Math.pow(1024, 3)).toFixed(1);
    const freeGB = +(os.freemem() / Math.pow(1024, 3)).toFixed(1);
    let javas = [];
    try { javas = listAllJavas(); } catch { javas = []; }
    let gameDir = null;
    try { gameDir = sanitizeSettings(Object.assign({}, getDefaultSettings(), readJSON(SETTINGS_FILE, {}))).gameDir || getDefaultSettings().gameDir; } catch { gameDir = null; }
    let versionsCached = 0;
    try {
      const cached = readJSON(VERSIONS_CACHE_FILE, null);
      if (cached && cached.data && Array.isArray(cached.data.versions)) versionsCached = cached.data.versions.length;
      else if (gameDir) versionsCached = installedVersions(gameDir).size;
    } catch { versionsCached = 0; }
    let modsCount = 0;
    try { modsCount = gameDir ? readMods(gameDir).length : 0; } catch { modsCount = 0; }
    const probe = async (url, opts) => {
      try {
        const r = await fetch(url, Object.assign({ signal: AbortSignal.timeout(6000) }, opts || {}));
        return !!r.ok;
      } catch { return false; }
    };
    const results = await Promise.all([
      probe('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json', { headers: { 'User-Agent': 'Cinder' } }),
      probe('https://api.modrinth.com/v2/search?query=test&limit=1', { headers: { 'User-Agent': 'Cinder/1.0 (github:cinder-launcher)' } }),
      probe('https://minotar.net/helm/Steve/64.png', { headers: { 'User-Agent': 'Cinder' } })
    ]);
    return { platform, totalGB, freeGB, javas, versionsCached, modsCount, net: { mojang: results[0], modrinth: results[1], minotar: results[2] }, error: null };
  } catch (e) {
    try {
      return { platform: process.platform, totalGB: 0, freeGB: 0, javas: [], versionsCached: 0, modsCount: 0, net: { mojang: false, modrinth: false, minotar: false }, error: String((e && e.message) || e).slice(0, 300) };
    } catch { return { error: 'diag failed' }; }
  }
});

// ---- Cinder Client (in-game Fabric mod, installed per game directory) ----
const CLIENT_MOD_VERSION = '1.0.0';
const CLIENT_MC = '1.21.11';
const CLIENT_FAPI_PROJECT = 'fabric-api';
const MODRINTH_API = 'https://api.modrinth.com/v2';
const MODRINTH_UA = { 'User-Agent': 'Cinder/2.0 (github:cinder-launcher)' };

function clientModFile(gameDir) {
  try {
    const dir = path.join(String(gameDir || ''), 'mods');
    if (!fs.existsSync(dir)) return null;
    const hits = fs.readdirSync(dir).filter((f) => /^cinder-client-.*\.jar$/.test(f) && !f.endsWith('.disabled'));
    if (!hits.length) return null;
    hits.sort().reverse();
    return path.join(dir, hits[0]);
  } catch { return null; }
}

function writeSharedClientConfig(gameDir) {
  try {
    const s = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
    const file = path.join(String(gameDir), '.cinder-client.json');
    let prev = {};
    try { prev = JSON.parse(fs.readFileSync(file, 'utf8')); } catch {}
    writeJSON(file, {
      hudProfile: (prev && prev.hudProfile) || 'default',
      accent: s.accent || 'indigo',
      clientVersion: CLIENT_MOD_VERSION
    });
  } catch {}
}

async function modrinthPickFile(project, mcVersion, loader) {
  const q = `${MODRINTH_API}/project/${project}/version?game_versions=${encodeURIComponent(JSON.stringify([mcVersion]))}&loaders=${encodeURIComponent(JSON.stringify([loader]))}&limit=5`;
  const res = await fetch(q, { headers: MODRINTH_UA });
  if (!res.ok) throw new Error('Modrinth query failed: ' + res.status);
  const list = await res.json();
  for (const v of list || []) {
    const files = (v && v.files) || [];
    const primary = files.find((f) => f.primary) || files[0];
    if (primary && primary.url && primary.filename && primary.filename.endsWith('.jar')) {
      return { url: primary.url, filename: path.basename(primary.filename) };
    }
  }
  throw new Error('No ' + loader + ' file for ' + mcVersion);
}

async function downloadToFile(url, dest) {
  const res = await fetch(url, { headers: MODRINTH_UA });
  if (!res.ok) throw new Error('Download failed: ' + res.status);
  const buf = Buffer.from(await res.arrayBuffer());
  const tmp = dest + '.part';
  fs.writeFileSync(tmp, buf);
  fs.renameSync(tmp, dest);
  return buf.length;
}

async function installClientMod(gameDir, send) {
  const modsDir = path.join(gameDir, 'mods');
  fs.mkdirSync(modsDir, { recursive: true });
  // 1) client jar from the local Gradle build (never downloaded — our own code)
  const localJar = path.join(__dirname, 'client', 'build', 'libs', `cinder-client-${CLIENT_MOD_VERSION}.jar`);
  if (!fs.existsSync(localJar)) {
    throw new Error('Client jar not built yet — run the Gradle build in client/ first (see client/README.md)');
  }
  const dest = path.join(modsDir, `cinder-client-${CLIENT_MOD_VERSION}.jar`);
  fs.copyFileSync(localJar, dest);
  send(`> Cinder Client ${CLIENT_MOD_VERSION} installed from local build`);
  // 2) Fabric API for the client's MC version (from Modrinth, skipped if present)
  const haveApi = fs.readdirSync(modsDir).some((f) => /^fabric-api-.*\.jar$/.test(f) && !f.endsWith('.disabled'));
  if (!haveApi) {
    send('> Fetching Fabric API for ' + CLIENT_MC + ' from Modrinth…');
    const pick = await modrinthPickFile(CLIENT_FAPI_PROJECT, CLIENT_MC, 'fabric');
    await downloadToFile(pick.url, path.join(modsDir, pick.filename));
    send('> Installed ' + pick.filename);
  }
  // 3) shared launcher<->client config
  writeSharedClientConfig(gameDir);
  return { ok: true, file: dest };
}

ipcMain.handle('client:status', (_, gameDir) => {
  const file = clientModFile(gameDir);
  return { installed: !!file, file: file || null, modVersion: CLIENT_MOD_VERSION, mc: CLIENT_MC };
});

ipcMain.handle('client:install', async (_, p = {}) => {
  const gameDir = String((p && p.gameDir) || '');
  if (!gameDir) throw new Error('No game directory set');
  const win = BrowserWindow.getAllWindows()[0];
  const send = (m) => win && win.webContents.send('game:log', m);
  return installClientMod(gameDir, send);
});

ipcMain.handle('client:update', async (_, p = {}) => {
  const gameDir = String((p && p.gameDir) || '');
  if (!gameDir) throw new Error('No game directory set');
  const win = BrowserWindow.getAllWindows()[0];
  const send = (m) => win && win.webContents.send('game:log', m);
  send('> Updating Cinder Client…');
  return installClientMod(gameDir, send);
});

ipcMain.handle('client:profiles', (_, gameDir) => {
  try {
    const dir = path.join(String(gameDir || ''), 'config', 'hud-profiles');
    if (!fs.existsSync(dir)) return [];
    return fs.readdirSync(dir)
      .filter((f) => f.endsWith('.json'))
      .map((f) => f.slice(0, -5))
      .filter((n) => /^[A-Za-z0-9 _\-]{1,32}$/.test(n))
      .sort();
  } catch { return []; }
});

ipcMain.handle('client:profile-set', (_, p = {}) => {
  const gameDir = String((p && p.gameDir) || '');
  const name = String((p && p.name) || '').trim().slice(0, 32);
  if (!gameDir) throw new Error('No game directory set');
  if (!/^[A-Za-z0-9 _\-]{1,32}$/.test(name)) throw new Error('Invalid profile name');
  const file = path.join(gameDir, '.cinder-client.json');
  let prev = {};
  try { prev = JSON.parse(fs.readFileSync(file, 'utf8')); } catch {}
  prev.hudProfile = name;
  prev.clientVersion = CLIENT_MOD_VERSION;
  writeJSON(file, prev);
  return true;
});

// ---- Launch ----
let launching = false;
let currentChild = null;
function splitJvmArgs(str) {
  // split on spaces but respect double quotes
  const out = [];
  let cur = '', inQ = false;
  for (const ch of String(str || '')) {
    if (ch === '"') { inQ = !inQ; continue; }
    if (ch === ' ' && !inQ) { if (cur) { out.push(cur); cur = ''; } continue; }
    cur += ch;
  }
  if (cur) out.push(cur);
  return out.slice(0, 20);
}
function quickPlayFor(serverHost, mcVersion) {
  const host = String(serverHost || '').trim();
  if (!host) return undefined;
  const m = String(mcVersion || '').match(/^1\.(\d+)/);
  const minor = m ? parseInt(m[1], 10) : 99;
  // 1.20+ supports multiplayer quickPlay, older use legacy
  if (minor >= 20 || !String(mcVersion).startsWith('1.')) return { type: 'multiplayer', identifier: host };
  return { type: 'legacy', identifier: host };
}

ipcMain.handle('game:launch', async (event, { account, version, settings }) => {
  if (launching) throw new Error('Already launching');
  launching = true;
  const win = BrowserWindow.getAllWindows()[0];
  const send = (msg) => win && win.webContents.send('game:log', msg);

  try {
    const { Client, Authenticator } = require('minecraft-launcher-core');
    const launcher = new Client();
    const gameDir = settings.gameDir;

    fs.mkdirSync(gameDir, { recursive: true });
    fs.mkdirSync(path.join(gameDir, 'mods'), { recursive: true });

    launcher.on('debug', (e) => send('[debug] ' + e));
    launcher.on('data', (e) => send(String(e).trim()));
    launcher.on('progress', (e) => {
      if (e.type === 'version' || e.type === 'assets' || e.type === 'natives' || e.type === 'libraries' || e.type === 'forge') {
        win && win.webContents.send('game:progress', e);
      }
    });
    launcher.on('close', (code) => {
      win && win.webContents.send('game:closed', code);
      send(`> Game closed (code ${code}).`);
      currentChild = null;
      try {
        // If we hid the launcher while playing, bring it back now.
        if (win && !win.isDestroyed() && !win.isVisible()) win.show();
      } catch {}
      try { discord.init({ settings }).catch(() => {}); } catch {}
      try { discord.setLauncher({ account: account.name, version }); } catch {}
    });

    let authorization;
    let modeLabel = 'offline';
    if (account.type === 'microsoft') {
      send(`> Refreshing Microsoft token for ${account.name}...`);
      authorization = await getMicrosoftAuth(account);
      modeLabel = 'online (premium)';
    } else {
      authorization = Authenticator.getAuth(account.name);
      modeLabel = 'offline (cracked servers only)';
    }

    const autoJava = settings.autoJava !== false;
    let javaPath = getJavaForVersion(version, settings.javaPath, autoJava);
    if (javaPath && javaPath !== 'java' && !fs.existsSync(javaPath)) {
      throw new Error('Java not found at: ' + javaPath + ' — fix it in Settings.');
    }
    if (settings.javaPath !== javaPath) {
      send(`> Auto-selected Java ${javaPath} (needs Java ${requiredJavaForVersion(version)}+ for ${version})`);
    }
    if (javaPath === 'java') javaPath = undefined;

    let versionId = version;
    let loaderLabel = 'vanilla';
    const wantLoader = settings.loader && settings.loader !== 'vanilla' ? settings.loader : null;
    if (wantLoader) {
      const stored = readJSON(SETTINGS_FILE, {});
      const profiles = { ...(stored.loaderProfiles || {}), ...(settings.loaderProfiles || {}) };
      let pid = profiles[`${wantLoader}:${version}`];
      const vf = (id) => path.join(gameDir, 'versions', id, id + '.json');
      if (!pid || !fs.existsSync(vf(pid))) {
        send(`> Loader ${wantLoader} for ${version} isn't installed — installing it automatically...`);
        try {
          const r = await loaderLib.installLoader({ loader: wantLoader, mc: version, gameDir, log: send });
          pid = r.profileId;
          const cur = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
          cur.loaderProfiles = { ...(cur.loaderProfiles || {}), [`${wantLoader}:${version}`]: pid };
          writeJSON(SETTINGS_FILE, cur);
        } catch (e) {
          throw new Error(`Automatic install of ${wantLoader} for ${version} failed: ${e.message}`);
        }
      }
      versionId = pid;
      loaderLabel = wantLoader;
    }

    const customArgs = splitJvmArgs(settings.jvmArgs);
    // CINDER-INSERT jvmPreset (after customArgs)
    let _jvmPreset = (settings && ['balanced', 'performance', 'latency'].includes(settings.jvmPreset)) ? settings.jvmPreset : 'balanced';
    if (_jvmPreset === 'latency') {
      const _major = (() => {
        try {
          const hit = listAllJavas().find((j) => j.path === javaPath);
          return hit ? hit.major : 0;
        } catch { return 0; }
      })();
      if (_major > 0 && _major < 21) {
        send(`> JVM preset [latency] needs Java 21+ but selected Java is ${_major} — falling back to balanced`);
        _jvmPreset = 'balanced';
      }
    }
    const _presetFlags = _jvmPreset === 'performance'
      ? ['-XX:+UseG1GC', '-XX:MaxGCPauseMillis=200', '-XX:+ParallelRefProcEnabled']
      : _jvmPreset === 'latency' ? ['-XX:+UseZGC'] : [];
    if (_presetFlags.length) {
      const _hay = customArgs.join(' ').toLowerCase();
      for (const _f of _presetFlags) {
        if (!_hay.includes(String(_f).toLowerCase())) customArgs.push(_f);
      }
      send('> JVM preset [' + _jvmPreset + '] applied: ' + _presetFlags.join(' '));
    }
    const quickPlay = quickPlayFor(settings.serverHost, version);

    const opts = {
      clientPackage: null,
      authorization,
      root: gameDir,
      version: { number: versionId, type: 'release' },
      memory: { max: settings.maxRam, min: settings.minRam },
      window: settings.fullscreen
        ? { fullscreen: true }
        : { width: Number(settings.width) || 854, height: Number(settings.height) || 480 },
      javaPath,
      customArgs: customArgs.length ? customArgs : undefined,
      quickPlay,
      timeout: (Number(settings.launchTimeoutSec) || 30) * 1000
    };

    send(`> Launching Minecraft ${version} as ${account.name} [${modeLabel}, ${loaderLabel}]...`);
    send(`> Game dir: ${gameDir}`);
    if (quickPlay) send(`> Auto-joining server: ${quickPlay.identifier}`);
    if (customArgs.length) send(`> Extra JVM args: ${customArgs.join(' ')}`);
    if (account.type !== 'microsoft') send('> Note: offline mode works on cracked/offline servers only.');
    const child = await launcher.launch(opts);
    currentChild = child || null;
    try {
      const act = settings.launchAction || 'minimize';
      if (win && !win.isDestroyed()) {
        if (act === 'minimize') win.minimize();
        else if (act === 'hide') win.hide();
      }
    } catch {}
    send('> Game process started. You can close this log and play.');
    try { await discord.init({ settings }); } catch {}
    try { discord.setGame({ account: account.name, version, serverHost: settings.serverHost, loader: loaderLabel }); } catch {}

    // persist last played + recent servers
    try {
      const cur = sanitizeSettings({ ...getDefaultSettings(), ...readJSON(SETTINGS_FILE, {}) });
      cur.lastAccountId = account.id;
      cur.lastVersion = version;
      if (settings.serverHost && String(settings.serverHost).trim()) {
        const h = String(settings.serverHost).trim();
        cur.recentServers = [h, ...(cur.recentServers || []).filter(x => x !== h)].slice(0, 8);
        cur.serverHost = h;
      }
      writeJSON(SETTINGS_FILE, cur);
    } catch {}

    win && win.webContents.send('game:started');
    return { ok: true };
  } catch (err) {
    send('ERROR: ' + (err.message || String(err)));
    send('TIP: Check Java path in Settings. For online mode you must own Minecraft Java.');
    throw err;
  } finally {
    launching = false;
  }
});

ipcMain.handle('game:kill', async () => {
  try {
    if (currentChild && !currentChild.killed) {
      currentChild.kill();
      return true;
    }
  } catch {}
  return false;
});
ipcMain.handle('game:is-running', () => !!(currentChild && !currentChild.killed));
