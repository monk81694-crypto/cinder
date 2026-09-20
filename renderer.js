/* Cinder v1.0 renderer — state, toasts, modal, dashboard, servers, mods, logs, presence, name forge */
const $ = (id) => document.getElementById(id);
const api = (window.cinder || window.tlauncher);
let accounts = [], versions = [], settings = null, javas = [], sysInfo = null;
let selectedVersion = null, activeAccountId = null;
let modsCache = [], modChecks = [];
let gameRunning = false;
let logLines = []; // { text, cls, time }
let versionsMeta = { fromCache: false, stale: false };

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const avatarUrl = (name, size = 64) => `https://minotar.net/helm/${encodeURIComponent(String(name || 'steve'))}/${size}.png`;

/* ---------- Toasts ---------- */
function toast(msg, type = 'info', ms = 3500) {
  const box = $('toasts');
  const el = document.createElement('div');
  el.className = `toast toast-${type === 'error' ? 'err' : type === 'ok' ? 'ok' : 'info'}`;
  const ico = type === 'ok' ? '✓' : type === 'error' ? '✕' : 'ℹ';
  el.innerHTML = `<span class="toast-ico">${ico}</span><span>${esc(msg)}</span>`;
  box.appendChild(el);
  setTimeout(() => { el.classList.add('out'); setTimeout(() => el.remove(), 300); }, ms);
}

/* ---------- Modal (replaces alert/confirm) ---------- */
let modalResolve = null;
function confirmModal({ title = 'Confirm', text = '', extraHTML = '', okLabel = 'Confirm', danger = false }) {
  return new Promise((resolve) => {
    modalResolve = resolve;
    $('modal-title').textContent = title;
    $('modal-text').textContent = text;
    $('modal-extra').innerHTML = extraHTML;
    const ok = $('modal-ok');
    ok.textContent = okLabel;
    ok.style.filter = danger ? 'hue-rotate(140deg) saturate(2)' : '';
    $('modal-backdrop').classList.remove('hidden');
  });
}
function closeModal(val) {
  $('modal-backdrop').classList.add('hidden');
  $('modal-extra').innerHTML = '';
  if (modalResolve) { modalResolve(val); modalResolve = null; }
}
$('modal-cancel').onclick = () => closeModal(false);
$('modal-ok').onclick = () => closeModal(true);
$('modal-backdrop').addEventListener('click', (e) => { if (e.target.id === 'modal-backdrop') closeModal(false); });

/* ---------- Logs ---------- */
function classifyLog(msg) {
  const m = String(msg || '');
  if (/^ERROR|^.*(failed|error|exception|unable|couldn)/i.test(m)) return 'line-error';
  if (/^TIP|^WARN|^.*warn/i.test(m)) return 'line-warn';
  if (/ready as|Installed|Logged in|Game process started|✓|success/i.test(m)) return 'line-ok';
  if (/^\[debug\]/.test(m)) return 'line-dim';
  return '';
}
function log(msg) {
  const line = { text: String(msg ?? ''), cls: classifyLog(msg), time: new Date() };
  logLines.push(line);
  if (logLines.length > 2000) logLines = logLines.slice(-2000);
  appendLogLine(line);
  // ping nav dot if user isn't looking at logs
  if (!$('tab-logs').classList.contains('active')) $('nav-log-dot').classList.remove('hidden');
}
function appendLogLine(line) {
  if (hideDebugLine(line.text)) return;
  const filter = ($('logs-search').value || '').toLowerCase();
  if (filter && !line.text.toLowerCase().includes(filter)) return;
  const el = $('logs');
  const div = document.createElement('div');
  div.className = 'line ' + line.cls;
  div.textContent = line.text;
  el.appendChild(div);
  while (el.children.length > 2000) el.firstChild.remove();
  if ($('logs-autoscroll').checked) el.scrollTop = el.scrollHeight;
}
function rerenderLogs() {
  const el = $('logs');
  el.innerHTML = '';
  const filter = ($('logs-search').value || '').toLowerCase();
  logLines.filter(l => !hideDebugLine(l.text) && (!filter || l.text.toLowerCase().includes(filter))).slice(-800).forEach(l => {
    const div = document.createElement('div');
    div.className = 'line ' + l.cls;
    div.textContent = l.text;
    el.appendChild(div);
  });
  el.scrollTop = el.scrollHeight;
}

/* ---------- Tabs + shortcuts + Discord presence ---------- */
function pushPresence(screen) {
  try {
    const acc = activeAccount();
    const r = api.presenceUpdate({ screen, account: acc ? acc.name : null, version: selectedVersion });
    if (r && typeof r.catch === 'function') r.catch(() => {});
  } catch {}
}
function gotoTab(name) {
  document.querySelectorAll('.nav').forEach(b => b.classList.toggle('active', b.dataset.tab === name));
  document.querySelectorAll('.tab').forEach(t => t.classList.toggle('active', t.id === 'tab-' + name));
  if (name === 'logs') $('nav-log-dot').classList.add('hidden');
  pushPresence(name);
}
document.querySelectorAll('.nav').forEach(btn => { btn.onclick = () => gotoTab(btn.dataset.tab); });
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); doLaunch(); return; }
  if (!(e.ctrlKey || e.metaKey)) return;
  const map = { 1: 'play', 2: 'versions', 3: 'accounts', 4: 'mods', 5: 'settings', 6: 'logs', 7: 'skins', 8: 'worlds', 9: 'client' };
  if (map[e.key]) { e.preventDefault(); gotoTab(map[e.key]); }
});

/* ---------- Accounts ---------- */
async function loadAccounts() {
  accounts = await api.accountsList();
  if (!activeAccountId) activeAccountId = (settings && settings.lastAccountId) || (accounts[0] && accounts[0].id) || null;
  if (activeAccountId && !accounts.find(a => a.id === activeAccountId)) activeAccountId = (accounts[0] && accounts[0].id) || null;
  renderAccounts();
}
function activeAccount() { return accounts.find(a => a.id === activeAccountId) || accounts[0] || null; }
function renderAccounts() {
  const list = $('account-list');
  list.innerHTML = '';
  const sel = $('play-account');
  sel.innerHTML = '';
  accounts.forEach(a => {
    const isMS = a.type === 'microsoft';
    // dropdown
    const opt = document.createElement('option');
    opt.value = a.id; opt.textContent = `${a.name} — ${isMS ? 'online' : 'offline'}`;
    if (a.id === activeAccountId) opt.selected = true;
    sel.appendChild(opt);
    // card row
    const div = document.createElement('div');
    div.className = 'item' + (a.id === activeAccountId ? ' active-row' : '');
    div.innerHTML =
      `<div class="item-main"><img class="avatar" src="${avatarUrl(a.name, 64)}" alt="" loading="lazy" onerror="this.style.display='none'">` +
      `<div><b>${esc(a.name)}</b> ` +
      `<span class="badge ${a.id === activeAccountId ? 'badge-active' : ''}">${a.id === activeAccountId ? 'active' : (isMS ? 'online premium' : 'offline')}</span>` +
      `<div class="item-sub">${isMS ? '🔒 Microsoft · auto-refresh' : '🔓 Offline · cracked servers only'}</div></div></div>`;
    const acts = document.createElement('div');
    acts.className = 'item-actions';
    if (a.id !== activeAccountId) {
      const use = document.createElement('button');
      use.textContent = 'Use';
      use.onclick = async (ev) => { ev.stopPropagation(); await setActiveAccount(a.id); };
      acts.appendChild(use);
    }
    const del = document.createElement('button');
    del.textContent = 'Delete';
    del.className = 'btn-danger';
    del.onclick = async (ev) => {
      ev.stopPropagation();
      if (!await confirmModal({ title: 'Delete account?', text: `Remove "${a.name}" from this launcher? (Files stay on disk.)`, okLabel: 'Delete', danger: true })) return;
      accounts = await api.accountsRemove(a.id);
      if (activeAccountId === a.id) activeAccountId = (accounts[0] && accounts[0].id) || null;
      renderAccounts();
      toast('Account removed', 'ok');
    };
    acts.appendChild(del);
    div.appendChild(acts);
    div.style.cursor = 'pointer';
    div.onclick = () => setActiveAccount(a.id);
    list.appendChild(div);
  });
  if (!accounts.length) sel.innerHTML = '<option value="">-- add account first --</option>';
  syncAccountUI();
}
async function setActiveAccount(id) {
  activeAccountId = id;
  try { settings = await api.settingsSave({ lastAccountId: id }); } catch {}
  renderAccounts();
  pushPresence(document.querySelector('.nav.active')?.dataset?.tab || 'play');
}
function syncAccountUI() {
  const acc = activeAccount();
  const sel = $('play-account');
  if (acc) sel.value = acc.id;
  $('side-account').textContent = acc ? acc.name : 'No account';
  const av = $('side-avatar'), dot = $('side-dot');
  if (acc) { av.src = avatarUrl(acc.name, 64); av.style.display = ''; dot.style.display = 'none'; }
  else { av.style.display = 'none'; dot.style.display = ''; }
  const pa = $('play-avatar');
  if (acc) { pa.src = avatarUrl(acc.name, 128); pa.style.display = ''; } else pa.style.display = 'none';
  $('play-greeting').textContent = acc ? `Welcome back, ${acc.name}` : 'Welcome to Cinder';
  const isMS = acc && acc.type === 'microsoft';
  $('play-mode-badge').textContent = !acc ? 'no account' : isMS ? '🔒 online premium' : '🔓 offline';
  $('play-detail').textContent = !acc
    ? 'Add an account to get started'
    : isMS ? 'Premium — Hypixel & all online servers work' : 'Offline — cracked / offline-mode servers only';
  $('stat-accounts').textContent = String(accounts.length);
}

