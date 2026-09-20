package gg.cinder.client;

import gg.cinder.client.config.CinderConfig;
import gg.cinder.client.editor.HudEditorScreen;
import gg.cinder.client.hud.HudManager;
import gg.cinder.client.hud.modules.CrosshairModule;
import gg.cinder.client.menu.ClientMenuScreen;
import gg.cinder.client.perf.PerformanceInstaller;
import gg.cinder.client.util.ScreenshotHelper;
import gg.cinder.client.zoom.ZoomController;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.option.KeyBinding.Category;
import net.minecraft.client.util.InputUtil;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Cinder client mod entrypoint.
 *
 * <p>Original code. Implements {@link ClientModInitializer} (the client
 * entrypoint whose {@code onInitializeClient} runs on the physical
 * client; {@code fabric.mod.json} should register this class under the
 * {@code "client"} entrypoint, which is owned by the scaffold agent).</p>
 *
 * <p>Startup order in {@link #onInitializeClient}:</p>
 * <ol>
 *   <li>Remember the instance and load {@link CinderConfig} (local file
 *       plus the launcher shared file merge).</li>
 *   <li>Initialise the HUD via {@code HudManager.init()} (hud agent).</li>
 *   <li>Apply performance defaults via
 *       {@code PerformanceInstaller.install()} (perf agent).</li>
 *   <li>Register the menu keybind (Right Shift, category
 *       {@code "Cinder"}) and open the menu on tick.</li>
 * </ol>
 *
 * <p>Cross-agent API assumptions (HudManager, PerformanceInstaller and
 * HudEditorScreen are owned by other agents): {@code HudManager.init()}
 * exists; {@code PerformanceInstaller.install()} is a static no-arg method
 * that applies the performance defaults at startup (no commands are
 * registered here; the in-game performance UI is the perf agent's
 * {@code menu/PerfScreen}, opened from the client menu).</p>
 */
public class CinderClient implements ClientModInitializer {

    /** Mod logger. */
    public static final Logger LOGGER = LoggerFactory.getLogger("cinder-client");

    /** Mod instance set during {@link #onInitializeClient}. */
    public static CinderClient INSTANCE;

    /** Loaded client config, available after {@link #onInitializeClient}. */
    public static CinderConfig CONFIG;

    /** Translation key for the menu keybind. */
    public static final String MENU_KEY_ID = "key.cinder.menu";

    /** Keybind category, shown verbatim as "Cinder" in the controls screen. */
    public static final String KEY_CATEGORY = "Cinder";

    private KeyBinding menuKey;

    @Override
    public void onInitializeClient() {
        INSTANCE = this;
        CONFIG = CinderConfig.load();
        LOGGER.info("Cinder Client initialising (profile={}, accent={})",
                CONFIG.getHudProfile(), CONFIG.getAccent());

        try {
            HudManager.init();
        } catch (Exception e) {
            LOGGER.error("Cinder Client: HudManager.init() failed", e);
        }

        try {
            PerformanceInstaller.install();
        } catch (Exception e) {
            LOGGER.error("Cinder Client: PerformanceInstaller.install() failed", e);
        }

        menuKey = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                MENU_KEY_ID, InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_RIGHT_SHIFT,
                KeyBinding.Category.create(Identifier.of("cinder", "general"))));

        ZoomController.register();
        ScreenshotHelper.registerKey();

        ClientTickEvents.END_CLIENT_TICK.register(this::onEndTick);
        ClientTickEvents.END_CLIENT_TICK.register(ZoomController::tick);
        ClientTickEvents.END_CLIENT_TICK.register(ScreenshotHelper::pollKeys);
        try {
            CrosshairModule.load();
        } catch (Exception e) {
            LOGGER.error("Cinder Client: CrosshairModule.load() failed", e);
        }
        LOGGER.info("Cinder Client ready. Press Right Shift to open the menu.");
    }

    private void onEndTick(MinecraftClient client) {
        if (menuKey == null) {
            return;
        }
        while (menuKey.wasPressed()) {
            if (client.currentScreen == null) {
                client.setScreen(new ClientMenuScreen(null));
            }
        }
    }

    /**
     * Returns the mod instance.
     *
     * @return the instance, or null before {@link #onInitializeClient} runs
     */
    public static CinderClient getInstance() {
        return INSTANCE;
    }

    /**
     * Returns the loaded config, lazily loading defaults when the mod has
     * not initialised yet (e.g. in unit tests). Never returns null and
     * never throws.
     *
     * @return the client config
     */
    public static synchronized CinderConfig getConfig() {
        if (CONFIG == null) {
            try {
                CONFIG = CinderConfig.load();
            } catch (Exception e) {
                CONFIG = new CinderConfig();
            }
        }
        return CONFIG;
    }

    /**
     * Returns the menu keybind.
     *
     * @return the keybind, or null before {@link #onInitializeClient} runs
     */
    public KeyBinding getMenuKey() {
        return menuKey;
    }

    /**
     * Opens the main client menu.
     *
     * @param parent screen to return to when the menu closes; may be null
     */
    public static void openMenu(Screen parent) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client != null) {
            client.setScreen(new ClientMenuScreen(parent));
        }
    }

    /**
     * Opens the HUD editor directly.
     *
     * @param parent screen to return to when the editor closes; may be null
     */
    public static void openHudEditor(Screen parent) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client != null) {
            client.setScreen(new HudEditorScreen(parent));
        }
    }
}
