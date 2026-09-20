// loader.js — pure node module (no electron): mod-loader install + mod compat checking.
// Used by main.js via require('./loader').

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const UA = { 'User-Agent': 'Cinder/1.0 (github:cinder-launcher)' };

async function fetchJson(url) {
  const res = await fetch(url, { headers: UA });
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status} for ${url}`);
    err.status = res.status;
    throw err;
  }
  return res.json();
}

const LOADERS = {
  fabric: {
    maven: 'https://maven.fabricmc.net/',
    listUrl: (mc) => `https://meta.fabricmc.net/v2/versions/loader/${mc}`,
    profileUrl: (mc, lv) => `https://meta.fabricmc.net/v2/versions/loader/${mc}/${lv}/profile/json`,
    pickVersion(entries) {
      const stables = entries.filter(e => e.loader && e.loader.stable).map(e => e.loader.version);
      if (stables.length) return stables[0];
      const all = entries.map(e => e.loader && e.loader.version).filter(Boolean);
      return all[0] || null;
    }
  },
  quilt: {
    maven: 'https://maven.quiltmc.org/repository/release/',
    listUrl: (mc) => `https://meta.quiltmc.org/v3/versions/loader/${mc}`,
    profileUrl: (mc, lv) => `https://meta.quiltmc.org/v3/versions/loader/${mc}/${lv}/profile/json`,
    pickVersion(entries) {
      const vers = entries.map(e => e.loader && e.loader.version).filter(Boolean);
      const stable = vers.filter(v => !/(beta|alpha|rc|pre|snapshot)/i.test(v));
      return stable[0] || vers[0] || null;
    }
  }
};

function mavenPath(name) {
  const parts = String(name).split(':');
  const [group, artifact, version, classifier] = parts;
  const dir = `${group.replace(/\./g, '/')}/${artifact}/${version}`;
  const file = `${artifact}-${version}${classifier ? '-' + classifier : ''}.jar`;
  return { dir, file };
}

function mergeArgs(parentArgs, childArgs) {
  if (!parentArgs) return childArgs;
  if (!childArgs) return parentArgs;
  if (typeof parentArgs === 'string' && typeof childArgs === 'string') return parentArgs + ' ' + childArgs;
  if (typeof parentArgs === 'object' && typeof childArgs === 'object' && !Array.isArray(parentArgs) && !Array.isArray(childArgs)) {
    const out = { ...parentArgs };
    for (const k of Object.keys(childArgs)) {
      if (Array.isArray(parentArgs[k]) && Array.isArray(childArgs[k])) out[k] = [...parentArgs[k], ...childArgs[k]];
      else out[k] = childArgs[k];
    }
    return out;
  }
  return childArgs;
}

// Dots break MCLC's version.id parsing (it sniffs MC version from the id),
// so the internal profile id uses underscores: e.g. fabric-0_19_5-1_21_4
function profileId(loader, loaderVer, mc) {
  const u = (s) => String(s).replace(/\./g, '_');
  return `${loader}-${u(loaderVer)}-${u(mc)}`;
}

async function getVanillaJson(mc) {
  const res = await fetch('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json', { headers: UA });
  if (!res.ok) throw new Error('Failed to fetch Mojang version list: ' + res.status);
  const manifest = await res.json();
  const entry = (manifest.versions || []).find(v => v.id === mc);
  if (!entry) throw new Error(`Minecraft ${mc} not found on Mojang's list`);
  return fetchJson(entry.url);
}