/* ---------- Versions ---------- */
async function loadVersions(force = false) {
  $('version-list').innerHTML = '<div class="skeleton">Loading versions…</div><div class="skeleton">Loading versions…</div><div class="skeleton">Loading versions…</div>';
  try {
    const data = await api.versionsList({ gameDir: settings.gameDir, force });
    versions = data.versions;
    versionsMeta = { fromCache: !!data.fromCache, stale: !!data.stale };
    $('versions-cache-note').textContent = data.stale ? '· offline cache' : data.fromCache ? '· cached' : '· live from Mojang';
    const rel = data.latest && data.latest.release;
    if (!selectedVersion) selectedVersion = (settings && settings.lastVersion) || rel || (versions.find(v => v.type === 'release') || versions[0] || {}).id || null;
    renderVersions();
    renderPlayVersions();
    $('stat-versions').textContent = String(versions.filter(v => v.installed).length || 0);
  } catch (e) {
    $('version-list').innerHTML = `<div class="muted">Failed: ${esc(e.message)} <button class="chip" id="ver-retry" type="button">Retry</button></div>`;
    const r = $('ver-retry');
    if (r) r.onclick = () => loadVersions(true);
  }
}
function renderVersions() {
  const q = ($('version-search').value || '').toLowerCase();
  const f = $('version-filter').value;
  const onlyInstalled = $('version-installed-only').checked;
  const list = $('version-list');
  list.innerHTML = '';
  const rows = versions
    .filter(v => (f === 'all' || v.type === f) && v.id.toLowerCase().includes(q) && (!onlyInstalled || v.installed))
    .slice(0, 60);
  if (!rows.length) { list.innerHTML = '<div class="muted" style="padding:18px">No versions match. Try clearing the search.</div>'; return; }
  rows.forEach(v => {
    const div = document.createElement('div');
    div.className = 'item' + (selectedVersion === v.id ? ' active-row' : '');
    const date = v.time ? new Date(v.time).toLocaleDateString() : '';
    div.innerHTML =
      `<div class="item-main">${v.installed ? '<span class="installed-dot" title="Installed"></span>' : ''}` +
      `<div><b>${esc(v.id)}</b> <span class="badge">${esc(v.type)}</span>` +
      `${v.installed ? '<span class="badge badge-installed">installed</span>' : ''}` +
      `${date ? `<div class="item-sub">${esc(date)}</div>` : ''}</div></div>`;
    const acts = document.createElement('div');
    acts.className = 'item-actions';
    const btn = document.createElement('button');
    btn.textContent = selectedVersion === v.id ? '✓ Selected' : 'Select';
    btn.disabled = selectedVersion === v.id;
    btn.onclick = () => { selectedVersion = v.id; renderVersions(); renderPlayVersions(); persistLastVersion(); };
    acts.appendChild(btn);
    const play = document.createElement('button');
    play.textContent = 'Play';
    play.title = 'Select + go to Play';
    play.onclick = () => { selectedVersion = v.id; renderVersions(); renderPlayVersions(); persistLastVersion(); gotoTab('play'); };
    acts.appendChild(play);
    if (v.installed) {
      const del = document.createElement('button');
      del.textContent = 'Uninstall';
      del.className = 'btn-danger';
      del.title = 'Delete downloaded files for this version';
      del.onclick = async () => {
        if (!await confirmModal({ title: 'Uninstall ' + v.id + '?', text: 'Deletes the downloaded version files. Saves and mods stay. You can re-download anytime.', okLabel: 'Uninstall', danger: true })) return;
        try {
          await api.versionsUninstall({ gameDir: settings.gameDir, id: v.id });
          toast(`Uninstalled ${v.id}`, 'ok');
          log(`> Uninstalled ${v.id}`);
          await loadVersions();
        } catch (e) { toast(e.message, 'error'); }
      };
      acts.appendChild(del);
    }
    div.appendChild(acts);
    list.appendChild(div);
  });
}
async function persistLastVersion() {
  try { if (selectedVersion) settings = await api.settingsSave({ lastVersion: selectedVersion }); } catch {}
}
function renderPlayVersions() {
  const sel = $('play-version');
  const prev = selectedVersion;
  sel.innerHTML = '';
  versions.slice(0, 60).forEach(v => {
    const opt = document.createElement('option');
    opt.value = v.id; opt.textContent = v.id + (v.installed ? ' ✓' : '') + ' (' + v.type + ')';
    if (v.id === prev) opt.selected = true;
    sel.appendChild(opt);
  });
  if (!selectedVersion && versions.length) {
    selectedVersion = versions.find(v => v.type === 'release')?.id || versions[0].id;
    sel.value = selectedVersion;
  } else if (prev) sel.value = prev;
  const cur = versions.find(v => v.id === selectedVersion);
  $('ver-installed-hint').textContent = cur ? (cur.installed ? '· ✓ installed' : '· will download') : '';
  const L = settings && settings.loader && settings.loader !== 'vanilla' ? ' + ' + settings.loader : '';
  $('side-version').textContent = (selectedVersion || 'No version') + L;
  $('stat-loader').textContent = (settings && settings.loader) || 'vanilla';
  updateLoaderStatus();
  updateJavaHint();
}

/* ---------- Appearance ---------- */
function applyAppearance() {
  try {
    document.body.dataset.accent = (settings && settings.accent) || 'mint';
    document.body.classList.toggle('reduce-motion', !!(settings && settings.reduceMotion));
  } catch {}
}
function hideDebugLine(text) {
  return !!settings && settings.showDebugLogs === false && /^\[debug\]/.test(String(text || ''));
}

/* ---------- Settings ---------- */
function parseRamG(v, fb) {
  const m = String(v || '').match(/^([0-9]+(?:\.[0-9]+)?)\s*([GM])/i);
  if (!m) return fb;
  let n = parseFloat(m[1]);
  if (m[2].toUpperCase() === 'M') n = n / 1024;
  return Math.max(1, Math.min(16, Math.round(n)));
}
async function loadSettings() {
  settings = await api.settingsGet();
  $('set-gamedir').value = settings.gameDir;
  $('set-autojava').checked = settings.autoJava !== false;
  const minG = parseRamG(settings.minRam, 2), maxG = parseRamG(settings.maxRam, 4);
  $('set-minram').value = minG; $('set-maxram').value = maxG;
  $('minram-val').textContent = minG + 'G'; $('maxram-val').textContent = maxG + 'G';
  $('set-width').value = settings.width;
  $('set-height').value = settings.height;
  $('set-fullscreen').checked = !!settings.fullscreen;
  $('set-jvmargs').value = settings.jvmArgs || '';
  if ($('set-launchaction')) $('set-launchaction').value = settings.launchAction || 'minimize';
  if ($('set-timeout')) $('set-timeout').value = settings.launchTimeoutSec || 30;
  if ($('set-accent')) $('set-accent').value = settings.accent || 'mint';
  if ($('set-motion')) $('set-motion').checked = !!settings.reduceMotion;
  if ($('set-debuglogs')) $('set-debuglogs').checked = settings.showDebugLogs !== false;
  if ($('logs-autoscroll')) $('logs-autoscroll').checked = settings.logsFollow !== false;
  applyAppearance();
  applySnow();
  syncNewSettingsUI();
  if ($('set-discord')) $('set-discord').checked = settings.discordPresence !== false;
  if ($('set-discord-id')) $('set-discord-id').value = settings.discordClientId || '';
  if ($('set-discord-dl')) $('set-discord-dl').value = settings.discordDownloadUrl || '';
  if ($('loader-select')) $('loader-select').value = settings.loader || 'vanilla';
  if ($('mods-loader') && (settings.loader === 'fabric' || settings.loader === 'quilt')) $('mods-loader').value = settings.loader;
  if (settings.serverHost) $('play-server').value = settings.serverHost;
  renderRecentServers();
  await refreshJavaList();
  refreshDiscordStatus();
  try {
    sysInfo = await api.systemInfo();
    $('ram-hint').textContent = `· system ${sysInfo.totalGB}G (${sysInfo.freeGB}G free)`;
  } catch {}
}

/* ---------- Discord presence status ---------- */
async function refreshDiscordStatus() {
  const el = $('discord-status');
  if (!el) return;
  try {
    const st = await api.presenceStatus();
    if (!st.installed) el.textContent = 'discord-rpc not installed — run npm install.';
    else if (settings && settings.discordPresence === false) el.textContent = 'Disabled — tick the box and Save to enable.';
    else if (!st.clientIdSet) el.textContent = 'Owner step pending: shared Application ID not baked in yet.';
    else if (st.connected) el.textContent = 'Connected ✓ — your Discord profile shows Cinder.';
    else el.textContent = 'Just open the Discord desktop app — no setup needed. Then hit Test.';
  } catch { el.textContent = ''; }
}
async function refreshJavaList() {
  try {
    const { javas: list, required } = await api.javaList(selectedVersion);
    javas = list || [];
    const sel = $('set-java');
    sel.innerHTML = '';
    const cur = settings.javaPath;
    const opts = [...javas.map(j => j.path)];
    if (cur && !opts.includes(cur)) opts.push(cur);
    if (!opts.includes('java')) opts.push('java (PATH)');
    opts.forEach(p => {
      const opt = document.createElement('option');
      opt.value = p === 'java (PATH)' ? 'java' : p;
      const major = (javas.find(j => j.path === p) || {}).major;
      opt.textContent = p + (major ? `  (Java ${major})` : '');
      if ((cur || 'java') === opt.value) opt.selected = true;
      sel.appendChild(opt);
    });
    void required;
    updateJavaHint();
  } catch {}
}
function updateJavaHint() {
  const el = $('java-need-hint');
  if (!el) return;
  if (!selectedVersion) { el.textContent = ''; return; }
  const need = (() => {
    const v = String(selectedVersion);
    const m = v.match(/^(\d+)\.(\d+)(?:\.(\d+))?/);
    if (!m) return 17;
    const M = +m[1], m2 = +m[2], p = +(m[3] || 0);
    if (M >= 26 || M === 25) return 25;
    if (M >= 24) return 21;
    if (M === 1) {
      if (m2 > 20 || (m2 === 20 && p >= 5)) return 21;
      if (m2 >= 18) return 17;
      if (m2 === 17) return 16;
      return 8;
    }
    return 21;
  })();
  el.textContent = `· ${selectedVersion} needs Java ${need}+`;
}
function renderRecentServers() {
  const box = $('recent-servers');
  box.innerHTML = '';
  (settings.recentServers || []).forEach(h => {
    const b = document.createElement('button');
    b.className = 'chip'; b.type = 'button'; b.textContent = h;
    b.onclick = () => { $('play-server').value = h; };
    box.appendChild(b);
  });
}

