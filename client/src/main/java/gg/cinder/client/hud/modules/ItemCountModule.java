package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.item.ItemStack;

/**
 * Held-item overlay: main-hand item name plus stack count, and the offhand
 * line when something is held there.
 *
 * <p>Example: {@code "Oak Planks x64"}. ORIGINAL fair-play code: reads
 * {@code getMainHandStack()} / {@code getOffHandStack()} and draws text only;
 * inventory contents are never touched.</p>
 *
 * <p>No per-frame allocations except the unavoidable text {@code String}s:
 * one reused {@link StringBuilder} builds each line, no {@code String.format},
 * no iterators. Height stays fixed (two rows); the second row is simply left
 * blank when the offhand is empty.</p>
 */
public final class ItemCountModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int LINE_H = 10;
    private static final String EMPTY_HAND = "Empty hand";

    private final StringBuilder line = new StringBuilder(64);

    public ItemCountModule() {
        super("ItemCount", 4, 164, 140, LINE_H * 2);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc.player == null || mc.world == null) {
            return;
        }

        ItemStack main = mc.player.getMainHandStack();
        line.setLength(0);
        if (main == null || main.isEmpty()) {
            line.append(EMPTY_HAND);
        } else {
            line.append(main.getItem().getName().getString());
            line.append(" x");
            line.append(main.getCount());
        }
        context.drawText(mc.textRenderer, line.toString(), x, y, TEXT_COLOR, true);

        ItemStack off = mc.player.getOffHandStack();
        if (off != null && !off.isEmpty()) {
            line.setLength(0);
            line.append(off.getItem().getName().getString());
            line.append(" x");
            line.append(off.getCount());
            context.drawText(mc.textRenderer, line.toString(), x, y + LINE_H, TEXT_COLOR, true);
        }
    }
}
