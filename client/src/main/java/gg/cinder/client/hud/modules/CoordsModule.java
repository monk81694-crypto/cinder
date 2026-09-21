package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

/**
 * Coords overlay: line one {@code "XYZ: x / y / z"} (block pos), line two
 * {@code "Facing: N"} / {@code "S"} / {@code "E"} / {@code "W"}.
 *
 * <p>ORIGINAL fair-play code: reads {@code player.getBlockPos()} and
 * {@code player.getYaw()} and draws Text only. Yaw mapping follows vanilla
 * convention (0 = South +Z, 90 = West, 180 = North, 270 = East): S in
 * [315,45), W in [45,135), N in [135,225), E in [225,315).</p>
 *
 * <p>No per-frame allocations except the unavoidable Text {@code String}s:
 * one reused pre-sized {@link StringBuilder} builds both lines in turn,
 * primitive math only.</p>
 */
public final class CoordsModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int LINE_H = 10;

    private final StringBuilder Text = new StringBuilder(48);

    public CoordsModule() {
        super("Coords", 4, 32, 140, LINE_H * 2 + 2);
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
        var pos = mc.player.getBlockPos();
        Text.setLength(0);
        Text.append("XYZ: ");
        Text.append(pos.getX());
        Text.append(" / ");
        Text.append(pos.getY());
        Text.append(" / ");
        Text.append(pos.getZ());
        context.drawText(mc.textRenderer, Text.toString(), x, y, TEXT_COLOR, true);

        float yaw = mc.player.getYaw();
        float n = yaw % 360.0F;
        if (n < 0.0F) {
            n += 360.0F;
        }
        char dir;
        if (n >= 315.0F || n < 45.0F) {
            dir = 'S';
        } else if (n < 135.0F) {
            dir = 'W';
        } else if (n < 225.0F) {
            dir = 'N';
        } else {
            dir = 'E';
        }
        Text.setLength(0);
        Text.append("Facing: ");
        Text.append(dir);
        context.drawText(mc.textRenderer, Text.toString(), x, y + LINE_H, TEXT_COLOR, true);
    }
}