/* ---------- Mods ---------- */
async function loadMods() {
  if (!settings) return;
  const mods = await api.modsList(settings.gameDir);
  modsCache = mods;
  let checks = [];
  try {
    checks = await api.modsCheck({ gameDir: settings.gameDir, mc: selectedVersion, launcherLoader: settings.loader || 'vanilla' });
  } catch {}
  modChecks = checks;
  const byName = Object.fromEntries(checks.map(c => [c.name, c]));
  const compatBadge = (c) => {
    if (!c) return '';
    const map = { ok: ['compat-ok', '✓'], bad: ['compat-bad', '✗'], warn: ['compat-warn', '!'], unknown: ['compat-unknown', '?'] };
    const [cls, mark] = map[c.status] || map.unknown;
    const reason = (c.loader !== 'unknown' ? c.loader + ' · ' : '') + (c.reason || '');
    return `<span class="badge ${cls}" title="${esc(reason)}">${mark} ${esc(c.loader)}</span>`;
  };
  const list = $('mod-list');
  list.innerHTML = mods.length ? '' : '<div class="muted" style="padding:18px">No mods installed yet — search above or add a custom .jar.</div>';
  mods.forEach(m => {
    const div = document.createElement('div');
    div.className = 'item' + (m.enabled ? '' : ' mod-disabled');
    div.innerHTML =
      `<div class="item-main"><span>🧩</span><div><b>${esc(m.display)}</b> ` +
      `<span class="badge">${(m.size / 1024).toFixed(0)} KB</span>${compatBadge(byName[m.name])}` +
      `${m.enabled ? '' : '<span class="badge">disabled</span>'}` +
      `</div></div>`;
    const acts = document.createElement('div');
    acts.className = 'item-actions';
    const tgl = document.createElement('label');
    tgl.className = 'toggle';
    tgl.title = m.enabled ? 'Disable mod' : 'Enable mod';
    tgl.innerHTML = `<input type="checkbox" ${m.enabled ? 'checked' : ''}><span class="track"></span>`;
    tgl.querySelector('input').onchange = async (e) => {
      e.target.disabled = true;
      try {
        modsCache = await api.modsToggle({ gameDir: settings.gameDir, name: m.name });
        await loadMods();
        toast(`${m.display} ${m.enabled ? 'disabled' : 'enabled'}`, 'ok');
      } catch (err) { toast(err.message, 'error'); e.target.checked = m.enabled; }
      finally { e.target.disabled = false; }
    };
    acts.appendChild(tgl);
    const del = document.createElement('button');
    del.textContent = 'Delete';
    del.className = 'btn-danger';
    del.onclick = async (ev) => {
      ev.target.disabled = true;
      if (!await confirmModal({ title: 'Delete mod?', text: m.display, okLabel: 'Delete', danger: true })) { ev.target.disabled = false; return; }
      try { await api.modsDelete({ gameDir: settings.gameDir, name: m.name }); await loadMods(); toast('Mod deleted', 'ok'); }
      catch (e) { toast(e.message, 'error'); }
      finally { ev.target.disabled = false; }
    };
    acts.appendChild(del);
    div.appendChild(acts);
    list.appendChild(div);
  });
  const on = mods.filter(m => m.enabled).length;
  $('stat-mods').textContent = `${on}/${mods.length}`;
}

function loaderLabel() {
  return settings && settings.loader && settings.loader !== 'vanilla' ? settings.loader : null;
}
function updateLoaderStatus() {
  const el = $('loader-status');
  if (!el || !settings) return;
  const L = loaderLabel();
  if (!L) { el.textContent = 'Vanilla — mods are stored but will not load.'; return; }
  const pid = (settings.loaderProfiles || {})[`${L}:${selectedVersion}`];
  el.textContent = pid
    ? `${L} installed for ${selectedVersion || '?'} ✓ — mods will load on Launch.`
    : `${L} selected but NOT installed for ${selectedVersion || '?'} — hit "Install loader".`;
}

async function installLoader() {
  const L = $('loader-select').value;
  settings.loader = L;
  settings = await api.settingsSave({ loader: L });
  if (L === 'vanilla') { updateLoaderStatus(); renderPlayVersions(); return; }
  if (!selectedVersion) { toast('Select a version first', 'error'); return; }
  const btn = $('btn-loader-install');
  btn.disabled = true;
  $('loader-status').textContent = `Installing ${L} for ${selectedVersion}…`;
  gotoTab('logs');
  log(`> Installing ${L} for ${selectedVersion}...`);
  try {
    const { profileId, loaderVersion, settings: s } = await api.loaderInstall({ loader: L, mc: selectedVersion, gameDir: settings.gameDir });
    settings = s;
    log(`> ${L} ${loaderVersion} ready as "${profileId}"`);
    toast(`${L} ${loaderVersion} installed`, 'ok');
    loadMods();
  } catch (e) {
    log('Loader install failed: ' + e.message);
    toast('Loader install failed: ' + e.message, 'error');
  } finally {
    btn.disabled = false;
    updateLoaderStatus();
    renderPlayVersions();
  }
}

function fmtDownloads(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(1) + 'M';
  if (n >= 1e3) return (n / 1e3).toFixed(1) + 'k';
  return String(n || 0);
}

async function searchMods() {
  const q = $('mods-search').value.trim();
  if (!q) return;
  const box = $('mods-results');
  box.innerHTML = '<div class="skeleton">Searching Modrinth…</div>';
  try {
    const hits = await api.modsSearch(q);
    box.innerHTML = hits.length ? '' : '<div class="muted" style="padding:18px">No mods found.</div>';
    hits.forEach(h => {
      const div = document.createElement('div');
      div.className = 'item mod-row';
      const icon = h.icon
        ? `<img class="mod-icon" src="${esc(h.icon)}" alt="" loading="lazy" onerror="this.remove()" />`
        : `<span class="mod-icon mod-icon-fallback">🧩</span>`;
      div.innerHTML = `${icon}<span class="mod-info"><b>${esc(h.title)}</b> <span class="badge">⬇ ${fmtDownloads(h.downloads)}</span><div class="muted">${esc((h.desc || '').slice(0, 140))}</div></span>`;
      const btn = document.createElement('button');
      btn.textContent = 'Install';
      btn.onclick = () => installModFlow(h, btn);
      div.appendChild(btn);
      box.appendChild(div);
    });
  } catch (e) {
    box.innerHTML = `<div class="muted" style="padding:18px">Search failed: ${esc(e.message)}</div>`;
  }
}

async function installModFlow(h, btn) {
  btn.disabled = true;
  const old = btn.textContent;
  btn.textContent = '…';
  try {
    const loader = $('mods-loader').value;
    const myLoader = settings.loader && settings.loader !== 'vanilla' ? settings.loader : null;
    const { fallback, versions } = await api.modsVersions({ projectId: h.id, gameVersion: selectedVersion, loader });
    if (!versions.length) throw new Error('No files for ' + loader + (selectedVersion ? ' on ' + selectedVersion : ''));
    // version picker modal
    const opts = versions.slice(0, 6).map((v, i) =>
      `<option value="${i}" ${i === 0 ? 'selected' : ''}>${esc(v.number)} · MC ${esc((v.game || []).slice(0, 3).join(', ') || '?')} · [${esc((v.loaders || []).join(','))}]</option>`
    ).join('');
    let extra = `<select id="modal-verpick" class="select">${opts}</select>`;
    if (!myLoader) extra += `<p class="muted" style="margin:10px 0 0">No loader installed — the mod won't load until you install Fabric/Quilt. Continue anyway?</p>`;
    else if (fallback) extra += `<p class="muted" style="margin:10px 0 0">No ${esc(loader)} file for ${esc(selectedVersion || '?')} — showing closest builds, may not work.</p>`;
    const ok = await confirmModal({ title: `Install ${h.title}?`, text: `${versions.length} compatible file(s) found. Pick one:`, extraHTML: extra, okLabel: 'Install' });
    if (!ok) { btn.disabled = false; btn.textContent = old; return; }
    const idx = Number((document.getElementById('modal-verpick') || {}).value || 0);
    const v = versions[idx] || versions[0];
    const fam = (l) => (l === 'fabric' || l === 'quilt') ? 'fabric-family' : l;
    if (myLoader && !v.loaders.includes(myLoader) && !(fam(loader) === 'fabric-family' && fam(myLoader) === 'fabric-family')) {
      const force = await confirmModal({ title: 'Loader mismatch', text: `"${h.title}" ${v.number} is for [${v.loaders.join(', ')}] but your loader is ${myLoader}. It likely won't work. Install anyway?`, okLabel: 'Install anyway', danger: true });
      if (!force) { btn.disabled = false; btn.textContent = old; return; }
    }
    if (fallback) log(`> Note: no ${loader} file for ${selectedVersion}, installing ${v.number} (${(v.game.slice(0, 3)).join(', ')}) instead — may not work.`);
    await api.modsDownload({ fileUrl: v.file.url, fileName: v.file.name, gameDir: settings.gameDir });
    toast(`Installed ${h.title} ${v.number}`, 'ok');
    const modLoader = fam(loader);
    if (modLoader && (!settings.loader || settings.loader === 'vanilla')) {
      settings = await api.settingsSave({ loader: modLoader });
      if ($('loader-select')) $('loader-select').value = modLoader;
      log(`> Auto-selected ${modLoader} loader (from installed mod) — it installs itself on Launch.`);
      updateLoaderStatus(); renderPlayVersions();
    }
    loadMods();
  } catch (e) {
    toast('Install failed: ' + e.message, 'error');
    log('Mod install failed: ' + e.message);
  } finally {
    btn.disabled = false; btn.textContent = 'Install';
  }
}

