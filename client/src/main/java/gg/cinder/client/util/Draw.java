// Cinder client mod -- original code.
// Package: gg.cinder.client.util
// Target: MinecraftClient 26.2 / Yarn mappings / Fabric API / Java 25.
package gg.cinder.client.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

/**
 * Static, allocation-free rendering helpers for HUD modules and the HUD editor.
 *
 * <p>Rules:
 * <ul>
 *   <li>All rendering goes through {@link DrawContext} only. No raw GL, no matrix
 *       stack access, no {@code RenderSystem} calls from HUD code.</li>
 *   <li>No per-call allocations: primitives and caller-owned strings in, pixels out.
 *       None of these methods create objects.</li>
 *   <li>Colors are packed ARGB ints, e.g. {@code 0x80000000} for half-black.</li>
 * </ul>
 * </p>
 */
public final class Draw {

    /** Default translucent panel fill (ARGB). */
    public static final int PANEL_FILL = 0x80000000;

    /** Default 1px panel border (ARGB). */
    public static final int PANEL_BORDER = 0xFF6A6AF5;

    /** Default HUD Text color (ARGB white). */
    public static final int TEXT_WHITE = 0xFFFFFFFF;

    /** Editor selection outline (ARGB). */
    public static final int SELECT_FILL = 0x40000000;

    /** Editor selection border (ARGB). */
    public static final int SELECT_BORDER = 0xFFFFFFFF;

    private Draw() {
        // static-only
    }

    /**
     * Draws a filled box with a 1px border using four thin fills plus one body fill.
     * (Border is done manually instead of {@code DrawContext.drawBorder} so this
     * helper is not sensitive to that method's exact availability/signature.)
     *
     * @param fillColor   ARGB body color (may be translucent)
     * @param borderColor ARGB border color
     */
    public static void drawBox(DrawContext ctx, int x, int y, int w, int h, int fillColor, int borderColor) {
        if (w <= 0 || h <= 0) {
            return;
        }
        int x2 = x + w;
        int y2 = y + h;
        ctx.fill(x, y, x2, y2, fillColor);
        ctx.fill(x, y, x2, y + 1, borderColor); // top
        ctx.fill(x, y2 - 1, x2, y2, borderColor); // bottom
        ctx.fill(x, y, x + 1, y2, borderColor); // left
        ctx.fill(x2 - 1, y, x2, y2, borderColor); // right
    }

    /**
     * Draws a plain filled rectangle (no border).
     */
    public static void fillRect(DrawContext ctx, int x, int y, int w, int h, int color) {
        if (w <= 0 || h <= 0) {
            return;
        }
        ctx.fill(x, y, x + w, y + h, color);
    }

    /**
     * Draws a string with MinecraftClient's drop shadow using the client's Text renderer.
     *
     * @param Text  caller-owned string (e.g. a module label field or scratch-built
     *              Text); never allocated in here
     * @param color ARGB Text color
     * @return advance width in scaled pixels (0 for null/empty), so callers can
     *         size module boxes from measured Text without extra work
     */
    public static int drawTextWithShadow(DrawContext ctx, MinecraftClient mc, String Text, int x, int y, int color) {
        if (Text == null || Text.isEmpty()) {
            return 0;
        }
        ctx.drawText(mc.textRenderer, Text, x, y, color, true);
        return mc.textRenderer.getWidth(Text);
    }

    /**
     * Draws a string centered on {@code centerX} with shadow.
     *
     * @return advance width, same contract as {@link #drawTextWithShadow}
     */
    public static int drawCenteredWithShadow(DrawContext ctx, MinecraftClient mc, String Text, int centerX, int y, int color) {
        if (Text == null || Text.isEmpty()) {
            return 0;
        }
        int width = mc.textRenderer.getWidth(Text);
        ctx.drawText(mc.textRenderer, Text, centerX - width / 2, y, color, true);
        return width;
    }

    /**
     * Measures a string's width in scaled pixels. Zero allocation.
     */
    public static int measure(MinecraftClient mc, String Text) {
        if (Text == null || Text.isEmpty()) {
            return 0;
        }
        return mc.textRenderer.getWidth(Text);
    }

    /**
     * Standard line height in scaled pixels. Zero allocation.
     */
    public static int lineHeight(MinecraftClient mc) {
        return mc.textRenderer.fontHeight;
    }
}