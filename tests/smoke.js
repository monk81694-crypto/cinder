// Cinder bug-test suite — catches the boring-but-fatal stuff:
// dangling element IDs, renderer→preload→main IPC mismatches, invalid
// generated names, broken version-range math. Pure node, no Electron.
// Run: npm test
'use strict';

const fs = require('fs');
const path = require('path');
const assert = require('assert');
const { execFileSync } = require('child_process');

const DIR = path.join(__dirname, '..');
const read = (f) => fs.readFileSync(path.join(DIR, f), 'utf8');

let pass = 0;
let fail = 0;
function test(name, fn) {
  try {
    fn();
    pass++;
    console.log(`  ok   ${name}`);
  } catch (e) {
    fail++;
    console.log(`  FAIL ${name}: ${e.message}`);
  }
}

console.log('== syntax ==');
for (const f of ['main.js', 'preload.js', 'renderer.js', 'discord.js', 'names.js', 'loader.js']) {
  test(`${f} parses`, () => {
    execFileSync(process.execPath, ['--check', path.join(DIR, f)], { stdio: 'pipe' });
  });
}

console.log('== element IDs (renderer vs index.html) ==');
test('every $(id) in renderer exists in HTML or is created dynamically', () => {
  const renderer = read('renderer.js');
  const html = read('index.html');
  const used = new Set();
  for (const m of renderer.matchAll(/\$\(\s*['"]([^'"]+)['"]\s*\)/g)) used.add(m[1]);
  const missing = [];
  for (const id of used) {
    const inHtml = html.includes(`id="${id}"`);
    // dynamically created via innerHTML/templates inside renderer itself
    const dynamic = renderer.includes(`id="${id}"`) || renderer.includes(`id='${id}'`);
    if (!inHtml && !dynamic) missing.push(id);
  }
  assert.strictEqual(missing.length, 0, `dangling IDs: ${missing.join(', ')}`);
});