/* ---------- Game running state ---------- */
function setGameRunning(running) {
  gameRunning = running;
  $('btn-kill').classList.toggle('hidden', !running);
  $('side-game-row').classList.toggle('hidden', !running);
  $('btn-launch').disabled = running;
  if (running) {
    $('dl-bar').style.width = '100%';
    $('dl-text').textContent = 'Game running…';
  }
}

/* ---------- Launch ---------- */
async function doLaunch() {
  const acc = accounts.find(a => a.id === $('play-account').value) || activeAccount();
  if (!acc) { toast('Add an account first', 'error'); gotoTab('accounts'); return; }
  if (acc.id !== activeAccountId) await setActiveAccount(acc.id);
  if (!selectedVersion) { toast('Select a version first', 'error'); gotoTab('versions'); return; }
  if (acc.type !== 'microsoft') {
    const go = await confirmModal({
      title: 'Offline account — online servers will reject you',
      text: `"${acc.name}" is offline. Premium / online-mode servers (Hypixel, etc.) answer with "Failed to log in: Invalid session". Only cracked or offline-mode servers accept it. Sign in with Microsoft instead to play anywhere.`,
      okLabel: 'Launch anyway'
    });
    if (!go) { gotoTab('accounts'); return; }
  }
  const serverHost = ($('play-server').value || '').trim();
  // persist server choice immediately so refresh keeps it
  try { settings = await api.settingsSave({ serverHost, lastVersion: selectedVersion, lastAccountId: acc.id }); renderRecentServers(); } catch {}
  gotoTab('logs');
  log(`> Launch request: ${selectedVersion} as ${acc.name}${serverHost ? ' → ' + serverHost : ''}`);
  $('btn-launch').disabled = true;
  $('dl-bar').style.width = '2%';
  $('dl-text').textContent = 'Starting…';
  try {
    await api.launch({ account: acc, version: selectedVersion, settings: { ...settings, serverHost } });
    log('> Done');
    toast(`Launched ${selectedVersion}`, 'ok');
  } catch (e) {
    log('Launch failed: ' + e.message);
    toast('Launch failed: ' + e.message, 'error', 5000);
  } finally { $('btn-launch').disabled = gameRunning; }
}
async function doKill() {
  try {
    const killed = await api.killGame();
    toast(killed ? 'Kill signal sent' : 'No game running', killed ? 'ok' : 'info');
  } catch (e) { toast(e.message, 'error'); }
}

/* ---------- Name forge (offline username creator) ---------- */
function renderNameSuggestions(names) {
  const box = $('name-suggestions');
  if (!box) return;
  box.innerHTML = '';
  (names || []).forEach(n => {
    const b = document.createElement('button');
    b.className = 'chip'; b.type = 'button'; b.textContent = n;
    b.title = 'Click to use this name';
    b.onclick = () => { $('new-account').value = n; $('new-account').focus(); };
    box.appendChild(b);
  });
}
async function forgeNames() {
  const box = $('name-suggestions');
  const style = ($('name-style') || {}).value || 'mixed';
  try {
    if (box) box.innerHTML = '<span class="muted">Forging names…</span>';
    const names = await api.accountsSuggestNames({ count: 6, style });
    renderNameSuggestions(names);
    if (names[0] && !$('new-account').value) $('new-account').value = names[0];
  } catch (e) { toast(e.message, 'error'); if (box) box.innerHTML = ''; }
}

/* ---------- Events ---------- */
$('btn-add-account').onclick = async () => {
  const name = $('new-account').value;
  try {
    accounts = await api.accountsAdd(name);
    $('new-account').value = '';
    activeAccountId = accounts[accounts.length - 1].id;
    renderAccounts();
    toast('Account added', 'ok');
    pushPresence(document.querySelector('.nav.active')?.dataset?.tab || 'accounts');
  } catch (e) { toast(e.message, 'error'); }
};
$('new-account').onkeydown = (e) => { if (e.key === 'Enter') $('btn-add-account').click(); };
if ($('btn-dice-name')) $('btn-dice-name').onclick = forgeNames;
$('btn-ms-login').onclick = async () => {
  const btn = $('btn-ms-login');
  btn.disabled = true;
  $('ms-status').textContent = 'Waiting for Microsoft login popup...';
  log('> Microsoft login started...');
  try {
    accounts = await api.accountsMicrosoft();
    activeAccountId = accounts[accounts.length - 1].id;
    renderAccounts();
    $('ms-status').textContent = 'Logged in ✓';
    log('> Microsoft login success: ' + (accounts[accounts.length - 1]?.name || ''));
    toast('Microsoft login success', 'ok');
  } catch (e) {
    $('ms-status').textContent = '';
    toast(e.message, 'error', 5000);
    log('Microsoft login failed: ' + e.message);
  } finally { btn.disabled = false; setTimeout(() => $('ms-status').textContent = '', 4000); }
};
$('btn-refresh-versions').onclick = () => loadVersions(true);
$('version-search').oninput = renderVersions;
$('version-filter').onchange = renderVersions;
$('version-installed-only').onchange = renderVersions;
$('play-version').onchange = (e) => { selectedVersion = e.target.value; renderPlayVersions(); loadMods(); updateJavaHint(); refreshJavaList(); pushPresence('play'); };
$('play-account').onchange = (e) => setActiveAccount(e.target.value);
$('loader-select').onchange = async (e) => {
  settings = await api.settingsSave({ loader: e.target.value });
  updateLoaderStatus();
  renderPlayVersions();
  loadMods();
};
$('btn-loader-install').onclick = installLoader;
$('set-minram').oninput = (e) => $('minram-val').textContent = e.target.value + 'G';
$('set-maxram').oninput = (e) => $('maxram-val').textContent = e.target.value + 'G';
$('res-presets').addEventListener('click', (e) => {
  const b = e.target.closest('.chip');
  if (!b) return;
  $('set-width').value = b.dataset.w;
  $('set-height').value = b.dataset.h;
});
$('ram-presets').addEventListener('click', (e) => {
  const b = e.target.closest('.chip');
  if (!b) return;
  $('set-minram').value = b.dataset.minram;
  $('set-maxram').value = b.dataset.maxram;
  $('minram-val').textContent = b.dataset.minram + 'G';
  $('maxram-val').textContent = b.dataset.maxram + 'G';
  toast(`RAM preset ${b.dataset.minram}G–${b.dataset.maxram}G picked — hit Save`, 'info');
});
if ($('set-accent')) $('set-accent').onchange = async (e) => {
  settings = await api.settingsSave({ accent: e.target.value });
  applyAppearance();
};
if ($('set-motion')) $('set-motion').onchange = async (e) => {
  settings = await api.settingsSave({ reduceMotion: e.target.checked });
  applyAppearance();
};
if ($('set-debuglogs')) $('set-debuglogs').onchange = async (e) => {
  settings = await api.settingsSave({ showDebugLogs: e.target.checked });
  rerenderLogs();
};
if ($('logs-autoscroll')) $('logs-autoscroll').onchange = async (e) => {
  try { settings = await api.settingsSave({ logsFollow: e.target.checked }); } catch {}
};
if ($('btn-open-saves')) $('btn-open-saves').onclick = () => api.folderOpen(settings.gameDir + '/saves');
if ($('btn-open-shots')) $('btn-open-shots').onclick = () => api.folderOpen(settings.gameDir + '/screenshots');
$('btn-browse-gamedir').onclick = async () => {
  const p = await api.pickDir($('set-gamedir').value);
  if (p) $('set-gamedir').value = p;
};
$('btn-browse-java').onclick = async () => {
  const p = await api.pickJava();
  if (p) {
    settings = await api.settingsSave({ javaPath: p });
    await refreshJavaList();
  }
};
$('set-java').onchange = async (e) => {
  settings = await api.settingsSave({ javaPath: e.target.value });
};
$('set-autojava').onchange = async (e) => {
  settings = await api.settingsSave({ autoJava: e.target.checked });
  toast(e.target.checked ? 'Auto-Java on' : 'Auto-Java off — using selected Java', 'info');
};
$('btn-save-settings').onclick = async () => {
  let minG = Number($('set-minram').value), maxG = Number($('set-maxram').value);
  if (minG > maxG) { [minG, maxG] = [maxG, minG]; toast('Swapped min/max RAM', 'info'); }
  if (sysInfo && maxG > Math.max(2, Math.floor(sysInfo.totalGB) - 1)) {
    toast(`Max RAM ${maxG}G is high for a ${sysInfo.totalGB}G system — game may stutter`, 'info', 5000);
  }
  settings = await api.settingsSave({
    gameDir: $('set-gamedir').value, javaPath: $('set-java').value,
    autoJava: $('set-autojava').checked,
    minRam: minG + 'G', maxRam: maxG + 'G',
    width: Number($('set-width').value), height: Number($('set-height').value),
    fullscreen: $('set-fullscreen').checked,
    jvmArgs: $('set-jvmargs').value,
    launchAction: $('set-launchaction').value,
    launchTimeoutSec: Number($('set-timeout').value),
    accent: $('set-accent').value,
    reduceMotion: $('set-motion').checked,
    showDebugLogs: $('set-debuglogs').checked,
    logsFollow: $('logs-autoscroll').checked,
    discordPresence: $('set-discord') ? $('set-discord').checked : true,
    discordClientId: $('set-discord-id') ? $('set-discord-id').value.trim() : '',
    discordDownloadUrl: $('set-discord-dl') ? $('set-discord-dl').value.trim() : '',
    jvmPreset: $('set-jvmpreset') ? $('set-jvmpreset').value : 'balanced',
    showSnow: $('set-showsnow') ? $('set-showsnow').checked : true,
    checkUpdates: $('set-checkupdates') ? $('set-checkupdates').checked : true,
    serverHost: ($('play-server').value || '').trim()
  });
  renderRecentServers();
  applyAppearance();
  applySnow();
  syncNewSettingsUI();
  rerenderLogs();
  refreshDiscordStatus();
  pushPresence('settings');
  $('settings-status').textContent = 'Saved ✓';
  toast('Settings saved', 'ok');
  setTimeout(() => $('settings-status').textContent = '', 2000);
};
if ($('set-discord')) $('set-discord').onchange = async (e) => {
  settings = await api.settingsSave({ discordPresence: e.target.checked });
  refreshDiscordStatus();
  pushPresence('settings');
};
if ($('btn-test-discord')) $('btn-test-discord').onclick = async () => {
  const id = ($('set-discord-id').value || '').trim();
  settings = await api.settingsSave({ discordClientId: id, discordPresence: $('set-discord').checked, discordDownloadUrl: ($('set-discord-dl').value || '').trim() });
  try {
    await api.presenceUpdate({ screen: 'settings', account: activeAccount()?.name || null, version: selectedVersion });
    const st = await api.presenceStatus();
    if (st.connected) toast('Connected ✓ — your profile now says Cinder, not Minecraft', 'ok', 5000);
    else if (!st.clientIdSet) toast('Save an Application ID first (see step-by-step above)', 'error');
    else toast('Discord not reachable — open the Discord desktop app, then hit Test again', 'error', 5000);
  } catch (e) { toast(e.message, 'error'); }
  refreshDiscordStatus();
};
if ($('discord-help-link')) $('discord-help-link').onclick = (e) => {
  e.preventDefault();
  confirmModal({
    title: 'Discord status — how it works',
    text: 'Nothing to set up: keep the Discord desktop app open and your profile shows Cinder automatically (players never touch the dev portal — the ID is built in). “Minecraft” instead of Cinder? That is Discord auto-detecting Java because we never connected: hit Test until Connected ✓. Still losing? Discord → Settings → Registered Games → remove the Minecraft/javaw entry → restart Discord → Test again.',
    okLabel: 'Got it'
  }).then(() => {});
};
$('btn-open-gamedir').onclick = () => api.folderOpen(settings.gameDir);
$('btn-open-mods').onclick = () => api.folderOpen(settings.gameDir + '/mods');
$('btn-refresh-mods').onclick = loadMods;
$('btn-mods-search').onclick = searchMods;
$('mods-search').onkeydown = (e) => { if (e.key === 'Enter') searchMods(); };
$('btn-add-jar').onclick = async () => {
  try {
    const added = await api.modsImportJar(settings.gameDir);
    if (added.length) { log('> Added custom jars: ' + added.join(', ')); toast(`Added ${added.length} jar(s)`, 'ok'); loadMods(); }
  } catch (e) { toast(e.message, 'error'); }
};
$('btn-clear-logs').onclick = () => { logLines = []; $('logs').innerHTML = ''; };
$('btn-copy-logs').onclick = async () => {
  try { await navigator.clipboard.writeText(logLines.map(l => l.text).join('\n')); toast('Logs copied', 'ok'); }
  catch { toast('Copy failed', 'error'); }
};
$('logs-search').oninput = rerenderLogs;
$('btn-launch').onclick = doLaunch;
$('btn-kill').onclick = doKill;
$('btn-kill-side').onclick = doKill;

