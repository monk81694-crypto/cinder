// Cinder client mod -- original code.
// Hold-to-zoom implemented WITHOUT mixins: smoothly narrows the FOV option
// while the zoom key is held and restores the user's value on release.
// [ and ] adjust the zoom divisor (1x..10x). Purely visual, fair-play.
package gg.cinder.client.zoom;

import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.option.SimpleOption;
import net.minecraft.client.util.InputUtil;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;

public final class ZoomController {

    private static final double MIN_DIVISOR = 1.0;
    private static final double MAX_DIVISOR = 10.0;
    private static final double STEP = 0.5;
    private static final double SMOOTH = 0.35;
    private static final int FOV_FLOOR = 30;

    private static KeyBinding zoomKey;
private static KeyBinding widerKey;
private static KeyBinding narrowerKey;
    private static double divisor = 4.0;
    private static double current = -1.0;
    private static int baseFov = 70;
    private static boolean holding;

    private ZoomController() {
    }

    /** Registers hold-C plus [ / ] adjust keys. Idempotent. */
    public static void register() {
        if (zoomKey != null) {
            return;
        }
        KeyBinding.Category cat = KeyBinding.Category.create(Identifier.of("cinder", "general"));
        zoomKey = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.cinder.zoom", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_C, cat));
        widerKey = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.cinder.zoom.in", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_RIGHT_BRACKET, cat));
        narrowerKey = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.cinder.zoom.out", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_LEFT_BRACKET, cat));
    }

    /** Per-tick smoothing. Call every client tick. */
    public static void tick(MinecraftClient mc) {
        if (mc == null || mc.options == null) {
            return;
        }
        while (widerKey != null && widerKey.wasPressed()) {
            divisor = Math.min(MAX_DIVISOR, divisor + STEP);
        }
        while (narrowerKey != null && narrowerKey.wasPressed()) {
            divisor = Math.max(MIN_DIVISOR, divisor - STEP);
        }
        SimpleOption<Integer> fov = mc.options.getFov();
        boolean want = zoomKey != null && zoomKey.isPressed()
                && mc.player != null && mc.world != null && mc.currentScreen == null;
        if (!want) {
            if (holding) {
                fov.setValue(baseFov);
                current = baseFov;
            } else {
                baseFov = fov.getValue();
                current = baseFov;
            }
            holding = false;
            return;
        }
        if (!holding) {
            baseFov = fov.getValue();
            holding = true;
        }
        if (current < 0) {
            current = baseFov;
        }
        double target = Math.max(FOV_FLOOR, (double) baseFov / divisor);
        current += (target - current) * SMOOTH;
        if (Math.abs(current - target) < 0.5) {
            current = target;
        }
        int v = (int) Math.round(current);
        if (v != fov.getValue()) {
            fov.setValue(v);
        }
    }

    /** Current divisor (1.0 = no zoom). Display only. */
    public static double getDivisor() {
        return divisor;
    }
}