async function installLoader({ loader, mc, gameDir, log = () => {} }) {
  const meta = LOADERS[loader];
  if (!meta) throw new Error('Unsupported loader: ' + loader + ' (use fabric or quilt)');

  log(`> Looking up ${loader} builds for ${mc}...`);
  let entries;
  try {
    entries = await fetchJson(meta.listUrl(mc));
  } catch (e) {
    throw new Error(`${loader} has no builds for ${mc} yet. Try an older MC version.`);
  }
  const loaderVer = meta.pickVersion(entries || []);
  if (!loaderVer) throw new Error(`${loader} has no usable builds for ${mc} yet.`);

  log(`> Using ${loader} ${loaderVer}. Downloading profile...`);
  let profile;
  try {
    profile = await fetchJson(meta.profileUrl(mc, loaderVer));
  } catch (e) {
    throw new Error(`${loader} ${loaderVer} has no profile for ${mc} yet (mappings probably not published). Try an older MC version like 1.21.x.`);
  }
  if (!profile || profile.inheritsFrom !== mc || !Array.isArray(profile.libraries)) {
    throw new Error(`Bad ${loader} profile for ${mc}. Try again later.`);
  }

  log(`> Downloading ${profile.libraries.length} loader libraries...`);
  const libDir = path.join(gameDir, 'libraries');
  const enriched = [];
  for (const lib of profile.libraries) {
    const { dir, file } = mavenPath(lib.name);
    const base = String(lib.url || meta.maven).replace(/\/$/, '') + '/';
    const url = base + dir + '/' + file;
    const destDir = path.join(libDir, ...dir.split('/'));
    fs.mkdirSync(destDir, { recursive: true });
    const dest = path.join(destDir, file);
    if (!fs.existsSync(dest)) {
      const res = await fetch(url, { headers: UA });
      if (!res.ok) throw new Error(`Couldn't download ${lib.name} (${res.status}). ${loader} may not fully support ${mc} yet.`);
      fs.writeFileSync(dest, Buffer.from(await res.arrayBuffer()));
    }
    const buf = fs.readFileSync(dest);
    enriched.push({
      ...lib,
      downloads: {
        artifact: {
          path: dir + '/' + file,
          url,
          sha1: crypto.createHash('sha1').update(buf).digest('hex'),
          size: buf.length
        }
      }
    });
  }

  log(`> Merging with vanilla ${mc}...`);
  const vanilla = await getVanillaJson(mc);
  const id = profileId(loader, loaderVer, mc);
  const merged = {
    ...vanilla,
    id,
    mainClass: profile.mainClass,
    minecraftArguments: undefined,
    arguments: mergeArgs(vanilla.arguments, profile.arguments),
    libraries: [...(vanilla.libraries || []), ...enriched],
    releaseTime: profile.releaseTime || vanilla.releaseTime,
    time: profile.time || vanilla.time,
    type: 'release'
  };
  delete merged.inheritsFrom;
  if (merged.minecraftArguments === undefined) delete merged.minecraftArguments;

  const verDir = path.join(gameDir, 'versions', id);
  fs.mkdirSync(verDir, { recursive: true });
  fs.writeFileSync(path.join(verDir, id + '.json'), JSON.stringify(merged, null, 2));
  log(`> Installed ${loader} ${loaderVer} for ${mc} as "${id}"`);
  return { profileId: id, loaderVersion: loaderVer };
}

// ---------- version compare + range matching ----------

function baseVer(v) {
  return String(v).split(/[-+]/)[0].replace(/\.x$/i, '');
}

function cmpVer(a, b) {
  const sa = String(a), sb = String(b);
  const ba = baseVer(sa), bb = baseVer(sb);
  const pa = ba.split('.').map(x => parseInt(x, 10) || 0);
  const pb = bb.split('.').map(x => parseInt(x, 10) || 0);
  const n = Math.max(pa.length, pb.length);
  for (let i = 0; i < n; i++) {
    const x = pa[i] || 0, y = pb[i] || 0;
    if (x !== y) return x < y ? -1 : 1;
  }
  // numeric parts equal: release (no suffix) > pre-release (has -suffix)
  const aPre = /[-+]/.test(sa) ? 1 : 0;
  const bPre = /[-+]/.test(sb) ? 1 : 0;
  if (aPre !== bPre) return aPre < bPre ? 1 : -1;
  return 0;
}

