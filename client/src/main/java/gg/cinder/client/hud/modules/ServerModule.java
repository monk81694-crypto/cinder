// Cinder client mod -- original code.
// Current server address display (or Singleplayer).
package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.ServerInfo;

public final class ServerModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;

    private final StringBuilder line = new StringBuilder(64);

    public ServerModule() {
        super("Server", 4, 214, 160, 12);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc == null || context == null || mc.textRenderer == null) {
            return;
        }
        ServerInfo entry = mc.getCurrentServerEntry();
        line.setLength(0);
        if (entry != null && entry.address != null) {
            line.append(entry.address);
        } else {
            line.append("Singleplayer");
        }
        context.drawText(mc.textRenderer, line.toString(), x, y, TEXT_COLOR, true);
    }
}
