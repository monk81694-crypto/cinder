package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

/**
 * FPS overlay: draws {@code "FPS: 240"}.
 *
 * <p>ORIGINAL fair-play code: read-only view of
 * {@code MinecraftClient.getFps()}; display only, never modifies
 * game state, sends no packets, and performs no automation.</p>
 *
 * <p>No per-frame allocations except the unavoidable Text {@code String}:
 * one reused pre-sized {@link StringBuilder}, no {@code String.format},
 * no streams/lambdas, no boxing.</p>
 */
public final class FpsModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;

    private final StringBuilder Text = new StringBuilder(16);

    public FpsModule() {
        super("FPS", 4, 4, 70, 12);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc == null || mc.player == null || mc.world == null) {
            return;
        }
        if (context == null || mc.textRenderer == null) {
            return;
        }
        int fps = mc.getCurrentFps();
        Text.setLength(0);
        Text.append("FPS: ");
        Text.append(fps);
        context.drawText(mc.textRenderer, Text.toString(), x, y, TEXT_COLOR, true);
    }
}