/* ---------- Custom titlebar ---------- */
try {
  const tb = api;
  if ($('tb-min')) $('tb-min').onclick = () => tb.winMin && tb.winMin();
  if ($('tb-max')) $('tb-max').onclick = () => tb.winMax && tb.winMax();
  if ($('tb-close')) $('tb-close').onclick = () => tb.winClose && tb.winClose();
  const bar = document.querySelector('.titlebar');
  if (bar) bar.addEventListener('dblclick', (e) => {
    if (e.target.closest('.tb-controls')) return;
    if (tb.winMax) tb.winMax();
  });
} catch {}

api.onLog((msg) => log(msg));
api.onProgress((p) => {
  if (p.task && p.total) {
    const pct = Math.round((p.task / p.total) * 100);
    $('dl-bar').style.width = pct + '%';
    $('dl-text').textContent = `${p.type || ''} ${pct}% (${p.task}/${p.total})`;
  } else if (p.type) {
    $('dl-text').textContent = `${p.type}…`;
  }
});
api.onGameStarted(() => {
  setGameRunning(true);
  try {
    const acc = activeAccount();
    api.presenceUpdate({ inGame: true, account: acc ? acc.name : null, version: selectedVersion, serverHost: ($('play-server').value || '').trim(), loader: settings.loader });
  } catch {}
});
api.onGameClosed(() => {
  setGameRunning(false);
  $('dl-bar').style.width = '0%';
  $('dl-text').textContent = 'Game closed. Ready to launch';
  pushPresence('play');
});

/* ============ Cinder v2 — dock, topbar, servers, palette (appended, non-breaking) ============ */
const FEATURED_SERVERS = [
  { name: 'Hypixel', ip: 'mc.hypixel.net', tag: 'Minigames', cat: 'minigames', cls: 'srv-hypixel', img: 'assets/img/news-servers.png', desc: 'The biggest Java network — BedWars, SkyBlock, SkyWars and more.' },
  { name: 'PikaNetwork', ip: 'play.pika-network.net', tag: 'Minigames', cat: 'minigames', cls: 'srv-craft', img: 'assets/img/news-servers.png', desc: 'Cracked-friendly minigames: BedWars, Practice, Survival.' },
  { name: 'GommeHD', ip: 'gommehd.net', tag: 'Minigames', cat: 'minigames', cls: 'srv-gomme', img: 'assets/img/news-servers.png', desc: 'Europe’s minigame giant — BedWars, SkyWars, CityBuild.' },
  { name: 'Minewind', ip: 'minewind.com', tag: 'SMP', cat: 'smp', cls: 'srv-minewind', img: 'assets/img/stage-bg.png', desc: 'Hardcore factions SMP — griefing and raiding allowed.' },
  { name: '2b2t', ip: '2b2t.org', tag: 'Anarchy', cat: 'anarchy', cls: 'srv-2b2t', img: 'assets/img/news-builds.jpg', desc: 'The oldest anarchy server. No rules, no mercy, long queue.' },
  { name: 'WilderCraft', ip: 'play.wildercraft.net', tag: 'SMP', cat: 'smp', cls: 'srv-wurst', img: 'assets/img/news-mods.jpg', desc: 'Chill community SMP with claims, jobs and events.' }
];
let serversFilter = 'all';

function quickJoin(ip) {
  const input = $('play-server');
  if (input) input.value = ip;
  const custom = $('servers-custom');
  if (custom) custom.value = ip;
  try {
    const chips = $('recent-servers');
    void chips;
  } catch {}
  gotoTab('play');
  toast(`Quick-join set to ${ip} — hit Launch`, 'ok');
  pushPresence('play');
}

function renderServers() {
  const grid = $('servers-grid');
  if (!grid) return;
  const q = (($('servers-search') || {}).value || '').toLowerCase();
  const rows = FEATURED_SERVERS.filter(s =>
    (serversFilter === 'all' || s.cat === serversFilter) &&
    (!q || (s.name + ' ' + s.ip).toLowerCase().includes(q)));
  grid.innerHTML = '';
  if (!rows.length) { grid.innerHTML = '<div class="muted" style="padding:18px">No servers match.</div>'; return; }
  rows.forEach(s => {
    const card = document.createElement('div');
    card.className = 'server-card';
    card.setAttribute('data-ip', s.ip);
    card.innerHTML =
      `<div class="srv-banner ${s.cls}"><img class="srv-img" src="${esc(s.img || '')}" alt="" loading="lazy" onerror="this.remove()"><span class="srv-tag">${esc(s.tag)}</span></div>` +
      `<div class="srv-body"><b>${esc(s.name)}</b><div class="srv-ip">${esc(s.ip)}</div>` +
      `<div class="srv-desc">${esc(s.desc)}</div>` +
      `<div class="srv-actions"><button type="button" data-act="copy">Copy IP</button>` +
      `<button type="button" data-act="join" class="srv-play">Join</button></div></div>`;
    const [copyBtn, joinBtn] = card.querySelectorAll('button');
    copyBtn.onclick = async () => {
      try { await navigator.clipboard.writeText(s.ip); toast(`Copied ${s.ip}`, 'ok'); }
      catch { toast(s.ip, 'info'); }
    };
    joinBtn.onclick = () => quickJoin(s.ip);
    grid.appendChild(card);
  });
  try { refreshServerPings(); } catch {}
}

const bodyUrl = (name) => `https://minotar.net/body/${encodeURIComponent(String(name || 'steve'))}/220.png`;

/* ---- Animated 3D skin stage (skinview3d, vendored — 2D img is the fallback) ---- */
let skinViewer = null, skinViewerOK = false, skinLoadedFor = null;
function ensureViewer() {
  if (skinViewer) return skinViewerOK;
  if (!window.skinview3d) return false;
  try {
    const canvas = $('skin-canvas');
    if (!canvas) return false;
    skinViewer = new window.skinview3d.SkinViewer({
      canvas,
      width: 300,
      height: 340,
      preserveDrawingBuffer: false
    });
    skinViewer.controls.enableZoom = false;
    skinViewer.controls.enablePan = false;
    skinViewer.autoRotate = true;
    skinViewer.autoRotateSpeed = 1.4;
    skinViewer.animation = new window.skinview3d.WalkingAnimation();
    skinViewer.animation.speed = 0.9;
    applySkinAnimState();
    skinViewerOK = true;
  } catch {
    skinViewer = null;
    skinViewerOK = false;
  }
  return skinViewerOK;
}
function applySkinAnimState() {
  const on = $('skin-anim') ? $('skin-anim').checked : true;
  const calm = on && !(settings && settings.reduceMotion);
  try {
    if (skinViewer) {
      if (skinViewer.animation) skinViewer.animation.paused = !calm;
      skinViewer.autoRotate = calm;
    }
    const st = $('stage');
    if (st) st.classList.toggle('still', !on);
  } catch {}
}
async function loadStageSkin(name) {
  if (!name) return;
  if (skinLoadedFor === name && skinViewerOK) return;
  const pa = $('play-avatar'), cv = $('skin-canvas');
  if (!ensureViewer()) return; // 2D fallback img stays visible
  try {
    const url = await api.skinGet(name);
    if (!url) throw new Error('no skin texture');
    await skinViewer.loadSkin(url, { model: 'auto-detect' });
    skinLoadedFor = name;
    if (pa) pa.style.display = 'none';
    if (cv) cv.style.display = '';
  } catch {
    if (cv) cv.style.display = 'none';
    if (pa) pa.style.display = '';
  }
}

