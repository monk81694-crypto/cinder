// Cinder client mod -- original code.
// Active potion effects with amplifier and seconds left (max 6 rows).
package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.text.Text;

public final class PotionModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int MAX_ROWS = 6;

    private final String[] cached = new String[MAX_ROWS];
    private int cachedCount;
    private long lastRebuild;

    public PotionModule() {
        super("Potions", 4, 228, 140, 12);
        for (int i = 0; i < MAX_ROWS; i++) {
            cached[i] = "";
        }
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc == null || mc.player == null || context == null || mc.textRenderer == null) {
            return;
        }
        long now = System.currentTimeMillis();
        if (now - lastRebuild > 500) {
            rebuild();
            lastRebuild = now;
        }
        for (int i = 0; i < cachedCount; i++) {
            context.drawText(mc.textRenderer, cached[i], x, y + i * 11, TEXT_COLOR, true);
        }
    }

    private void rebuild() {
        cachedCount = 0;
        for (StatusEffectInstance inst : mc.player.getActiveStatusEffects().values()) {
            if (cachedCount >= MAX_ROWS) {
                break;
            }
            String name;
            try {
                name = Text.translatable(inst.getTranslationKey()).getString();
            } catch (RuntimeException e) {
                name = "effect";
            }
            int seconds = Math.max(0, inst.getDuration() / 20);
            cached[cachedCount] = name + " " + (inst.getAmplifier() + 1) + " (" + seconds + "s)";
            cachedCount++;
        }
    }
}
