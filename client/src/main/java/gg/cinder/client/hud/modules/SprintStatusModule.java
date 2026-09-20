// Cinder client mod -- original code.
// Sprint/sneak status text, visible only while active.
package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

public final class SprintStatusModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;

    public SprintStatusModule() {
        super("Sprint", 4, 242, 90, 12);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc == null || mc.player == null || context == null || mc.textRenderer == null) {
            return;
        }
        String text = null;
        if (mc.player.isSneaking()) {
            text = "[Sneaking]";
        } else if (mc.player.isSprinting()) {
            text = "[Sprinting]";
        }
        if (text != null) {
            context.drawText(mc.textRenderer, text, x, y, TEXT_COLOR, true);
        }
    }
}