function testComparator(mc, op, ver) {
  ver = ver.trim().replace(/\.x$/i, '');
  if (ver === '' || ver === '*') return true;
  const c = cmpVer(mc, ver);
  switch (op) {
    case '>': return c > 0;
    case '>=': return c >= 0;
    case '<': return c < 0;
    case '<=': return c <= 0;
    case '=':
    case '==': return c === 0;
    case '^': { // same major
      if (c < 0) return false;
      return String(mc).split('.')[0] === String(ver).split('.')[0];
    }
    case '~': { // same major.minor (ignores -pre / +build suffix)
      if (c < 0) return false;
      const pm = baseVer(mc).split('.').slice(0, 2).join('.');
      const qm = baseVer(ver).split('.').slice(0, 2).join('.');
      return pm === qm;
    }
    case 'prefix': {
      const vp = ver.split('.');
      const mp = String(mc).split('.');
      return vp.every((seg, i) => seg.toLowerCase() === 'x' || mp[i] === seg);
    }
    default: return null; // unknown
  }
}

function testAndBranch(mc, branch) {
  // maven-style range: [1.20,1.21) (,1.20] etc, possibly unioned with comma outside brackets
  const tokens = branch.trim().split(/\s+/).filter(Boolean);
  let overall = true;
  let knowAny = false;
  for (const tok of tokens) {
    const m = tok.match(/^([\[\(])([^,\]\)]+)?,([^,\]\)]+)?([\]\)])$/);
    if (m) {
      knowAny = true;
      const [, left, low, high, right] = m;
      if (low && low.trim()) {
        const c = cmpVer(mc, low.trim());
        overall = overall && (left === '[' ? c >= 0 : c > 0);
      }
      if (high && high.trim()) {
        const c = cmpVer(mc, high.trim());
        overall = overall && (right === ']' ? c <= 0 : c < 0);
      }
      continue;
    }
    const cm = tok.match(/^(>=|<=|>|<|==|=|\^|~)?(.+)$/);
    if (!cm) return null;
    let [, op, ver] = cm;
    ver = ver.trim();
    if (!op) op = ver.includes('x') || ver.split('.').length < 3 ? 'prefix' : '=';
    if (ver === '*' || ver.toLowerCase() === 'x') continue;
    const r = testComparator(mc, op, ver);
    if (r === null) return null;
    knowAny = true;
    overall = overall && r;
  }
  return knowAny ? overall : null;
}

// range: string or array of strings. Array = OR. '||' splits OR.
function satisfiesMc(mc, range) {
  if (!range) return null;
  const ors = Array.isArray(range) ? range : String(range).split('||');
  let knowAny = false;
  for (const branch of ors) {
    const r = testAndBranch(mc, String(branch));
    if (r === true) return true;
    if (r !== null) knowAny = true;
  }
  return knowAny ? false : null;
}

// ---------- jar inspection ----------

function parseModsToml(text) {
  const out = { modLoader: null, loaderVersion: null, mcRange: null };
  const ml = text.match(/modLoader\s*=\s*"([^"]+)"/);
  const lv = text.match(/loaderVersion\s*=\s*"([^"]+)"/);
  if (ml) out.modLoader = ml[1];
  if (lv) out.loaderVersion = lv[1];
  // [[dependencies.<modid>]] blocks; find the one for minecraft
  const blocks = text.split(/\[\[dependencies\./);
  for (const b of blocks) {
    const idm = b.match(/^([^\]]+)\]\]/);
    if (!idm) continue;
    if (/minecraft/i.test(idm[1])) {
      const vr = b.match(/versionRange\s*=\s*"([^"]+)"/);
      if (vr) { out.mcRange = vr[1]; break; }
    }
  }
  return out;
}

