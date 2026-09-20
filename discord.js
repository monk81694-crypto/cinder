// discord.js — Discord Rich Presence for Cinder (main process only).
// Shows "Playing Cinder" (your application name) in the launcher and
// "Playing Minecraft 1.21 as Steve → hypixel.net" while in game.
//
// ZERO SETUP FOR PLAYERS: the launcher ships one shared Application ID, so
// anyone with the Discord desktop app open gets the status automatically —
// same experience as Valorant/Roblox (those are Discord-partnered; we get the
// same result via a shared app instead of per-user setup).
//
// OWNER ONE-TIME SETUP (radoslavgeme, ~1 min — players never touch this):
//   1. https://discord.com/developers/applications → New Application "Cinder"
//   2. Rich Presence → Art Assets → upload assets/logo-512.png as "logo"
//   3. Copy the Application ID → paste it as DEFAULT_CLIENT_ID below.
//      (NOT a bot: no token, no invites, no server. Just an ID + a logo.)
//
// TROUBLESHOOT ("it says Minecraft, not Cinder"):
//   Discord auto-detects javaw.exe as "Minecraft". Our connected presence
//   claims that slot instead — so this always means "not connected yet":
//   open the Discord desktop app → Settings → Discord → Test → Connected ✓.
//   Still losing? Discord → Settings → Registered Games → remove the
//   Minecraft/javaw entry → restart Discord → Test again.

// Shared Cinder application (owner: radoslavgeme). Players get presence
// automatically — no setup on their end.
const DEFAULT_CLIENT_ID = '1549531998329241742';

let RPC = null;
try {
  RPC = require('discord-rpc');
} catch {
  RPC = null; // discord-rpc not installed → presence disabled, no crash
}

let rpc = null;
let connected = false;
let connecting = false;
let enabled = true;
let clientId = '';
let activeClientId = '';
let logFn = () => {};
let lastActivity = null;
let startTimestamp = null;
let retryTimer = null;
// Download button shown on the profile activity. Placeholder until release:
// point it at your real download page in Settings → Discord anytime.
let downloadUrl = 'https://github.com/radoslavgeme/cinder';

function getClientId(settings) {
  const manual = String((settings && settings.discordClientId) || '').trim();
  if (manual) return manual;
  const env = String(process.env.CINDER_DISCORD_CLIENT_ID || '').trim();
  if (env) return env;
  // Shared built-in ID: zero setup for players. The placeholder below fails
  // the digit check on purpose, so it counts as "unset" until the owner
  // pastes the real Application ID.
  return /^\d{8,}$/.test(DEFAULT_CLIENT_ID) ? DEFAULT_CLIENT_ID : '';
}

function buttons() {
  if (!/^https?:\/\/.+/i.test(downloadUrl || '')) return undefined;
  return [{ label: 'Download Cinder', url: downloadUrl }];
}

function withButtons(activity) {
  const b = buttons();
  return b ? { ...activity, buttons: b } : activity;
}

async function init({ settings, log } = {}) {
  if (log) logFn = log;
  enabled = !settings || settings.discordPresence !== false;
  clientId = getClientId(settings);
  const u = String((settings && settings.discordDownloadUrl) || process.env.CINDER_DOWNLOAD_URL || downloadUrl).trim().slice(0, 200);
  if (/^https?:\/\/.+/i.test(u)) downloadUrl = u;
  scheduleRetry();
  if (!RPC || !enabled || !clientId) return false;
  if (connected || connecting) return connected;
  return connect();
}

// Background lifeline: user often starts Discord AFTER the launcher, or
// restarts Discord mid-session. Every 30s, take one shot at connecting.
// Unref'd + cleared on shutdown, so it never holds the app (or tests) open.
function scheduleRetry() {
  if (retryTimer) return;
  try {
    retryTimer = setInterval(() => {
      if (enabled && clientId && !connected && !connecting && RPC) {
        connect().catch(() => {});
      }
    }, 30000);
    if (retryTimer.unref) retryTimer.unref();
  } catch {}
}

async function connect() {
  connecting = true;
  try {
    // Always a fresh transport: a dead socket (Discord was closed, ID was
    // changed) poisons reuse and fails forever. Rebuilding is cheap.
    try { if (rpc) await rpc.destroy(); } catch {}
    rpc = null;
    RPC.register(clientId);
    rpc = new RPC.Client({ transport: 'ipc' });
    activeClientId = clientId;
    rpc.on('ready', () => {
      connected = true;
      connecting = false;
      if (lastActivity) safeSet(lastActivity);
      else setLauncher({ account: null, version: null });
    });
    rpc.transport && rpc.transport.on && rpc.transport.on('close', () => {
      connected = false;
      rpc = null;
    });
    await rpc.login({ clientId });
    connected = true;
    connecting = false;
    return true;
  } catch (e) {
    connecting = false;
    connected = false;
    rpc = null;
    // Discord closed / not installed — stay silent except a debug line.
    try { logFn('[debug] Discord presence unavailable: ' + (e.message || e)); } catch {}
    return false;
  }
}

function safeSet(activity) {
  lastActivity = activity;
  if (!rpc || !connected || !enabled || !clientId) return false;
  try {
    rpc.setActivity(activity);
    return true;
  } catch {
    return false;
  }
}

function baseAssets(inGame) {
  return {
    largeImageKey: 'logo',
    largeImageText: 'Cinder Launcher',
    smallImageKey: inGame ? 'cube' : undefined,
    smallImageText: inGame ? 'In game' : undefined,
  };
}

function setLauncher({ account, version, screen } = {}) {
  startTimestamp = startTimestamp || new Date();
  const details = 'In the launcher';
  const parts = [];
  if (screen) parts.push(labelForScreen(screen));
  if (version) parts.push(version);
  if (account) parts.push('as ' + account);
  return safeSet(withButtons({
    ...baseAssets(false),
    details,
    state: parts.join(' · ') || 'Picking a version',
    startTimestamp,
    instance: false,
  }));
}

function labelForScreen(screen) {
  const map = {
    play: 'Ready to launch',
    versions: 'Browsing versions',
    accounts: 'Managing accounts',
    mods: 'Browsing mods',
    settings: 'Tuning settings',
    logs: 'Reading logs',
  };
  return map[screen] || null;
}

function setScreen(screen, ctx = {}) {
  return setLauncher({ ...ctx, screen });
}

function setGame({ account, version, serverHost, loader } = {}) {
  startTimestamp = new Date();
  const cleanServer = String(serverHost || '').trim();
  return safeSet(withButtons({
    ...baseAssets(true),
    details: `Playing Minecraft ${version || ''}`.trim(),
    state: cleanServer
      ? `${account || 'Someone'} → ${cleanServer}`
      : `${account || 'Someone'}${loader && loader !== 'vanilla' ? ' · ' + loader : ''} · singleplayer`,
    startTimestamp,
    instance: false,
  }));
}

async function shutdown() {
  try {
    if (retryTimer) { clearInterval(retryTimer); retryTimer = null; }
    if (rpc) {
      if (connected) { try { await rpc.clearActivity(); } catch {} }
      try { await rpc.destroy(); } catch {}
    }
  } catch {}
  rpc = null;
  connected = false;
  connecting = false;
}

function status() {
  return { installed: !!RPC, enabled, connected, clientIdSet: !!clientId };
}

module.exports = { init, setLauncher, setScreen, setGame, shutdown, status, getClientId };
