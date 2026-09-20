// Cinder client mod -- original code.
// Base class for every Cinder HUD module (FPS, ping, coords, clock, ...).
// Target: Minecraft 1.21.11 / Yarn mappings / Fabric API / Java 21.
package gg.cinder.client.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

/**
 * Base class for HUD modules. Each module has a stable {@code id} (used for
 * config persistence and profiles), a human-readable {@code label}, an
 * {@code enabled} flag, and a bounding box ({@code x, y, w, h}).
 *
 * <h2>Performance contract (applies to ALL subclasses)</h2>
 * Do no per-frame allocations in {@link #render}: reuse the shared
 * {@link #scratch} buffer and plain fields. Keep rendering on
 * {@link DrawContext} only.
 */
public abstract class HudModule {

    /** Stable config key, e.g. {@code "FPS"}. Never rename once shipped. */
    protected final String id;

    /** Human-readable name shown in the HUD editor. Defaults to the id. */
    protected final String label;

    /** Whether this module renders. Persisted via config ModuleSettings by id. */
    protected boolean enabled = true;

    /** Bounding box in scaled pixels. Persisted via CinderConfig. */
    protected int x;
    protected int y;
    protected int w;
    protected int h;

    /** Live client instance for subclasses (textRenderer, player state, ...). */
    protected final MinecraftClient mc = MinecraftClient.getInstance();

    /**
     * Reusable scratch buffer for building display text without allocating.
     * Use via {@link #scratch()}: it clears the buffer and hands it back.
     */
    protected final StringBuilder scratch = new StringBuilder(64);

    /**
     * @param id       stable config id (also used as the display label)
     * @param defaultX default scaled x
     * @param defaultY default scaled y
     * @param defaultW default width (min 1)
     * @param defaultH default height (min 1)
     */
    protected HudModule(String id, int defaultX, int defaultY, int defaultW, int defaultH) {
        if (id == null || id.isEmpty()) {
            throw new IllegalArgumentException("HudModule id must be non-empty");
        }
        this.id = id;
        this.label = id;
        this.x = defaultX;
        this.y = defaultY;
        this.w = Math.max(1, defaultW);
        this.h = Math.max(1, defaultH);
    }

    /**
     * Draws the module. Called every HUD frame while enabled.
     *
     * @param ctx       draw context (all rendering goes through this)
     * @param tickDelta render tick delta for animations
     */
    public abstract void render(DrawContext ctx, float tickDelta);

    /**
     * Bridge for callers that pass the client explicitly; delegates to
     * {@link #render(DrawContext, float)}.
     */
    public final void render(DrawContext ctx, float tickDelta, MinecraftClient ignored) {
        render(ctx, tickDelta);
    }

    /** Stable config id. */
    public final String id() {
        return id;
    }

    /** Stable config id (alias for menu code). */
    public final String getId() {
        return id;
    }

    /** Display label. */
    public final String label() {
        return label;
    }

    /** Display label (alias). */
    public final String getLabel() {
        return label;
    }

    public final boolean isEnabled() {
        return enabled;
    }

    public final void setEnabled(boolean enabled) {
        this.enabled = enabled;
    }

    public final int getX() {
        return x;
    }

    public final int getY() {
        return y;
    }

    public final int getW() {
        return w;
    }

    public final int getH() {
        return h;
    }

    public final void setPosition(int nx, int ny) {
        this.x = nx;
        this.y = ny;
    }

    public final void setSize(int nw, int nh) {
        this.w = Math.max(1, nw);
        this.h = Math.max(1, nh);
    }

    public final void setBounds(int nx, int ny, int nw, int nh) {
        this.x = nx;
        this.y = ny;
        this.w = Math.max(1, nw);
        this.h = Math.max(1, nh);
    }

    /** Hit test against this module's box. */
    public final boolean contains(double mouseX, double mouseY) {
        return mouseX >= x && mouseX < x + w && mouseY >= y && mouseY < y + h;
    }

    /**
     * Hit test for the bottom-right resize handle square.
     *
     * @param handleSize handle edge length in scaled pixels (editor uses 8)
     */
    public final boolean isCornerHandle(double mouseX, double mouseY, int handleSize) {
        return mouseX >= (x + w - handleSize) && mouseX < x + w
                && mouseY >= (y + h - handleSize) && mouseY < y + h;
    }

    /** Clamps the box fully on-screen. Call after any move/resize. */
    public final void clampToScreen(int screenW, int screenH) {
        if (w > screenW) {
            w = Math.max(1, screenW);
        }
        if (h > screenH) {
            h = Math.max(1, screenH);
        }
        if (x < 0) {
            x = 0;
        }
        if (y < 0) {
            y = 0;
        }
        if (x + w > screenW) {
            x = screenW - w;
        }
        if (y + h > screenH) {
            y = screenH - h;
        }
    }

    /** Moves the box, then clamps it on-screen. */
    public final void moveTo(int nx, int ny, int screenW, int screenH) {
        this.x = nx;
        this.y = ny;
        clampToScreen(screenW, screenH);
    }

    /** Resizes the box honoring minimums, then clamps it on-screen. */
    public final void resizeTo(int nw, int nh, int minW, int minH, int screenW, int screenH) {
        this.w = Math.max(nw, Math.max(1, minW));
        this.h = Math.max(nh, Math.max(1, minH));
        clampToScreen(screenW, screenH);
    }

    /**
     * Snaps a coordinate to a grid (pure function, no allocation).
     *
     * @param grid grid size; {@code <= 1} disables snapping
     */
    public static int snapTo(int value, int grid) {
        if (grid <= 1) {
            return value;
        }
        return Math.round((float) value / (float) grid) * grid;
    }

    /** Alias kept for the layout test probe. */
    public static int snap(int value, int grid) {
        return snapTo(value, grid);
    }

    /**
     * Clears and returns the shared scratch buffer. Zero allocation after the
     * initial capacity is reached (buffer only grows, never per frame).
     */
    protected final StringBuilder scratch() {
        scratch.setLength(0);
        return scratch;
    }
}