console.log('== IPC wiring (renderer -> preload -> main) ==');
test('every api.method() used by renderer is exposed by preload', () => {
  const renderer = read('renderer.js');
  const preload = read('preload.js');
  const used = new Set([...renderer.matchAll(/\bapi\.(\w+)\s*\(/g)].map((m) => m[1]));
  const exposed = new Set([...preload.matchAll(/(\w+)\s*:\s*\([^)]*\)\s*=>\s*ipcRenderer\.invoke/g)].map((m) => m[1]));
  // event subscriptions use ipcRenderer.on instead of invoke
  for (const m of preload.matchAll(/(\w+)\s*:\s*\(cb\)\s*=>\s*ipcRenderer\.on/g)) exposed.add(m[1]);
  const missing = [...used].filter((m) => !exposed.has(m));
  assert.strictEqual(missing.length, 0, `not exposed: ${missing.join(', ')}`);
});
test('every invoked channel is handled in main', () => {
  const preload = read('preload.js');
  const main = read('main.js');
  const channels = new Set([...preload.matchAll(/ipcRenderer\.invoke\('([^']+)'/g)].map((m) => m[1]));
  const handled = new Set([...main.matchAll(/ipcMain\.handle\('([^']+)'/g)].map((m) => m[1]));
  const missing = [...channels].filter((c) => !handled.has(c));
  assert.strictEqual(missing.length, 0, `unhandled channels: ${missing.join(', ')}`);
});
test('every game:* event main sends has a renderer listener', () => {
  const main = read('main.js');
  const renderer = read('renderer.js');
  for (const ev of ['game:log', 'game:progress', 'game:started', 'game:closed']) {
    assert.ok(main.includes(`'${ev}'`), `main never sends ${ev}`);
  }
  for (const sub of ['onLog', 'onProgress', 'onGameStarted', 'onGameClosed']) {
    assert.ok(renderer.includes(`${sub}(`), `renderer never subscribes ${sub}`);
  }
});

console.log('== names.js ==');
test('200 generations per style are all valid MC names', () => {
  const n = require('../names.js');
  for (const style of ['ember', 'hero', 'dark', 'cute', 'random']) {
    for (let i = 0; i < 200; i++) {
      const name = n.generateName(style);
      assert.ok(n.isValidName(name), `style=${style} produced invalid ${JSON.stringify(name)}`);
    }
  }
});
test('suggestions are unique and exclude taken names', () => {
  const n = require('../names.js');
  const taken = ['Steve', 'EmberFox'];
  const out = n.suggestNames(8, { taken, style: 'mixed' });
  assert.strictEqual(out.length, 8);
  assert.strictEqual(new Set(out.map((s) => s.toLowerCase())).size, 8, 'duplicates suggested');
  for (const s of [...taken, ...out]) assert.ok(n.isValidName(s));
  assert.ok(!out.map((s) => s.toLowerCase()).includes('steve'), 'suggested a taken name');
});

console.log('== loader.js version math ==');
test('satisfiesMc handles real mod ranges', () => {
  const l = require('../loader.js');
  assert.strictEqual(l.satisfiesMc('1.20.1', '>=1.20 <1.21'), true);
  assert.strictEqual(l.satisfiesMc('1.19.4', '>=1.20'), false);
  assert.strictEqual(l.satisfiesMc('1.20.4', '~1.20.1'), true);
  assert.strictEqual(l.satisfiesMc('1.21', '~1.20.1'), false);
  assert.strictEqual(l.satisfiesMc('1.20.1', '1.20.x'), true);
  assert.strictEqual(l.satisfiesMc('1.20.1', '[1.20,1.21)'), true);
  assert.strictEqual(l.satisfiesMc('1.21', '[1.20,1.21)'), false);
  assert.strictEqual(l.satisfiesMc('1.20.1', null), null);
});
test('profileId avoids dots (MCLC version sniffing)', () => {
  const l = require('../loader.js');
  assert.strictEqual(l.profileId('fabric', '0.16.0', '1.21.4'), 'fabric-0_16_0-1_21_4');
});

console.log('== discord.js ==');
test('status() reports install state', () => {
  const st = require('../discord.js').status();
  assert.strictEqual(st.installed, true, 'discord-rpc not installed? run npm install');
  assert.strictEqual(typeof st.enabled, 'boolean');
  assert.strictEqual(typeof st.connected, 'boolean');
});
test('built-in shared Application ID fallback exists (zero-setup presence)', () => {
  const src = read('discord.js');
  assert.ok(src.includes('DEFAULT_CLIENT_ID'), 'no shared-ID constant');
  delete process.env.CINDER_DISCORD_CLIENT_ID;
  const d = require('../discord.js');
  // Manual entry still wins when provided…
  assert.strictEqual(d.getClientId({ discordClientId: '  987654321098765432  ' }), '987654321098765432');
  // …env var is second…
  process.env.CINDER_DISCORD_CLIENT_ID = '111222333444555666';
  assert.strictEqual(d.getClientId({}), '111222333444555666');
  delete process.env.CINDER_DISCORD_CLIENT_ID;
  // …and without either, the result is '' (placeholder) or digits (real ID
  // baked in) — never the placeholder text, never garbage.
  assert.ok(/^(\d{8,}|)$/.test(d.getClientId({})), 'built-in ID fallback broken');
});
test('presence auto-reconnects: retry timer + fresh transport per attempt', () => {
  const src = read('discord.js');
  assert.ok(src.includes('setInterval'), 'no background retry timer');
  assert.ok(src.includes('unref'), 'retry timer must be unref’d so it never blocks exit');
  assert.ok(src.includes('activeClientId'), 'ID change does not rebuild the client');
  assert.ok(/await rpc\.destroy\(\);?\s*\} catch/.test(src), 'dead transport is not discarded before reconnect');
});
test('Test button / launch / presence-update all (re)connect before setting', () => {
  const main = read('main.js');
  const inits = (main.match(/await discord\.init/g) || []).length;
  assert.ok(inits >= 2, `expected init-before-set in presence:update + launch, found ${inits}`);
  assert.ok(read('renderer.js').includes('api.presenceStatus()'), 'Test button never verifies connection');
});
test('Minecraft-vs-Cinder troubleshooting is documented where users look', () => {
  assert.ok(read('README.md').includes('Registered Games'), 'README troubleshooting missing');
  assert.ok(read('index.html').includes('Registered Games'), 'settings card troubleshooting missing');
  assert.ok(read('renderer.js').includes('Registered Games'), 'help popup troubleshooting missing');
});

console.log('== public-ready invariants ==');
test('no old-brand or personal-path leftovers', () => {
  for (const f of ['main.js', 'renderer.js', 'index.html', 'loader.js']) {
    const src = read(f);
    assert.ok(!/Tlaucher/.test(src), `${f} still mentions Tlaucher`);
  }
  assert.ok(!/jdk-25\.0\.4|jdk-17\.0\.20/.test(read('main.js')), 'hardcoded personal JDK path is back');
});
test('new settings keys exist with sane defaults', () => {
  const main = read('main.js');
  for (const key of ['launchAction', 'launchTimeoutSec', 'accent', 'reduceMotion', 'logsFollow', 'showDebugLogs']) {
    assert.ok(main.includes(key), `settings key missing: ${key}`);
  }
});
test('package.json has name, test script, discord dep, assets on disk', () => {
  const pkg = JSON.parse(read('package.json'));
  assert.strictEqual(pkg.name, 'cinder-launcher');
  assert.strictEqual(pkg.author, 'radoslavgeme');
  assert.ok(pkg.scripts && pkg.scripts.test, 'npm test script missing');
  assert.ok(pkg.dependencies['discord-rpc'], 'discord-rpc dep missing');
  for (const a of ['brand/logo.svg', 'brand/wordmark.svg', 'brand/logo-light.svg', 'brand/logo-dark.svg', 'brand/logo-mono.svg', 'brand/logo-mono-white.svg', 'assets/icon.ico', 'assets/icon.png', 'assets/logo-512.png', 'assets/icons/icon-16.png', 'assets/tray.png']) {
    assert.ok(fs.existsSync(path.join(DIR, a)), `${a} missing`);
  }
});
test('icon.ico is a real Windows icon, logo-512.png is a real PNG', () => {
  const ico = fs.readFileSync(path.join(DIR, 'assets/icon.ico'));
  assert.ok(ico[0] === 0 && ico[1] === 0 && ico[2] === 1 && ico[3] === 0, 'bad ICO magic');
  assert.ok(ico.length > 10000, 'icon.ico suspiciously small');
  const png = fs.readFileSync(path.join(DIR, 'assets/logo-512.png'));
  assert.ok(png[0] === 0x89 && png[1] === 0x50 && png[2] === 0x4e && png[3] === 0x47, 'bad PNG magic');
});
test('download-button + credit wiring is complete', () => {
  assert.ok(read('main.js').includes('discordDownloadUrl'), 'main missing download URL setting');
  assert.ok(read('index.html').includes('set-discord-dl'), 'HTML missing download link field');
  assert.ok(read('renderer.js').includes('discordDownloadUrl'), 'renderer missing download URL save/load');
  assert.ok(read('discord.js').includes('Download Cinder'), 'presence button missing');
  assert.ok(read('index.html').includes('radoslavgeme'), 'sidebar credit missing');
  assert.ok(read('index.html').includes('class="credit-bar"'), 'bottom credit bar missing');
  assert.ok(/no bot/i.test(read('index.html')), 'not-a-bot reassurance missing from card');
});

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
