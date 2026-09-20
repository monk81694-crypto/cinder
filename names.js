// names.js — offline username generator (pure node, no electron).
// Generates Minecraft-valid names: /^[A-Za-z0-9_]{3,16}$/
// These work instantly on offline-mode / cracked servers. Online-mode
// (premium) servers like Hypixel verify sessions with Mojang, so no
// generated name can join them without a paid Microsoft login — the
// launcher explains that in the UI instead of pretending otherwise.

const ADJECTIVES = [
  'Ashen', 'Blazing', 'Brave', 'Cinder', ' Cloudy', 'Crimson', 'Daring',
  'Ember', 'Frosty', 'Golden', 'Granite', 'Hollow', 'Iron', 'Jolly',
  'Kindly', 'Lucky', 'Misty', 'Nimble', 'Obsidian', 'Peppy', 'Quartz',
  'Rapid', 'Silent', 'Swift', 'Turbo', 'Umber', 'Vivid', 'Witty'
].map((s) => s.trim());

const NOUNS = [
  'Badger', 'Bear', 'Blaze', 'Creeper', 'Diver', 'Dragon', 'Dwarf',
  'Ender', 'Falcon', 'Fox', 'Golem', 'Hawk', 'Knight', 'Lynx', 'Miner',
  'Ocelot', 'Panda', 'Pickaxe', 'Piglin', 'Ranger', 'Rogue', 'Sentry',
  'Slime', 'Sparrow', 'Strider', 'Torch', 'Voyager', 'Warden', 'Wolf'
];

const SUFFIX_STYLES = ['number', 'underscore_number', 'x_wrap', 'plain'];

function randOf(arr, rnd) {
  return arr[Math.floor(rnd() * arr.length)];
}

function sanitize(name) {
  return String(name || '').replace(/[^A-Za-z0-9_]/g, '').slice(0, 16);
}

function isValidName(name) {
  return /^[A-Za-z0-9_]{3,16}$/.test(String(name || ''));
}

/** Generate one valid name. style: 'ember' | 'hero' | 'dark' | 'cute' | 'random' */
function generateName(style = 'random', rnd = Math.random) {
  const pick = () => {
    const adj = randOf(ADJECTIVES, rnd);
    const noun = randOf(NOUNS, rnd);
    let base = adj + noun; // e.g. EmberFox
    // style tweaks
    if (style === 'dark') base = randOf(['Shadow', 'Night', 'Grim', 'Void', 'Onyx'], rnd) + noun;
    if (style === 'cute') base = randOf(['Mochi', 'Pudding', 'Boba', 'Peanut', 'Muffin'], rnd) + noun;
    if (style === 'hero') base = randOf(['Captain', 'Turbo', 'Mega', 'Ultra', 'Super'], rnd) + noun;
    if (style === 'ember') base = randOf(['Ember', 'Cinder', 'Ashen', 'Blaze', 'Pyro'], rnd) + noun;

    const suffix = randOf(SUFFIX_STYLES, rnd);
    if (suffix === 'number' && rnd() < 0.55) base += String(Math.floor(rnd() * 99) + 1);
    if (suffix === 'underscore_number' && rnd() < 0.3) base += '_' + String(Math.floor(rnd() * 99) + 1);
    if (suffix === 'x_wrap' && rnd() < 0.18) base = 'x' + base + 'x';
    // 'plain' = no suffix
    return sanitize(base);
  };

  for (let i = 0; i < 50; i++) {
    const name = pick();
    if (isValidName(name)) return name;
  }
  // ultra-safe fallback
  return 'Player' + (Math.floor(Math.random() * 9000) + 1000);
}

/** Generate n unique suggestions, excluding taken names (case-insensitive). */
function suggestNames(count = 6, { taken = [], style = 'random' } = {}) {
  const seen = new Set(taken.map((s) => String(s).toLowerCase()));
  const out = [];
  let guard = 0;
  while (out.length < count && guard++ < 200) {
    const name = generateName(style === 'mixed' ? randOf(['ember', 'hero', 'dark', 'cute', 'random'], Math.random) : style);
    const low = name.toLowerCase();
    if (!seen.has(low)) {
      seen.add(low);
      out.push(name);
    }
  }
  return out;
}

module.exports = { ADJECTIVES, NOUNS, generateName, suggestNames, isValidName, sanitize };
