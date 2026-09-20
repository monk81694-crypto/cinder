package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

/**
 * Ping overlay: draws {@code "Ping: 42 ms"} or {@code "Ping: -"}.
 *
 * <p>ORIGINAL fair-play code: read-only view of the client play network
 * handler entry latency for the local player. Singleplayer (no handler or
 * no list entry) shows {@code "Ping: -"}. Display only; no packets are
 * sent and no connection state is changed.</p>
 *
 * <p>Never crashes on null handler/entry: every dereference is null-guarded
 * and the fallback glyph is drawn instead.</p>
 *
 * <p>No per-frame allocations except the unavoidable text {@code String}:
 * one reused pre-sized {@link StringBuilder}, no {@code String.format}.</p>
 */
public final class PingModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;

    private final StringBuilder text = new StringBuilder(24);

    public PingModule() {
        super("Ping", 4, 18, 100, 12);
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
        int latency = -1;
        var handler = mc.getNetworkHandler();
        if (handler != null) {
            var entry = handler.getPlayerListEntry(mc.player.getUuid());
            if (entry != null) {
                latency = entry.getLatency();
            }
        }
        text.setLength(0);
        text.append("Ping: ");
        if (latency < 0) {
            text.append("-");
        } else {
            text.append(latency);
            text.append(" ms");
        }
        context.drawText(mc.textRenderer, text.toString(), x, y, TEXT_COLOR, true);
    }
}
