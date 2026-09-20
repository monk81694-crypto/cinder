// Cinder client mod -- original code.
// Cosmetic center-screen marker drawn over the HUD (styles: cross/plus/dot
// with adjustable gap/size/length/color). Never touches aim, reach or hit
// logic. Settings persist in config/cinder-crosshair.json with a bridge for a
// future CinderConfig extras map (toExtras/applyExtras).
package gg.cinder.client.hud.modules;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

public final class CrosshairModule {

    /** Marker style. */
    public enum Style {
        CROSS, PLUS, DOT
    }

    private static Style style = Style.CROSS;
    private static int gap = 3;
    private static int length = 6;
    private static int size = 1;
    private static int color = 0xFFFFFFFF;

    private CrosshairModule() {
    }

    /** Draws the marker at screen center when in-game. Never throws. */
    public static void renderOverlay(DrawContext ctx) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc == null || mc.player == null || mc.world == null || mc.currentScreen != null) {
            return;
        }
        if (mc.options == null || mc.options.hudHidden) {
            return;
        }
        int cx = mc.getWindow().getScaledWidth() / 2;
        int cy = mc.getWindow().getScaledHeight() / 2;
        switch (style) {
            case DOT:
                ctx.fill(cx - size, cy - size, cx + size + 1, cy + size + 1, color);
                break;
            case PLUS:
                ctx.fill(cx - size, cy - length - gap, cx + size + 1, cy - gap + 1, color);
                ctx.fill(cx - size, cy + gap, cx + size + 1, cy + gap + length + 1, color);
                ctx.fill(cx - length - gap, cy - size, cx - gap + 1, cy + size + 1, color);
                ctx.fill(cx + gap, cy - size, cx + gap + length + 1, cy + size + 1, color);
                break;
            case CROSS:
            default:
                ctx.fill(cx - length - gap, cy, cx - gap + 1, cy + 1, color);
                ctx.fill(cx + gap, cy, cx + gap + length + 1, cy + 1, color);
                ctx.fill(cx, cy - length - gap, cx + 1, cy - gap + 1, color);
                ctx.fill(cx, cy + gap, cx + 1, cy + gap + length + 1, color);
                break;
        }
    }

    public static void setStyle(Style style) {
        if (style != null) {
            CrosshairModule.style = style;
        }
    }

    public static void setGap(int gap) {
        CrosshairModule.gap = Math.max(0, Math.min(16, gap));
    }

    public static void setLength(int length) {
        CrosshairModule.length = Math.max(1, Math.min(24, length));
    }

    public static void setSize(int size) {
        CrosshairModule.size = Math.max(1, Math.min(4, size));
    }

    public static void setColor(int color) {
        CrosshairModule.color = color;
    }

    /** Exports settings for a future CinderConfig extras map. */
    public static Map<String, String> toExtras() {
        Map<String, String> map = new HashMap<>();
        map.put("crosshair.style", style.name());
        map.put("crosshair.gap", Integer.toString(gap));
        map.put("crosshair.length", Integer.toString(length));
        map.put("crosshair.size", Integer.toString(size));
        map.put("crosshair.color", Integer.toString(color));
        return map;
    }

    /** Applies settings from a CinderConfig-style extras map. */
    public static void applyExtras(Map<String, String> extras) {
        if (extras == null) {
            return;
        }
        try {
            if (extras.containsKey("crosshair.style")) {
                setStyle(Style.valueOf(extras.get("crosshair.style")));
            }
        } catch (IllegalArgumentException ignored) {
        }
        setGap(parseInt(extras.get("crosshair.gap"), gap));
        setLength(parseInt(extras.get("crosshair.length"), length));
        setSize(parseInt(extras.get("crosshair.size"), size));
        if (extras.containsKey("crosshair.color")) {
            try {
                setColor(Integer.parseInt(extras.get("crosshair.color")));
            } catch (NumberFormatException ignored) {
            }
        }
    }

    private static int parseInt(String value, int fallback) {
        if (value == null) {
            return fallback;
        }
        try {
            return Integer.parseInt(value);
        } catch (NumberFormatException e) {
            return fallback;
        }
    }

    /** Persists settings to config/cinder-crosshair.json. Never throws. */
    public static void save() {
        try {
            Path file = configFile();
            Files.createDirectories(file.getParent());
            Gson gson = new GsonBuilder().setPrettyPrinting().create();
            JsonObject root = new JsonObject();
            for (Map.Entry<String, String> entry : toExtras().entrySet()) {
                root.addProperty(entry.getKey(), entry.getValue());
            }
            try (Writer writer = Files.newBufferedWriter(file)) {
                gson.toJson(root, writer);
            }
        } catch (IOException | RuntimeException ignored) {
        }
    }

    /** Loads settings from config/cinder-crosshair.json. Never throws. */
    public static void load() {
        try {
            Path file = configFile();
            if (!Files.isRegularFile(file)) {
                return;
            }
            try (Reader reader = Files.newBufferedReader(file)) {
                JsonObject root = JsonParser.parseReader(reader).getAsJsonObject();
                Map<String, String> map = new HashMap<>();
                for (String key : new String[] {"crosshair.style", "crosshair.gap",
                        "crosshair.length", "crosshair.size", "crosshair.color"}) {
                    if (root.has(key) && root.get(key).isJsonPrimitive()) {
                        map.put(key, root.get(key).getAsString());
                    }
                }
                applyExtras(map);
            }
        } catch (IOException | RuntimeException ignored) {
        }
    }

    private static Path configFile() {
        return FabricLoader.getInstance().getConfigDir().resolve("cinder-crosshair.json");
    }
}
