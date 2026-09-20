package gg.cinder.client.config;

/**
 * Per-module HUD settings entry for the Cinder client mod.
 *
 * <p>Original code. Instances of this class live inside
 * {@link CinderConfig#modules}, keyed by HUD module id, and are
 * (de)serialised with Gson. The HUD layer (see {@code HudManager}, owned by
 * another agent) applies sensible per-module default bounds when it first
 * registers a module; the values here are only a neutral fallback.</p>
 */
public class ModuleSettings {

    /** Whether this HUD module is rendered. */
    public boolean enabled = true;

    /** Top-left X of the module frame, in scaled screen pixels. */
    public int x = 4;

    /** Top-left Y of the module frame, in scaled screen pixels. */
    public int y = 4;

    /** Width of the module frame, in scaled screen pixels. */
    public int w = 100;

    /** Height of the module frame, in scaled screen pixels. */
    public int h = 16;

    /** No-arg constructor for Gson. */
    public ModuleSettings() {
    }

    /**
     * Creates a settings entry with explicit values.
     *
     * @param enabled whether the module is rendered
     * @param x top-left X in scaled pixels
     * @param y top-left Y in scaled pixels
     * @param w width in scaled pixels
     * @param h height in scaled pixels
     */
    public ModuleSettings(boolean enabled, int x, int y, int w, int h) {
        this.enabled = enabled;
        this.x = x;
        this.y = y;
        this.w = w;
        this.h = h;
    }

    /**
     * Returns a defensive copy of this entry.
     *
     * @return a new {@code ModuleSettings} with the same values
     */
    public ModuleSettings copy() {
        return new ModuleSettings(enabled, x, y, w, h);
    }

    @Override
    public String toString() {
        return "ModuleSettings{enabled=" + enabled
                + ", x=" + x
                + ", y=" + y
                + ", w=" + w
                + ", h=" + h
                + '}';
    }
}
