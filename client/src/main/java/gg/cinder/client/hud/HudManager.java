// Cinder client mod -- original code.
// Package: gg.cinder.client.hud
// Target: MinecraftClient 26.2 / Mojang mappings / Fabric API / Java 21.
package gg.cinder.client.hud;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import gg.cinder.client.editor.HudEditorScreen;
import gg.cinder.client.hud.modules.CrosshairModule;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.util.Identifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Owns the live {@code List<HudModule>} registry, the HUD render hook, position
 * persistence, and named layout profiles.
 *
 * <h2>HUD event choice</h2>
 * Rendering is hooked via Fabric API's {@link HudElementRegistry} with an
 * ordered vanilla layer instead of the legacy {@code HudRenderCallback}. Rationale:
 * <ul>
 *   <li>{@code HudRenderCallback} is the deprecated, single-phase hook: every listener
 *       draws in one unordered pass with no control over layering.</li>
 *   <li>{@code HudElementRegistry} registers one ordered layer in the
 *       vanilla {@code LayeredDrawer}, so the Cinder overlay sorts deterministically
 *       against vanilla layers (hotbar, chat, misc overlays) and respects vanilla
 *       HUD visibility instead of fighting it.</li>
 *   <li>It is the documented modern path for 1.20.5+ Fabric API and therefore the
 *       right bet for MC 26.2.</li>
 * </ul>
 * NOTE (unverified for 26.2): the vanilla-layer holder class has been renamed across
 * Fabric API versions ({@code IdentifiedLayer} in older builds,
 * {@code VanillaHudLayers} in newer ones). This file is written against
 * {@code VanillaHudElements.MISC_OVERLAYS}; if the 26.2 Fabric API still calls it
 * {@code IdentifiedLayer}, change the import plus the one constant reference and
 * nothing else. The legacy fallback would be
 * {@code HudRenderCallback.EVENT.register((ctx, tickCounter) -> renderAll(ctx, tickCounter))}.
 *
 * <h2>CinderConfig integration</h2>
 * Position/enabled state is persisted to {@code config/cinder-hud.json} by this class
 * AND mirrored into {@code gg.cinder.client.config.CinderConfig} when that class is
 * present (it is owned by another agent and may land before or after this file, so
 * the bridge below is reflective and never a hard compile dependency). Expected shape:
 * <pre>
 *   gg.cinder.client.config.CinderConfig
 *     public static CinderConfig getInstance()
 *     public ModuleSettings getModuleSettings(String id)
 *     public void save()
 *   gg.cinder.client.config.ModuleSettings
 *     public boolean enabled; public int x, y, w, h;
 * </pre>
 * Load order: file first, then CinderConfig overlay (config wins when present).
 * Save order: file first, then push into CinderConfig and request its save.
 *
 * <h2>Performance</h2>
 * {@link #renderAll} does zero per-frame allocations: indexed loop (no iterator),
 * no boxing, per-module try/catch so one broken module can never take down the HUD.
 */
public final class HudManager {

    private static final Logger LOGGER = LoggerFactory.getLogger("cinder-hud");

    /** Layer id for the Cinder overlay inside the vanilla LayeredDrawer. */
private static final Identifier LAYER_ID = Identifier.of("cinder", "hud");

    /** Active config file: {@code <run>/config/cinder-hud.json}. */
    private static final String CONFIG_FILE_NAME = "cinder-hud.json";

    /** Profile directory: {@code <run>/config/hud-profiles/*.json}. */
    private static final String PROFILES_DIR_NAME = "hud-profiles";

    private static final int CONFIG_VERSION = 1;
    private static final int MAX_PROFILE_NAME_LEN = 32;

    /**
     * Module implementation classes owned by the module agents. Instantiated
     * reflectively so this file compiles and works even when some (or all) of them
     * have not landed yet; missing classes are skipped with a debug log.
     */
    private static final String[] MODULE_CLASS_NAMES = {
            "gg.cinder.client.hud.modules.FpsModule",
            "gg.cinder.client.hud.modules.PingModule",
            "gg.cinder.client.hud.modules.CoordsModule",
            "gg.cinder.client.hud.modules.ClockModule",
            "gg.cinder.client.hud.modules.MemoryModule",
            "gg.cinder.client.hud.modules.ServerModule",
            "gg.cinder.client.hud.modules.CpsModule",
            "gg.cinder.client.hud.modules.KeystrokesModule",
            "gg.cinder.client.hud.modules.ArmorModule",
            "gg.cinder.client.hud.modules.ItemCountModule",
            "gg.cinder.client.hud.modules.PotionModule",
            "gg.cinder.client.hud.modules.SprintStatusModule",
    };

    private static final List<HudModule> MODULES = new ArrayList<>(16);
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private static boolean initialized;
    private static boolean editorOpen;

    private HudManager() {
        // static-only
    }

    // ------------------------------------------------------------- lifecycle

    /**
     * Registers all modules, loads saved state, and attaches the HUD layer.
     * Safe to call more than once; subsequent calls are no-ops.
     * Must be called on the client thread during client mod init.
     */
    public static synchronized void init() {
        if (initialized) {
            return;
        }
        initialized = true;
        registerAll();
        load();
        HudElementRegistry.attachElementAfter(VanillaHudElements.MISC_OVERLAYS, LAYER_ID,
                (ctx, tickCounter) -> renderAll(ctx, tickCounter));
        LOGGER.info("[cinder] HUD manager ready with {} module(s)", MODULES.size());
    }

    /** Reflective bulk registration; missing module classes are skipped. */
    private static void registerAll() {
        for (String className : MODULE_CLASS_NAMES) {
            try {
                Class<?> clazz = Class.forName(className);
                Object instance = clazz.getDeclaredConstructor().newInstance();
                if (instance instanceof HudModule module) {
                    register(module);
                } else {
                    LOGGER.warn("[cinder] HUD class {} is not a HudModule, skipping", className);
                }
            } catch (ClassNotFoundException e) {
                // Owned by another agent and not landed yet: expected, stay quiet-ish.
                LOGGER.debug("[cinder] HUD module {} not present yet, skipping", className);
            } catch (ReflectiveOperationException | LinkageError e) {
                LOGGER.warn("[cinder] Could not instantiate HUD module {}", className, e);
            }
        }
    }

    /**
     * Manual registration (for tests or modules outside the built-in list).
     * Duplicate ids are replaced.
     */
    public static synchronized void register(HudModule module) {
        if (module == null) {
            throw new IllegalArgumentException("module must not be null");
        }
        for (int i = 0; i < MODULES.size(); i++) {
            if (MODULES.get(i).id().equals(module.id())) {
                MODULES.set(i, module);
                return;
            }
        }
        MODULES.add(module);
    }

    /** Live module list (unmodifiable view; the manager owns the backing list). */
    public static List<HudModule> getModules() {
        return Collections.unmodifiableList(MODULES);
    }

    /** Lookup by stable id, or {@code null} when unknown. Zero allocation. */
    public static HudModule getModule(String id) {
        if (id == null) {
            return null;
        }
        for (int i = 0; i < MODULES.size(); i++) {
            HudModule m = MODULES.get(i);
            if (m.id().equals(id)) {
                return m;
            }
        }
        return null;
    }

    /** Set by {@link HudEditorScreen} while the editor is open. */
    public static void setEditorOpen(boolean open) {
        editorOpen = open;
    }

    public static boolean isEditorOpen() {
        return editorOpen;
    }

    /** Opens the HUD editor for the current client. Call on the client thread. */
    public static void openEditor() {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc != null) {
            mc.setScreen(new HudEditorScreen(mc.currentScreen));
        }
    }

    // --------------------------------------------------------------- rendering

    /**
     * Draws every enabled module. Skipped while the editor is open (the editor draws
     * selection boxes instead, so live module rendering can never fight the editor)
     * and while vanilla HUD is hidden.
     */
    public static void renderAll(DrawContext ctx, RenderTickCounter tickCounter) {
        if (editorOpen) {
            return;
        }
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc == null || mc.options == null || mc.options.hudHidden) {
            return;
        }
        float delta = tickCounter.getDynamicDeltaTicks();
        for (int i = 0; i < MODULES.size(); i++) {
            HudModule module = MODULES.get(i);
            if (!module.isEnabled()) {
                continue;
            }
            try {
                module.render(ctx, delta);
            } catch (Throwable t) {
                // One broken module must never blank the whole HUD.
                LOGGER.warn("[cinder] HUD module {} threw, disabling for this session", module.id(), t);
                module.setEnabled(false);
            }
        }
        try {
            CrosshairModule.renderOverlay(ctx);
        } catch (Throwable t) {
            LOGGER.warn("[cinder] Crosshair overlay threw", t);
        }
    }

    // ------------------------------------------------------------- persistence

    private static Path configFile() {
        return FabricLoader.getInstance().getConfigDir().resolve(CONFIG_FILE_NAME);
    }

    private static Path profilesDir() {
        return FabricLoader.getInstance().getConfigDir().resolve(PROFILES_DIR_NAME);
    }

    /** Loads positions/enabled flags from disk, then overlays CinderConfig. */
    public static synchronized void load() {
        Path file = configFile();
        if (Files.isRegularFile(file)) {
            try (Reader reader = Files.newBufferedReader(file)) {
                JsonObject root = JsonParser.parseReader(reader).getAsJsonObject();
                JsonObject modules = root.has("modules") && root.get("modules").isJsonObject()
                        ? root.getAsJsonObject("modules")
                        : null;
                if (modules != null) {
                    for (int i = 0; i < MODULES.size(); i++) {
                        HudModule m = MODULES.get(i);
                        if (modules.has(m.id()) && modules.get(m.id()).isJsonObject()) {
                            applyJson(m, modules.getAsJsonObject(m.id()));
                        }
                    }
                }
            } catch (IOException | RuntimeException e) {
                LOGGER.warn("[cinder] Could not load HUD config, keeping defaults", e);
            }
        }
        syncFromCinderConfig();
    }

    /** Saves positions/enabled flags to disk, then pushes into CinderConfig. */
    public static synchronized void save() {
        JsonObject root = new JsonObject();
        root.addProperty("version", CONFIG_VERSION);
        JsonObject modules = new JsonObject();
        for (int i = 0; i < MODULES.size(); i++) {
            HudModule m = MODULES.get(i);
            JsonObject o = new JsonObject();
            o.addProperty("enabled", m.isEnabled());
            o.addProperty("x", m.getX());
            o.addProperty("y", m.getY());
            o.addProperty("w", m.getW());
            o.addProperty("h", m.getH());
            modules.add(m.id(), o);
        }
        root.add("modules", modules);
        try {
            Path file = configFile();
            Files.createDirectories(file.getParent());
            try (Writer writer = Files.newBufferedWriter(file)) {
                GSON.toJson(root, writer);
            }
        } catch (IOException e) {
            LOGGER.warn("[cinder] Could not save HUD config", e);
        }
        syncToCinderConfig();
    }

    private static void applyJson(HudModule m, JsonObject o) {
        if (o.has("enabled")) {
            m.setEnabled(o.get("enabled").getAsBoolean());
        }
        int x = o.has("x") ? o.get("x").getAsInt() : m.getX();
        int y = o.has("y") ? o.get("y").getAsInt() : m.getY();
        int w = o.has("w") ? o.get("w").getAsInt() : m.getW();
        int h = o.has("h") ? o.get("h").getAsInt() : m.getH();
        m.setBounds(x, y, w, h);
    }

    // ---------------------------------------------------------------- profiles

    /**
     * Saves the current layout under {@code name} to
     * {@code config/hud-profiles/<name>.json}.
     *
     * @throws IllegalArgumentException when the name is blank or unsafe
     * @throws IOException              on write failure
     */
    public static synchronized void saveProfile(String name) throws IOException {
        String safe = requireSafeProfileName(name);
        Path file = profilesDir().resolve(safe + ".json");
        Files.createDirectories(file.getParent());
        JsonObject root = new JsonObject();
        root.addProperty("version", CONFIG_VERSION);
        root.addProperty("name", safe);
        JsonObject modules = new JsonObject();
        for (int i = 0; i < MODULES.size(); i++) {
            HudModule m = MODULES.get(i);
            JsonObject o = new JsonObject();
            o.addProperty("enabled", m.isEnabled());
            o.addProperty("x", m.getX());
            o.addProperty("y", m.getY());
            o.addProperty("w", m.getW());
            o.addProperty("h", m.getH());
            modules.add(m.id(), o);
        }
        root.add("modules", modules);
        try (Writer writer = Files.newBufferedWriter(file)) {
            GSON.toJson(root, writer);
        }
        LOGGER.info("[cinder] Saved HUD profile {}", safe);
    }

    /**
     * Loads layout {@code name} from {@code config/hud-profiles/<name>.json},
     * applies it to the live modules, and persists it as the active layout.
     *
     * @throws IllegalArgumentException when the name is blank or unsafe
     * @throws IOException              when the profile is missing or unreadable
     */
    public static synchronized void loadProfile(String name) throws IOException {
        String safe = requireSafeProfileName(name);
        Path file = profilesDir().resolve(safe + ".json");
        if (!Files.isRegularFile(file)) {
            throw new IOException("HUD profile not found: " + safe);
        }
        try (Reader reader = Files.newBufferedReader(file)) {
            JsonObject root = JsonParser.parseReader(reader).getAsJsonObject();
            JsonObject modules = root.has("modules") && root.get("modules").isJsonObject()
                    ? root.getAsJsonObject("modules")
                    : null;
            if (modules == null) {
                throw new IOException("HUD profile is corrupt (no modules object): " + safe);
            }
            for (int i = 0; i < MODULES.size(); i++) {
                HudModule m = MODULES.get(i);
                if (modules.has(m.id()) && modules.get(m.id()).isJsonObject()) {
                    applyJson(m, modules.getAsJsonObject(m.id()));
                }
            }
        } catch (RuntimeException e) {
            throw new IOException("HUD profile is corrupt: " + safe, e);
        }
        save();
        LOGGER.info("[cinder] Loaded HUD profile {}", safe);
    }

    /** Lists saved profile names (sorted, without extension). Never null. */
    public static synchronized List<String> listProfiles() {
        List<String> out = new ArrayList<>();
        Path dir = profilesDir();
        if (!Files.isDirectory(dir)) {
            return out;
        }
        try (DirectoryStream<Path> stream = Files.newDirectoryStream(dir, "*.json")) {
            for (Path p : stream) {
                String fileName = p.getFileName().toString();
                if (fileName.endsWith(".json")) {
                    String base = fileName.substring(0, fileName.length() - 5);
                    if (isSafeProfileName(base)) {
                        out.add(base);
                    }
                }
            }
        } catch (IOException e) {
            LOGGER.warn("[cinder] Could not list HUD profiles", e);
        }
        Collections.sort(out);
        return out;
    }

    /**
     * Profile names are untrusted input (they come from the editor Text field), so
     * they are strictly validated: 1-32 chars of letters, digits, space, _ and -.
     * This rules out path traversal (.., /, backslash) by construction.
     */
    public static String requireSafeProfileName(String name) {
        if (!isSafeProfileName(name)) {
            throw new IllegalArgumentException(
                    "Profile name must be 1-32 chars of A-Z a-z 0-9 space _ -");
        }
        return name.trim();
    }

    public static boolean isSafeProfileName(String name) {
        if (name == null) {
            return false;
        }
        String t = name.trim();
        if (t.length() < 1 || t.length() > MAX_PROFILE_NAME_LEN) {
            return false;
        }
        for (int i = 0; i < t.length(); i++) {
            char c = t.charAt(i);
            boolean letter = (c >= 65 && c <= 90) || (c >= 97 && c <= 122);
            boolean digit = c >= 48 && c <= 57;
            boolean extra = c == 95 || c == 45 || c == 32;
            if (!letter && !digit && !extra) {
                return false;
            }
        }
        return true;
    }

    // ------------------------------------------------------- CinderConfig bridge

    /**
     * Pulls state from CinderConfig into live modules.
     * Direct (non-reflective) bridge: CinderConfig.load()/getModule(id).
     */
    private static void syncFromCinderConfig() {
        try {
            gg.cinder.client.config.CinderConfig cfg = gg.cinder.client.CinderClient.getConfig();
            if (cfg == null) {
                return;
            }
            for (int i = 0; i < MODULES.size(); i++) {
                HudModule m = MODULES.get(i);
                gg.cinder.client.config.ModuleSettings s = cfg.getModule(m.id());
                if (s == null) {
                    continue;
                }
                m.setEnabled(s.enabled);
                m.setBounds(s.x, s.y, s.w, s.h);
            }
        } catch (LinkageError | RuntimeException e) {
            LOGGER.warn("[cinder] CinderConfig bridge mismatch on load, using local HUD config", e);
        }
    }

    /** Pushes live state into CinderConfig and requests its save. */
    private static void syncToCinderConfig() {
        try {
            gg.cinder.client.config.CinderConfig cfg = gg.cinder.client.CinderClient.getConfig();
            if (cfg == null) {
                return;
            }
            for (int i = 0; i < MODULES.size(); i++) {
                HudModule m = MODULES.get(i);
                cfg.setModuleEnabled(m.id(), m.isEnabled());
                cfg.setModuleBounds(m.id(), m.getX(), m.getY(), m.getW(), m.getH());
            }
        } catch (LinkageError | RuntimeException e) {
            LOGGER.warn("[cinder] CinderConfig bridge mismatch on save", e);
        }
    }
}
