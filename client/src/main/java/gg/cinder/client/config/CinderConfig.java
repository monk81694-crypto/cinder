package gg.cinder.client.config;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import net.fabricmc.loader.api.FabricLoader;

import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.regex.Pattern;

/**
 * Client-side configuration for the Cinder client mod.
 *
 * <p>Original code. Two JSON files are involved, both sharing the same
 * schema ({@code hudProfile}, {@code accent}, {@code clientVersion} plus a
 * {@code modules} map of id to {@link ModuleSettings}):</p>
 *
 * <ul>
 *   <li>Local file: {@code config/cinder-client.json} inside the instance.
 *       This is the file the mod owns: it reads it on startup and rewrites
 *       it on every change via {@link #save()}.</li>
 *   <li>Shared file: {@code <instance>/.cinder-client.json}, written by the
 *       Cinder launcher. On load, the shared file overrides
 *       {@code hudProfile}, {@code accent} and {@code clientVersion} ONLY.
 *       The {@code modules} map (positions, sizes, enabled flags) always
 *       stays local, so the launcher can never clobber the player's HUD
 *       layout.</li>
 * </ul>
 *
 * <p>All JSON parsing uses Gson (bundled with MinecraftClient) with pretty
 * printing. A corrupt file never crashes the game: it is backed up next to
 * the original with a {@code .bak} suffix and loading continues with
 * defaults. The merge itself lives in the pure static
 * {@link #mergeShared(JsonObject, JsonObject)} method, which touches no
 * MinecraftClient classes and is safe to unit test. Only the {@code load()} /
 * {@code save()} no-arg overloads resolve paths through
 * {@code FabricLoader}.</p>
 */
public class CinderConfig {

    /** File name of the mod-owned config inside the config directory. */
    public static final String FILE_NAME = "cinder-client.json";

    /** File name of the launcher-written shared config inside the instance directory. */
    public static final String SHARED_FILE_NAME = ".cinder-client.json";

    /** Default HUD profile name. */
    public static final String DEFAULT_HUD_PROFILE = "default";

    /** Default brand accent (ember). */
    public static final String DEFAULT_ACCENT = "#FF6B1A";

    /** Default client version string. */
    public static final String DEFAULT_CLIENT_VERSION = "1.0.0";

    private static final Pattern ACCENT_PATTERN =
            Pattern.compile("^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$");

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    /** Active HUD profile name. Overridden by the launcher shared file. */
    public String hudProfile = DEFAULT_HUD_PROFILE;

    /** Brand accent as a hex string such as {@code "#FF6B1A"}. Overridden by the launcher shared file. */
    public String accent = DEFAULT_ACCENT;

    /** Client version string. Overridden by the launcher shared file. */
    public String clientVersion = DEFAULT_CLIENT_VERSION;

    /**
     * Per-module settings keyed by HUD module id. Always local: the shared
     * launcher file never touches this map.
     */
    public Map<String, ModuleSettings> modules = new LinkedHashMap<>();

    /** Config directory this instance was loaded from; used by {@link #save()}. */
    private transient Path configDir;

    /**
     * Loads the config using the running game's directories.
     *
     * <p>Never throws: any failure (including running outside the game, as
     * in unit tests) yields defaults.</p>
     *
     * @return the loaded (and merged) config
     */
    public static CinderConfig load() {
        try {
            Path configDir = FabricLoader.getInstance().getConfigDir();
            Path gameDir = FabricLoader.getInstance().getGameDir();
            return load(configDir, gameDir);
        } catch (Throwable t) {
            System.err.println("[cinder-client] Config load failed, using defaults: " + t);
            return new CinderConfig();
        }
    }

    /**
     * Loads the local config from {@code configDir} and merges the launcher
     * shared file from {@code gameDir} on top of it.
     *
     * <p>Never throws: corrupt or missing files yield defaults (corrupt
     * files are backed up with a {@code .bak} suffix).</p>
     *
     * @param configDir directory holding {@code cinder-client.json}; may be null in tests
     * @param gameDir instance directory holding {@code .cinder-client.json}; may be null
     * @return the loaded (and merged) config
     */
    public static CinderConfig load(Path configDir, Path gameDir) {
        CinderConfig config = readLocal(configDir);
        config.configDir = configDir;
        try {
            if (gameDir != null) {
                JsonObject shared = readJsonSafe(gameDir.resolve(SHARED_FILE_NAME));
                if (shared != null) {
                    config = config.withShared(shared);
                }
            }
        } catch (Exception e) {
            System.err.println("[cinder-client] Ignoring launcher shared config: " + e.getMessage());
        }
        return config;
    }