/* ---- Stage + topbar mirrors (wrap, don't replace, original fns) ---- */
function syncStage() {
  try {
    const acc = activeAccount();
    const pa = $('play-avatar');
    if (pa && acc) {
      if (pa.dataset.body !== acc.name) {
        pa.dataset.body = acc.name;
        pa.onerror = function () { this.onerror = null; this.src = avatarUrl(acc.name, 128); };
        pa.src = bodyUrl(acc.name);
      }
      if (!skinViewerOK) pa.style.display = '';
    }
    loadStageSkin(acc ? acc.name : null);
    const greet = $('play-greeting');
    if (greet) greet.textContent = acc ? `${acc.name}` : 'Welcome to Cinder';
    const sub = $('launch-sub');
    if (sub) {
      const L = settings && settings.loader && settings.loader !== 'vanilla' ? settings.loader : 'vanilla';
      const C = clientState && clientState.installed ? ' +client' : '';
      sub.textContent = selectedVersion ? `${selectedVersion} · ${L}${C}` : 'pick a version';
    }
    const il = $('instance-label');
    if (il) {
      const L = settings && settings.loader && settings.loader !== 'vanilla' ? ' · ' + settings.loader : '';
      const C = clientState && clientState.installed ? ' · client' : '';
      il.textContent = (selectedVersion || 'no version') + L + C;
    }
    const tn = $('top-account-name'), ta = $('top-account-avatar'), ts = $('top-account-sub');
    if (tn) tn.textContent = acc ? acc.name : 'No account';
    if (ta) {
      if (acc) { ta.src = avatarUrl(acc.name, 64); ta.style.display = ''; }
      else ta.style.display = 'none';
    }
    if (ts) ts.textContent = !acc ? 'click to add' : acc.type === 'microsoft' ? 'online' : 'offline';
  } catch {}
}

const _syncAccountUI_v2 = syncAccountUI;
syncAccountUI = function () { _syncAccountUI_v2(); syncStage(); };
const _renderPlayVersions_v2 = renderPlayVersions;
renderPlayVersions = function () { _renderPlayVersions_v2(); syncStage(); };
const _applyAppearance_v2 = applyAppearance;
applyAppearance = function () { _applyAppearance_v2(); applySkinAnimState(); };
const _setGameRunning_v2 = setGameRunning;
setGameRunning = function (running) {
  _setGameRunning_v2(running);
  try {
    const tp = $('top-presence');
    if (tp) tp.style.background = running ? '#7dff9a' : '';
  } catch {}
};

/* tab history for back/forward */
const _gotoTab_v2 = gotoTab;
let tabHist = ['play'], tabHistIdx = 0;
gotoTab = function (name) {
  _gotoTab_v2(name);
  try {
    tabHist = tabHist.slice(0, tabHistIdx + 1);
    if (tabHist[tabHistIdx] !== name) { tabHist.push(name); tabHistIdx++; }
  } catch {}
};

/* stage + topbar controls */
try {
  if ($('top-account')) $('top-account').onclick = () => gotoTab('accounts');
  if ($('global-search')) $('global-search').onclick = () => openPalette();
  if ($('bell-btn')) $('bell-btn').onclick = () => gotoTab('logs');
  if ($('instance-pill')) $('instance-pill').onclick = () => gotoTab('versions');
  if ($('version-arrow')) $('version-arrow').onclick = () => gotoTab('versions');
  if ($('nav-back')) $('nav-back').onclick = () => { if (tabHistIdx > 0) { tabHistIdx--; _gotoTab_v2(tabHist[tabHistIdx]); } };
  if ($('nav-fwd')) $('nav-fwd').onclick = () => { if (tabHistIdx < tabHist.length - 1) { tabHistIdx++; _gotoTab_v2(tabHist[tabHistIdx]); } };
  if ($('skin-anim')) $('skin-anim').onchange = () => applySkinAnimState();
  document.querySelectorAll('[data-goto]').forEach(b => {
    b.onclick = () => gotoTab(b.dataset.goto);
  });
  if ($('servers-search')) $('servers-search').oninput = renderServers;
  if ($('servers-filters')) $('servers-filters').addEventListener('click', (e) => {
    const b = e.target.closest('.chip');
    if (!b) return;
    serversFilter = b.dataset.f || 'all';
    document.querySelectorAll('#servers-filters .chip').forEach(c => c.classList.toggle('chip-on', c === b));
    renderServers();
  });
  if ($('servers-join')) $('servers-join').onclick = () => {
    const ip = (($('servers-custom') || {}).value || '').trim();
    if (!ip) { toast('Paste a server IP first', 'error'); return; }
    quickJoin(ip);
  };
  if ($('servers-custom')) $('servers-custom').onkeydown = (e) => { if (e.key === 'Enter') $('servers-join').click(); };
} catch {}

/* initial paint for new surfaces */
try { renderServers(); syncStage(); } catch {}

/* ---- Command palette (Ctrl+K) ---- */
let paletteSel = 0;
function paletteCommands() {
  return [
    { icon: '▶', label: 'Launch game now', hint: 'Ctrl+Enter', run: () => doLaunch() },
    { icon: '⌂', label: 'Go to Home', hint: 'Ctrl+1', run: () => gotoTab('play') },
    { icon: '◉', label: 'Go to Servers', hint: '', run: () => gotoTab('servers') },
    { icon: '▤', label: 'Go to Versions', hint: 'Ctrl+2', run: () => gotoTab('versions') },
    { icon: '☺', label: 'Go to Accounts', hint: 'Ctrl+3', run: () => gotoTab('accounts') },
    { icon: '🧩', label: 'Go to Mods', hint: 'Ctrl+4', run: () => gotoTab('mods') },
    { icon: '⚙', label: 'Go to Settings', hint: 'Ctrl+5', run: () => gotoTab('settings') },
    { icon: '≡', label: 'Go to Logs', hint: 'Ctrl+6', run: () => gotoTab('logs') },
    { icon: '✕', label: 'Kill running game', hint: '', run: () => doKill() },
    { icon: '☺', label: 'Open Skins', hint: 'Ctrl+7', run: () => { gotoTab('skins'); if (typeof initSkinTabViewer === 'function') initSkinTabViewer(); } },
    { icon: '📦', label: 'Open Worlds', hint: 'Ctrl+8', run: () => { gotoTab('worlds'); if (typeof loadWorlds === 'function') loadWorlds(); } },
    { icon: '⟳', label: 'Check for updates now', hint: '', run: () => { if (typeof checkUpdatesFlow === 'function') checkUpdatesFlow(true); } },
    { icon: '⧉', label: 'Copy diagnostics', hint: '', run: () => { if (typeof runDiagCopy === 'function') runDiagCopy(); } },
    { icon: '⚡', label: 'Open Client', hint: 'Ctrl+9', run: () => { gotoTab('client'); if (typeof refreshClientState === 'function') refreshClientState(); } }
  ];
}
function openPalette() {
  const bd = $('palette-backdrop');
  if (!bd) return;
  bd.classList.remove('hidden');
  const inp = $('palette-input');
  inp.value = '';
  paletteSel = 0;
  renderPalette('');
  setTimeout(() => inp.focus(), 30);
}
function closePalette() {
  const bd = $('palette-backdrop');
  if (bd) bd.classList.add('hidden');
}
function renderPalette(q) {
  const list = $('palette-list');
  if (!list) return;
  const query = (q || '').toLowerCase();
  let items = paletteCommands().filter(c => !query || c.label.toLowerCase().includes(query));
  const vhits = (versions || []).filter(v => query && v.id.toLowerCase().includes(query)).slice(0, 4)
    .map(v => ({ icon: '▤', label: `Play ${v.id}`, hint: v.type, run: () => { selectedVersion = v.id; renderVersions(); renderPlayVersions(); persistLastVersion(); gotoTab('play'); closePalette(); } }));
  const shits = FEATURED_SERVERS.filter(s => query && (s.name + ' ' + s.ip).toLowerCase().includes(query)).slice(0, 3)
    .map(s => ({ icon: '◉', label: `Join ${s.name} (${s.ip})`, hint: s.tag, run: () => { closePalette(); quickJoin(s.ip); } }));
  items = [...shits, ...vhits, ...items].slice(0, 12);
  list.innerHTML = '';
  if (!items.length) { list.innerHTML = '<div class="muted" style="padding:16px">No matches.</div>'; return; }
  paletteSel = Math.max(0, Math.min(paletteSel, items.length - 1));
  items.forEach((c, i) => {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'palette-item' + (i === paletteSel ? ' sel' : '');
    b.innerHTML = `<span class="palette-ico">${esc(c.icon)}</span><span>${esc(c.label)}</span><small>${esc(c.hint || '')}</small>`;
    b.onclick = () => { closePalette(); c.run(); };
    b.onmousemove = () => { paletteSel = i; list.querySelectorAll('.palette-item').forEach((el, j) => el.classList.toggle('sel', j === i)); };
    list.appendChild(b);
  });
  list._items = items;
}
try {
  if ($('palette-input')) {
    $('palette-input').addEventListener('input', (e) => { paletteSel = 0; renderPalette(e.target.value); });
    $('palette-input').addEventListener('keydown', (e) => {
      const list = $('palette-list');
      const items = (list && list._items) || [];
      if (e.key === 'ArrowDown') { e.preventDefault(); paletteSel = Math.min(items.length - 1, paletteSel + 1); renderPalette(e.target.value); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); paletteSel = Math.max(0, paletteSel - 1); renderPalette(e.target.value); }
      else if (e.key === 'Enter') { e.preventDefault(); const c = items[paletteSel]; closePalette(); if (c) c.run(); }
    });
  }
  if ($('palette-backdrop')) $('palette-backdrop').addEventListener('click', (e) => { if (e.target.id === 'palette-backdrop') closePalette(); });
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      const bd = $('palette-backdrop');
      if (bd && !bd.classList.contains('hidden')) closePalette();
      else openPalette();
      return;
    }
    if (e.key === 'Escape') closePalette();
  });
} catch {}