function checkModJar(jarPath) {
  const AdmZip = require('adm-zip');
  const zip = new AdmZip(jarPath);
  const read = (name) => {
    try {
      const e = zip.getEntry(name);
      return e ? zip.readAsText(e) : null;
    } catch { return null; }
  };
  // Quilt QMD
  const qmd = read('quilt.qmd.json');
  if (qmd) {
    let mcRange = null;
    try {
      const j = JSON.parse(qmd);
      const deps = j.depends || [];
      const mc = deps.find(d => d && /minecraft/i.test(d.id || ''));
      if (mc) mcRange = mc.versions || null;
    } catch {}
    return { loader: 'quilt', mcRange };
  }
  // Fabric (or quilt shim)
  const fmj = read('fabric.mod.json');
  if (fmj) {
    try {
      const j = JSON.parse(fmj);
      if (j.quilt_loader) return { loader: 'quilt', mcRange: j.depends && j.depends.minecraft ? j.depends.minecraft : null };
      const dep = j.depends && j.depends.minecraft;
      return { loader: 'fabric', mcRange: dep || null };
    } catch {}
    return { loader: 'fabric', mcRange: null };
  }
  // Forge / NeoForge
  const toml = read('META-INF/mods.toml');
  if (toml) {
    const t = parseModsToml(toml);
    let loader = 'forge/neoforge';
    if (t.loaderVersion) {
      const low = t.loaderVersion.match(/[\[\(]\s*(\d+)/);
      if (low) loader = parseInt(low[1], 10) >= 40 ? 'forge' : 'neoforge';
    }
    return { loader, mcRange: t.mcRange };
  }
  return { loader: 'unknown', mcRange: null };
}

function resolveStatus(mod, { launcherLoader, mc }) {
  // mod: { loader, mcRange }
  if (!mod || mod.loader === 'unknown') {
    return { status: 'unknown', reason: "Couldn't read mod metadata" };
  }
  const L = launcherLoader || 'vanilla';
  const fam = (l) => (l === 'fabric' || l === 'quilt') ? 'fabric-family' : l;
  if (L === 'vanilla') {
    return { status: 'bad', reason: 'Needs a loader — install Fabric or Quilt in the Mods tab' };
  }
  if (mod.loader === 'forge' || mod.loader === 'neoforge' || mod.loader === 'forge/neoforge') {
    return { status: 'bad', reason: `This is a ${mod.loader === 'forge/neoforge' ? 'Forge/NeoForge' : mod.loader} mod — won't run on ${L}` };
  }
  if (fam(mod.loader) !== fam(L)) {
    // quilt mod on fabric (or vice versa beyond shim)
    return { status: 'bad', reason: `Made for ${mod.loader}, you're running ${L}` };
  }
  let compatNote = null;
  if (mod.loader === 'fabric' && L === 'quilt') compatNote = 'Quilt usually runs Fabric mods';
  const sat = mod.mcRange ? satisfiesMc(mc, mod.mcRange) : null;
  if (sat === false) return { status: 'bad', reason: `Wants MC ${Array.isArray(mod.mcRange) ? mod.mcRange.join(' / ') : mod.mcRange}, you're on ${mc}` };
  if (sat === null && mod.mcRange) return { status: 'warn', reason: compatNote || 'Could not verify MC version range' };
  return { status: 'ok', reason: compatNote || `Works with ${L} on ${mc}` };
}

function checkInstalledMods(modsDir, ctx) {
  if (!fs.existsSync(modsDir)) return [];
  return fs.readdirSync(modsDir)
    .filter(f => f.endsWith('.jar'))
    .map(f => {
      let mod = { loader: 'unknown', mcRange: null };
      try { mod = checkModJar(path.join(modsDir, f)); } catch {}
      return { name: f, ...mod, ...resolveStatus(mod, ctx) };
    });
}

module.exports = {
  LOADERS,
  installLoader,
  profileId,
  satisfiesMc,
  checkModJar,
  resolveStatus,
  checkInstalledMods
};