    /**
     * Returns a copy of this config with the launcher shared values applied.
     *
     * @param shared parsed shared file content; may be null (no-op then)
     * @return this config with {@code hudProfile}, {@code accent} and
     *         {@code clientVersion} taken from {@code shared} where present
     */
    public CinderConfig withShared(JsonObject shared) {
        JsonObject base = GSON.toJsonTree(this).getAsJsonObject();
        JsonObject merged = mergeShared(base, shared);
        try {
            CinderConfig result = GSON.fromJson(merged, CinderConfig.class);
            if (result == null) {
                return this;
            }
            if (result.modules == null) {
                result.modules = new LinkedHashMap<>();
            }
            result.configDir = this.configDir;
            result.sanitize();
            return result;
        } catch (Exception e) {
            System.err.println("[cinder-client] Ignoring launcher shared config: " + e.getMessage());
            return this;
        }
    }

    /**
     * Pure merge of the launcher shared file over the local config tree.
     *
     * <p>Only {@code hudProfile}, {@code accent} and
     * {@code clientVersion} are copied from {@code shared} to
     * {@code base} (and only when present as strings). Every other key,
     * notably {@code modules}, is left untouched. Touches no MinecraftClient
     * classes.</p>
     *
     * @param base the local config tree; mutated and returned (a fresh
     *             object is created when null)
     * @param shared the shared launcher file tree; may be null (no-op then)
     * @return the merged {@code base} object
     */
    public static JsonObject mergeShared(JsonObject base, JsonObject shared) {
        if (base == null) {
            base = new JsonObject();
        }
        if (shared == null) {
            return base;
        }
        copySharedString(shared, base, "hudProfile");
        copySharedString(shared, base, "accent");
        copySharedString(shared, base, "clientVersion");
        return base;
    }

    private static void copySharedString(JsonObject shared, JsonObject base, String key) {
        if (shared.has(key)
                && shared.get(key) != null
                && shared.get(key).isJsonPrimitive()
                && shared.get(key).getAsJsonPrimitive().isString()) {
            base.add(key, shared.get(key));
        }
    }

    private static CinderConfig readLocal(Path configDir) {
        if (configDir == null) {
            return new CinderConfig();
        }
        Path file = configDir.resolve(FILE_NAME);
        JsonObject obj = readJsonSafe(file);
        if (obj == null) {
            return new CinderConfig();
        }
        try {
            CinderConfig parsed = GSON.fromJson(obj, CinderConfig.class);
            if (parsed == null) {
                return new CinderConfig();
            }
            if (parsed.modules == null) {
                parsed.modules = new LinkedHashMap<>();
            }
            parsed.sanitize();
            return parsed;
        } catch (Exception e) {
            backupCorrupt(file);
            System.err.println("[cinder-client] Local config corrupt, backed up and reset: " + e.getMessage());
            return new CinderConfig();
        }
    }

    /**
     * Reads a JSON object from disk without ever throwing. A corrupt file
     * is backed up with a {@code .bak} suffix and null is returned; a
     * missing file also yields null (no backup).
     *
     * @param file the file to read; may be null
     * @return the parsed object, or null when missing/unreadable/corrupt
     */
    public static JsonObject readJsonSafe(Path file) {
        if (file == null) {
            return null;
        }
        try {
            if (!Files.isRegularFile(file)) {
                return null;
            }
            String Text = new String(Files.readAllBytes(file), StandardCharsets.UTF_8);
            if (Text.trim().isEmpty()) {
                return null;
            }
            return GSON.fromJson(Text, JsonObject.class);
        } catch (Exception e) {
            backupCorrupt(file);
            System.err.println("[cinder-client] Corrupt JSON at " + file + ", backed up: " + e.getMessage());
            return null;
        }
    }

    private static void backupCorrupt(Path file) {
        try {
            if (file != null && Files.isRegularFile(file)) {
                Path backup = file.resolveSibling(file.getFileName().toString() + ".bak");
                Files.copy(file, backup, StandardCopyOption.REPLACE_EXISTING);
            }
        } catch (Exception ignored) {
            // Best effort only: backing up must never break loading.
        }
    }

    private void sanitize() {
        if (hudProfile == null || hudProfile.isEmpty()) {
            hudProfile = DEFAULT_HUD_PROFILE;
        }
        if (!isValidAccent(accent)) {
            accent = DEFAULT_ACCENT;
        }
        if (clientVersion == null || clientVersion.isEmpty()) {
            clientVersion = DEFAULT_CLIENT_VERSION;
        }
        if (modules == null) {
            modules = new LinkedHashMap<>();
        }
        modules.entrySet().removeIf(entry -> entry.getKey() == null || entry.getValue() == null);
    }

    /**
     * Persists this config to the directory it was loaded from (or the
     * game's config directory when unknown). Never throws.
     */
    public synchronized void save() {
        Path dir = configDir;
        if (dir == null) {
            try {
                dir = FabricLoader.getInstance().getConfigDir();
            } catch (Throwable t) {
                System.err.println("[cinder-client] Skipping config save (no game directories): " + t);
                return;
            }
        }
        save(dir);
    }