/* ============ Cinder v2.0 — settings: snow, JVM preset, updates, diagnostics ============ */
function applySnow() {
  try { document.body.dataset.snow = (settings && settings.showSnow === false) ? 'off' : 'on'; } catch {}
}
function syncNewSettingsUI() {
  try {
    if ($('set-jvmpreset')) $('set-jvmpreset').value = (settings && settings.jvmPreset) || 'balanced';
    if ($('set-showsnow')) $('set-showsnow').checked = !(settings && settings.showSnow === false);
    if ($('set-checkupdates')) $('set-checkupdates').checked = !(settings && settings.checkUpdates === false);
  } catch {}
}
async function checkUpdatesFlow(manual) {
  manual = !!manual;
  const el = $('updates-status');
  try {
    if (el) el.textContent = 'Checking for updates...';
    const r = await api.updateCheck();
    if (!r || r.error) {
      const msg = 'Update check failed: ' + ((r && r.error) || 'unknown error');
      if (el) el.textContent = msg;
      try { log('> ' + msg); } catch {}
      if (manual) toast(msg, 'error');
      return r;
    }
    if (r.newer) {
      const msg = 'Update available: ' + r.latest + ' (you have ' + r.current + ')';
      if (el) el.textContent = msg + ' — see GitHub releases';
      try { log('> ' + msg + (r.url ? ' -> ' + r.url : '')); } catch {}
      toast(msg + ' — download from GitHub', 'info', 6000);
      if (r.url && typeof api.linkOpen === 'function') {
        confirmModal({ title: 'Update available', text: msg + '. Open the releases page to download it?', okLabel: 'Open releases' })
          .then((go) => { if (go) api.linkOpen(r.url).catch((e) => toast('Could not open link: ' + e.message, 'error')); })
          .catch(() => {});
      }
    } else {
      if (el) el.textContent = 'Up to date (' + r.current + ')';
      if (manual) toast('Up to date (' + r.current + ')', 'ok');
    }
    return r;
  } catch (e) {
    const msg = 'Update check failed: ' + ((e && e.message) || e);
    if (el) el.textContent = msg;
    if (manual) toast(msg, 'error');
  }
}
async function bootHook() {
  try {
    if (settings && settings.checkUpdates === false) return;
    await checkUpdatesFlow(false);
  } catch {}
}
async function runDiagCopy() {
  const el = $('diag-status');
  try {
    if (el) el.textContent = 'Collecting diagnostics...';
    const d = await api.diagRun();
    await navigator.clipboard.writeText(JSON.stringify(d, null, 2));
    if (el) el.textContent = 'Diagnostics copied to clipboard';
    toast('Diagnostics copied to clipboard', 'ok');
    try { log('> Diagnostics copied to clipboard'); } catch {}
  } catch (e) {
    const msg = 'Copy failed: ' + ((e && e.message) || e);
    if (el) el.textContent = msg;
    toast(msg, 'error');
  }
}
if ($('set-jvmpreset')) {
  $('set-jvmpreset').onchange = async (e) => {
    try { settings = await api.settingsSave({ jvmPreset: e.target.value }); toast('JVM preset saved: ' + settings.jvmPreset, 'ok'); }
    catch (err) { toast(err.message, 'error'); }
  };
}
if ($('set-showsnow')) {
  $('set-showsnow').onchange = async (e) => {
    try { settings = await api.settingsSave({ showSnow: e.target.checked }); applySnow(); }
    catch (err) { toast(err.message, 'error'); }
  };
}
if ($('set-checkupdates')) {
  $('set-checkupdates').onchange = async (e) => {
    try { settings = await api.settingsSave({ checkUpdates: e.target.checked }); }
    catch (err) { toast(err.message, 'error'); }
  };
}
if ($('btn-check-updates')) $('btn-check-updates').onclick = () => checkUpdatesFlow(true);
if ($('btn-copy-diag')) $('btn-copy-diag').onclick = () => runDiagCopy();

