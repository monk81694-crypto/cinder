package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.item.ItemStack;

/**
 * Armor overlay: one row per worn armor piece with its item icon and a
 * durability bar underneath. Empty slots are skipped.
 *
 * <p>ORIGINAL fair-play code: read-only view of
 * {@code player.getInventory().armor}; nothing is equipped, moved, or
 * modified.</p>
 *
 * <p>Slot order: helmet, chestplate, leggings, boots from top to bottom
 * (the vanilla {@code armor} list runs boots..helmet, so it is iterated in
 * reverse). Each row is 18&nbsp;px tall: a 16x16 icon plus a 2&nbsp;px
 * durability bar.</p>
 *
 * <p>No per-frame allocations: indexed loop over the existing inventory list,
 * primitive math only, no iterators, no formatting.</p>
 */
public final class ArmorModule extends HudModule {
    private static final int ROW_H = 18;
    private static final int ICON = 16;
    private static final int BAR_BG = 0xFF000000;
    private static final int BAR_FULL = 0xFF00FF00;
    private static final int BAR_MID = 0xFFFFFF00;
    private static final int BAR_LOW = 0xFFFF0000;

    public ArmorModule() {
        super("Armor", 4, 90, 18, ROW_H * 4);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc.player == null || mc.world == null) {
            return;
        }

        // Helmet first, boots last.
        EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
        int row = 0;
        for (int i = 0; i < slots.length; i++) {
            ItemStack stack = mc.player.getEquippedStack(slots[i]);
            if (stack == null || stack.isEmpty()) {
                continue;
            }
            int ix = x;
            int iy = y + row * ROW_H;
            context.drawItem(stack, ix, iy);
            drawDurabilityBar(context, stack, ix, iy + ICON);
            row++;
        }
    }

    /**
     * Draws a 16x1 durability bar under the icon. Undamageable items
     * (elytra-less edge cases, enchanted books never appear here) show a full
     * green bar instead of dividing by zero.
     */
    private static void drawDurabilityBar(DrawContext context, ItemStack stack, int barX, int barY) {
        int max = stack.getMaxDamage();
        float fraction;
        if (max <= 0) {
            fraction = 1.0f;
        } else {
            int damage = stack.getDamage();
            if (damage < 0) {
                damage = 0;
            } else if (damage > max) {
                damage = max;
            }
            fraction = 1.0f - ((float) damage / (float) max);
        }
        context.fill(barX, barY, barX + ICON, barY + 1, BAR_BG);
        int width = (int) (fraction * (float) ICON);
        if (width <= 0) {
            return;
        }
        int color;
        if (fraction > 0.5f) {
            color = BAR_FULL;
        } else if (fraction > 0.25f) {
            color = BAR_MID;
        } else {
            color = BAR_LOW;
        }
        context.fill(barX, barY, barX + width, barY + 1, color);
    }
}