    /**
     * Persists this config as pretty-printed JSON, writing to a temp file
     * and moving it over the target (atomically when supported). Never
     * throws.
     *
     * @param dir directory receiving {@code cinder-client.json}; ignored when null
     */
    public synchronized void save(Path dir) {
        if (dir == null) {
            return;
        }
        try {
            Files.createDirectories(dir);
            Path file = dir.resolve(FILE_NAME);
            byte[] data = GSON.toJson(this).getBytes(StandardCharsets.UTF_8);
            Path tmp = dir.resolve(FILE_NAME + ".tmp");
            Files.write(tmp, data);
            try {
                Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
            } catch (AtomicMoveNotSupportedException ex) {
                Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING);
            }
            this.configDir = dir;
        } catch (Exception e) {
            System.err.println("[cinder-client] Failed to save config: " + e.getMessage());
        }
    }

    /** Returns the active HUD profile name. */
    public synchronized String getHudProfile() {
        return hudProfile;
    }

    /** Returns the brand accent hex string (e.g. {@code "#FF6B1A"}). */
    public synchronized String getAccent() {
        return accent;
    }

    /** Returns the client version string. */
    public synchronized String getClientVersion() {
        return clientVersion;
    }

    /**
     * Returns the live modules map. Callers mutating entries directly
     * should call {@link #save()} afterwards (or use the
     * {@code setModule*} helpers, which persist automatically).
     *
     * @return the live id-to-settings map, never null
     */
    public synchronized Map<String, ModuleSettings> getModules() {
        return modules;
    }

    /** Sets the HUD profile and persists. */
    public void setHudProfile(String hudProfile) {
        synchronized (this) {
            this.hudProfile = (hudProfile == null || hudProfile.isEmpty()) ? DEFAULT_HUD_PROFILE : hudProfile;
        }
        save();
    }

    /** Sets the accent (falling back to ember when invalid) and persists. */
    public void setAccent(String accent) {
        synchronized (this) {
            this.accent = isValidAccent(accent) ? accent : DEFAULT_ACCENT;
        }
        save();
    }

    /** Sets the client version and persists. */
    public void setClientVersion(String clientVersion) {
        synchronized (this) {
            this.clientVersion =
                    (clientVersion == null || clientVersion.isEmpty()) ? DEFAULT_CLIENT_VERSION : clientVersion;
        }
        save();
    }

    /**
     * Returns the settings for a module, creating defaults on first use.
     * Does not save; use {@code setModule*} to persist changes.
     *
     * @param id the HUD module id; must not be null
     * @return the (possibly newly created) settings entry
     */
    public synchronized ModuleSettings getModule(String id) {
        if (id == null) {
            throw new IllegalArgumentException("module id must not be null");
        }
        ModuleSettings existing = modules.get(id);
        if (existing == null) {
            existing = new ModuleSettings();
            modules.put(id, existing);
        }
        return existing;
    }

    /** Sets a module's enabled flag and persists. */
    public void setModuleEnabled(String id, boolean enabled) {
        synchronized (this) {
            getModule(id).enabled = enabled;
        }
        save();
    }

    /** Sets a module's bounds and persists. */
    public void setModuleBounds(String id, int x, int y, int w, int h) {
        synchronized (this) {
            ModuleSettings settings = getModule(id);
            settings.x = x;
            settings.y = y;
            settings.w = w;
            settings.h = h;
        }
        save();
    }

    /**
     * Checks an accent value: {@code #RRGGBB} or {@code #AARRGGBB}.
     *
     * @param accent the value to check; may be null
     * @return true when the value is a valid accent hex string
     */
    public static boolean isValidAccent(String accent) {
        return accent != null && ACCENT_PATTERN.matcher(accent).matches();
    }

    /**
     * Converts an accent hex string to a packed ARGB int for rendering.
     *
     * @param accent the hex value; may be null/invalid
     * @param fallback returned when the value is invalid
     * @return the packed ARGB color, or {@code fallback}
     */
    public static int parseAccentOr(String accent, int fallback) {
        if (!isValidAccent(accent)) {
            return fallback;
        }
        try {
            String hex = accent.substring(1);
            if (hex.length() == 6) {
                return 0xFF000000 | (int) Long.parseLong(hex, 16);
            }
            return (int) Long.parseLong(hex, 16);
        } catch (NumberFormatException e) {
            return fallback;
        }
    }

    /**
     * Serialises this config to a Gson tree (test helper).
     *
     * @return this config as a {@code JsonObject}
     */
    public synchronized JsonObject toJsonObject() {
        return GSON.toJsonTree(this).getAsJsonObject();
    }
}