/* ============ Cinder v2.0 — Worlds (singleplayer backups) ============ */
function worldsFmtSize(bytes) {
  const n = Number(bytes) || 0;
  if (n >= 1024 * 1024 * 1024) return (n / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
  if (n >= 1024 * 1024) return (n / (1024 * 1024)).toFixed(1) + ' MB';
  if (n >= 1024) return (n / 1024).toFixed(0) + ' KB';
  return n + ' B';
}
function worldsFmtDate(mtime) {
  try {
    const t = new Date(Number(mtime));
    if (Number.isNaN(t.getTime())) return '';
    return t.toLocaleString();
  } catch { return ''; }
}
function worldsRowHTML(w) {
  const size = worldsFmtSize(w.size);
  const date = worldsFmtDate(w.mtime);
  return '<div class="item worlds-row"><div class="item-main"><span>🌍</span><div><b>' + esc(w.name) + '</b> ' +
    '<span class="badge">' + esc(size) + '</span>' +
    (date ? '<div class="item-sub">' + esc(date) + '</div>' : '') +
    '</div></div><div class="item-actions">' +
    '<button type="button" data-act="backup" data-name="' + esc(w.name) + '">Backup</button>' +
    '<button type="button" data-act="restore" data-name="' + esc(w.name) + '">Restore</button>' +
    '<button type="button" data-act="delete" data-name="' + esc(w.name) + '" class="btn-danger">Delete</button>' +
    '</div></div>';
}
async function loadWorlds() {
  const box = $('worlds-list');
  if (!box) return;
  box.innerHTML = '<div class="skeleton">Loading worlds…</div>';
  try {
    if (!settings || !settings.gameDir) { box.innerHTML = ''; return; }
    const worlds = await api.worldsList(settings.gameDir);
    box.innerHTML = (worlds || []).map(worldsRowHTML).join('');
  } catch (e) {
    box.innerHTML = '<div class="muted" style="padding:18px">Failed: ' + esc(e.message) + '</div>';
  }
}
if ($('worlds-refresh')) $('worlds-refresh').onclick = () => loadWorlds();
if ($('worlds-open')) $('worlds-open').onclick = () => api.folderOpen(settings.gameDir + '/saves');
if ($('worlds-list')) $('worlds-list').addEventListener('click', async (e) => {
  const btn = e.target.closest ? e.target.closest('button[data-act]') : null;
  if (!btn) return;
  const act = btn.dataset.act;
  const name = btn.dataset.name;
  btn.disabled = true;
  try {
    if (act === 'backup') {
      const r = await api.worldsBackup({ gameDir: settings.gameDir, name });
      toast('Backed up ' + name + ' → ' + r.file + ' (' + r.count + ' files)', 'ok');
    } else if (act === 'restore') {
      const ok = await confirmModal({ title: 'Restore backup?', text: 'Restore the newest backup of "' + name + '"? The current world is kept as a .bak copy.', okLabel: 'Restore', danger: true });
      if (!ok) return;
      await api.worldsRestore({ gameDir: settings.gameDir, name });
      toast('Restored ' + name, 'ok');
    } else if (act === 'delete') {
      const ok = await confirmModal({ title: 'Delete world?', text: 'Permanently delete "' + name + '"? This cannot be undone — back it up first if unsure.', okLabel: 'Delete', danger: true });
      if (!ok) return;
      await api.worldsDelete({ gameDir: settings.gameDir, name });
      toast('Deleted ' + name, 'ok');
    } else return;
    await loadWorlds();
  } catch (err) { toast(err.message, 'error'); }
  finally { btn.disabled = false; }
});

/* ============ Cinder v2.0 — live server pings ============ */
let serverPingSeq = 0;
function serverPingSetBadge(card, cls, text, title) {
  let el = card.querySelector('[data-ping]');
  if (!el) {
    el = document.createElement('span');
    el.className = 'ping ping-wait';
    el.setAttribute('data-ping', '');
    const anchor = card.querySelector('.srv-ip');
    if (anchor && anchor.parentNode) anchor.insertAdjacentElement('afterend', el);
    else {
      const body = card.querySelector('.srv-body');
      if (body) body.prepend(el);
      else card.appendChild(el);
    }
  }
  el.className = 'ping ' + cls;
  el.textContent = text;
  if (title) el.title = title;
  else el.removeAttribute('title');
  return el;
}
async function serverPingCheckCard(card, my) {
  let ip = String(card.getAttribute('data-ip') || '').trim();
  if (!ip) {
    const fallback = card.querySelector('.srv-ip');
    ip = fallback ? fallback.textContent.trim() : '';
  }
  if (!ip) return;
  serverPingSetBadge(card, 'ping-wait', '● …', '');
  let res = null;
  try { res = await api.serverStatus(ip); }
  catch (e) { res = { ok: false, error: String((e && e.message) || e || '') }; }
  if (my !== serverPingSeq) return;
  if (res && res.ok) {
    const n = Number(res.players) || 0;
    const ms = Number(res.pingMs) || 0;
    const tip = [res.motd, res.version].filter(Boolean).join(' · ').slice(0, 160);
    serverPingSetBadge(card, 'ping-on', '● ' + n + ' online · ' + ms + 'ms', tip);
  } else {
    serverPingSetBadge(card, 'ping-off', '● offline', (res && res.error) || '');
  }
}
async function refreshServerPings() {
  const grid = document.getElementById('servers-grid');
  if (!grid || !api || typeof api.serverStatus !== 'function') return;
  const cards = Array.from(grid.querySelectorAll('.server-card'));
  if (!cards.length) return;
  const my = ++serverPingSeq;
  const queue = cards.slice();
  const workers = Array.from({ length: Math.min(4, queue.length) }, async () => {
    while (queue.length) {
      if (my !== serverPingSeq) return;
      const card = queue.shift();
      await serverPingCheckCard(card, my);
    }
  });
  await Promise.all(workers);
}

/* ============ Cinder v2.0 — Skins tab 3D viewer ============ */
let skinTabViewer = null;
let skinTabViewerOK = false;
let skinTabTextureURL = null;
let skinTabTextureLabel = 'skin';
let skinTabInitDone = false;
function skinTabSetStatus(msg) {
  const el = $('skin-lib-info');
  if (el) el.textContent = msg;
}
function initSkinTabViewer() {
  if (skinTabViewer) return skinTabViewerOK;
  if (!window.skinview3d) {
    skinTabSetStatus('3D preview unavailable — skin library not loaded.');
    return false;
  }
  const canvas = $('skin-tab-canvas');
  if (!canvas) return false;
  try {
    skinTabViewer = new window.skinview3d.SkinViewer({ canvas, width: 300, height: 340, preserveDrawingBuffer: true });
    skinTabViewer.controls.enableZoom = false;
    skinTabViewer.controls.enablePan = false;
    skinTabViewer.autoRotate = true;
    skinTabViewer.autoRotateSpeed = 1.2;
    skinTabViewer.animation = new window.skinview3d.WalkingAnimation();
    skinTabViewer.animation.speed = 1.0;
    skinTabViewerOK = true;
    skinTabSetStatus('Viewer ready — preview a name or your account.');
  } catch (err) {
    skinTabViewer = null;
    skinTabViewerOK = false;
    skinTabSetStatus('Could not start the 3D viewer.');
  }
  return skinTabViewerOK;
}
function loadSkinTabTexture(url, label) {
  skinTabTextureLabel = label || 'skin';
  if (!url) {
    skinTabSetStatus('No skin found for that name.');
    return Promise.resolve(false);
  }
  if (!initSkinTabViewer()) return Promise.resolve(false);
  skinTabSetStatus('Loading ' + skinTabTextureLabel + '…');
  return skinTabViewer.loadSkin(url, { model: 'auto-detect' }).then(() => {
    skinTabTextureURL = url;
    skinTabSetStatus('Showing ' + skinTabTextureLabel + '.');
    return true;
  }).catch(() => {
    skinTabSetStatus('Could not load that skin.');
    return false;
  });
}
async function skinTabLoadByName() {
  const input = $('skin-name-input');
  const name = input ? input.value.trim() : '';
  if (!name) {
    skinTabSetStatus('Type a username first.');
    if (input) input.focus();
    return;
  }
  skinTabSetStatus('Fetching skin for ' + name + '…');
  try {
    const url = await api.skinGet(name);
    await loadSkinTabTexture(url, name);
  } catch (err) {
    skinTabSetStatus('Could not fetch skin for ' + name + '.');
  }
}
async function skinTabViewAccount() {
  skinTabSetStatus('Loading active profile skin…');
  try {
    const acc = activeAccount();
    if (!acc || !acc.name) {
      skinTabSetStatus('No account yet — add one on the Profiles tab.');
      return;
    }
    const url = await api.skinGet(acc.name);
    await loadSkinTabTexture(url, acc.name);
  } catch (err) {
    skinTabSetStatus('Could not load the account skin.');
  }
}
function skinTabSafeLabel(label) {
  return String(label || 'skin').replace(/[^A-Za-z0-9_-]+/g, '_').slice(0, 32) || 'skin';
}
function downloadSkinTab() {
  const canvas = $('skin-tab-canvas');
  if (!canvas) return;
  if (!skinTabTextureURL || !skinTabViewerOK) {
    skinTabSetStatus('Preview something first, then download.');
    return;
  }
  try {
    const safe = skinTabSafeLabel(skinTabTextureLabel);
    const a = document.createElement('a');
    a.href = canvas.toDataURL('image/png');
    a.download = 'cinder-' + safe + '.png';
    document.body.appendChild(a);
    a.click();
    a.remove();
    skinTabSetStatus('Saved cinder-' + safe + '.png.');
  } catch (err) {
    try {
      const b = document.createElement('a');
      b.href = skinTabTextureURL;
      b.download = 'cinder-' + skinTabSafeLabel(skinTabTextureLabel) + '.png';
      b.target = '_blank';
      document.body.appendChild(b);
      b.click();
      b.remove();
    } catch (err2) {
      skinTabSetStatus('Download blocked by the browser.');
    }
  }
}
function skinTabHandleUploadFile(file) {
  if (!file) return;
  if (file.size > 2 * 1024 * 1024) {
    skinTabSetStatus('That file is too big — skins must be under 2 MB.');
    return;
  }
  if (file.type !== 'image/png' && !/\.png$/i.test(file.name || '')) {
    skinTabSetStatus('Pick a PNG skin file (64x64).');
    return;
  }
  const reader = new FileReader();
  reader.onload = (ev) => {
    const dataUrl = ev && ev.target ? ev.target.result : null;
    if (dataUrl) loadSkinTabTexture(dataUrl, (file.name || 'upload').replace(/\.png$/i, ''));
  };
  reader.onerror = () => skinTabSetStatus('Could not read that file.');
  reader.readAsDataURL(file);
}
function skinTabLazyInit() {
  if (skinTabInitDone) return;
  skinTabInitDone = true;
  initSkinTabViewer();
  try { api.presenceUpdate({ screen: 'skins' }); } catch (err) {}
}
(function skinTabWire() {
  const navBtn = document.querySelector('.nav[data-tab="skins"]');
  if (navBtn) navBtn.addEventListener('click', skinTabLazyInit);
  const section = document.getElementById('tab-skins');
  if (section && typeof MutationObserver !== 'undefined') {
    new MutationObserver(() => { if (section.classList.contains('active')) skinTabLazyInit(); })
      .observe(section, { attributes: true, attributeFilter: ['class'] });
  }
  if ($('skin-tab-load')) $('skin-tab-load').addEventListener('click', skinTabLoadByName);
  if ($('skin-name-input')) $('skin-name-input').addEventListener('keydown', (e) => { if (e.key === 'Enter') skinTabLoadByName(); });
  if ($('skin-tab-view-account')) $('skin-tab-view-account').addEventListener('click', skinTabViewAccount);
  if ($('skin-tab-download')) $('skin-tab-download').addEventListener('click', downloadSkinTab);
  if ($('skin-tab-upload')) $('skin-tab-upload').addEventListener('click', () => { const p = $('skin-file-input'); if (p) p.click(); });
  if ($('skin-file-input')) $('skin-file-input').addEventListener('change', (e) => {
    const f = e.target.files && e.target.files[0];
    skinTabHandleUploadFile(f);
    e.target.value = '';
  });
})();

/* ============ Cinder v2.0 — Client tab (in-game mod install + profiles) ============ */
let clientState = null;
async function refreshClientState() {
  try {
    clientState = await api.clientStatus(settings.gameDir);
  } catch (e) {
    clientState = { installed: false, error: String((e && e.message) || e) };
  }
  paintClientState();
}
function paintClientState() {
  const el = $('client-status'), badge = $('client-state-badge');
  if (!clientState) return;
  if (el) {
    el.textContent = clientState.installed
      ? 'Installed: ' + (clientState.file || 'cinder-client jar present')
      : (clientState.error ? 'Not installed (' + clientState.error + ')' : 'Not installed.');
  }
  if (badge) {
    badge.textContent = clientState.installed ? 'active' : 'not installed';
    badge.className = 'badge ' + (clientState.installed ? 'badge-active' : '');
  }
}
async function loadHudProfiles() {
  const sel = $('hud-profile-list');
  if (!sel) return;
  sel.innerHTML = '';
  try {
    const names = await api.clientProfiles(settings.gameDir);
    if (!names.length) {
      sel.innerHTML = '<option value="">No profiles yet — create one in-game</option>';
      return;
    }
    names.forEach((n) => {
      const opt = document.createElement('option');
      opt.value = n; opt.textContent = n;
      sel.appendChild(opt);
    });
  } catch (e) {
    sel.innerHTML = '<option value="">Failed to list profiles</option>';
  }
}
if ($('client-install')) $('client-install').onclick = async () => {
  const btn = $('client-install');
  btn.disabled = true;
  try {
    gotoTab('logs');
    const r = await api.clientInstall({ gameDir: settings.gameDir });
    toast('Client installed', 'ok');
    log('> Client installed: ' + (r && r.file));
    await refreshClientState();
    await loadHudProfiles();
  } catch (e) { toast(e.message, 'error'); }
  finally { btn.disabled = false; }
};
if ($('client-update')) $('client-update').onclick = async () => {
  const btn = $('client-update');
  btn.disabled = true;
  try {
    gotoTab('logs');
    await api.clientUpdate({ gameDir: settings.gameDir });
    toast('Client updated', 'ok');
    await refreshClientState();
  } catch (e) { toast(e.message, 'error'); }
  finally { btn.disabled = false; }
};
if ($('hud-profile-refresh')) $('hud-profile-refresh').onclick = () => loadHudProfiles();
if ($('hud-profile-set')) $('hud-profile-set').onclick = async () => {
  const sel = $('hud-profile-list');
  const name = sel ? sel.value : '';
  if (!name) { toast('Pick a profile first', 'error'); return; }
  try {
    await api.clientProfileSet({ gameDir: settings.gameDir, name });
    toast('Active HUD profile: ' + name, 'ok');
  } catch (e) { toast(e.message, 'error'); }
};

/* initial paint for new surfaces */
try { renderServers(); syncStage(); } catch {}

(async () => {
  await loadSettings();
  if (settings.lastAccountId) activeAccountId = settings.lastAccountId;
  if (settings.lastVersion) selectedVersion = settings.lastVersion;
  await loadAccounts();
  await loadVersions();
  await Promise.all([loadMods(), loadWorlds()]);
  bootHook().catch(() => {});
  await refreshClientState();
  await loadHudProfiles();
  try { setGameRunning(await api.isGameRunning()); } catch {}
  pushPresence('play');
  forgeNames();
  log('Cinder v2.0 ready. Offline = username only (offline-mode servers). Online = Microsoft login (premium, all servers). Java auto-picks per version. Ctrl+Enter launches.');
})();
