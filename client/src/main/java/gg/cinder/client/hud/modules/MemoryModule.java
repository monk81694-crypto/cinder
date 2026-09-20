// Cinder client mod -- original code.
// Memory usage overlay, cached twice per second. Off by default.
package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

public final class MemoryModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;

    private final StringBuilder line = new StringBuilder(32);
    private long lastCheck;
    private long usedBytes;
    private long totalBytes;

    public MemoryModule() {
        super("Memory", 4, 200, 130, 12);
        setEnabled(false);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc == null || context == null || mc.textRenderer == null) {
            return;
        }
        long now = System.currentTimeMillis();
        if (now - lastCheck > 500) {
            Runtime runtime = Runtime.getRuntime();
            totalBytes = runtime.totalMemory();
            usedBytes = totalBytes - runtime.freeMemory();
            lastCheck = now;
        }
        line.setLength(0);
        line.append("Mem: ");
        line.append(usedBytes / 1048576L);
        line.append('/');
        line.append(totalBytes / 1048576L);
        line.append("MB");
        context.drawText(mc.textRenderer, line.toString(), x, y, TEXT_COLOR, true);
    }
}
