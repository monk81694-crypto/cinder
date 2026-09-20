package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;

/**
 * Keystrokes overlay: W/A/S/D + Space + LMB/RMB boxes, highlighted while held.
 *
 * <p>ORIGINAL fair-play code: reads {@code mc.options.*Key.isPressed()} and
 * draws boxes only. No input injection, no macros.</p>
 *
 * <p>Fixed grid layout (offsets from {@code x,y} are precomputed once in the
 * constructor, so {@code render} does no arithmetic allocation):</p>
 * <pre>
 *        [ W ]
 *   [ A ][ S ][ D ]
 *   [    Space    ]
 *   [ LMB ][ RMB ]
 * </pre>
 *
 * <p>No per-frame allocations: all rects are final int fields, all labels are
 * static constants, no {@code String.format}, no iterators.</p>
 */
public final class KeystrokesModule extends HudModule {
    private static final int BOX = 18;
    private static final int GAP = 2;
    private static final int WIDE_H = 12;

    private static final int IDLE_BG = 0x80000000;
    private static final int PRESSED_BG = 0xCCFFFFFF;
    private static final int IDLE_FG = 0xFFFFFFFF;
    private static final int PRESSED_FG = 0xFF000000;

    private static final String L_W = "W";
    private static final String L_A = "A";
    private static final String L_S = "S";
    private static final String L_D = "D";
    private static final String L_SPACE = "Space";
    private static final String L_LMB = "LMB";
    private static final String L_RMB = "RMB";

    // Precomputed rects, relative to (x, y). w/h are constant per box.
    private final int wX;
    private final int wY;
    private final int aX;
    private final int aY;
    private final int sX;
    private final int sY;
    private final int dX;
    private final int dY;
    private final int spaceX;
    private final int spaceY;
    private final int spaceW;
    private final int lmbX;
    private final int lmbY;
    private final int lmbW;
    private final int rmbX;
    private final int rmbY;
    private final int rmbW;

    public KeystrokesModule() {
        super("Keystrokes", 4, 20, 58, 66);
        int step = BOX + GAP;
        wX = step;
        wY = 0;
        aX = 0;
        aY = step;
        sX = step;
        sY = step;
        dX = step * 2;
        dY = step;
        spaceX = 0;
        spaceY = step * 2;
        spaceW = step * 2 + BOX;
        lmbX = 0;
        lmbY = step * 2 + WIDE_H + GAP;
        lmbW = (spaceW - GAP) / 2;
        rmbX = lmbW + GAP;
        rmbY = lmbY;
        rmbW = spaceW - lmbW - GAP;
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc.player == null || mc.world == null) {
            return;
        }

        drawKey(context, wX, wY, BOX, BOX, L_W, mc.options.forwardKey.isPressed());
        drawKey(context, aX, aY, BOX, BOX, L_A, mc.options.leftKey.isPressed());
        drawKey(context, sX, sY, BOX, BOX, L_S, mc.options.backKey.isPressed());
        drawKey(context, dX, dY, BOX, BOX, L_D, mc.options.rightKey.isPressed());
        drawKey(context, spaceX, spaceY, spaceW, WIDE_H, L_SPACE, mc.options.jumpKey.isPressed());
        drawKey(context, lmbX, lmbY, lmbW, WIDE_H, L_LMB, mc.options.attackKey.isPressed());
        drawKey(context, rmbX, rmbY, rmbW, WIDE_H, L_RMB, mc.options.useKey.isPressed());
    }

    /**
     * Draws one key box. Label centering uses cached string widths only;
     * no objects are created.
     */
    private void drawKey(DrawContext context, int relX, int relY,
            int boxW, int boxH, String label, boolean pressed) {
        int x0 = x + relX;
        int y0 = y + relY;
        int bg = pressed ? PRESSED_BG : IDLE_BG;
        int fg = pressed ? PRESSED_FG : IDLE_FG;
        context.fill(x0, y0, x0 + boxW, y0 + boxH, bg);
        int tw = mc.textRenderer.getWidth(label);
        int tx = x0 + (boxW - tw) / 2;
        int ty = y0 + (boxH - 8) / 2;
        context.drawText(mc.textRenderer, label, tx, ty, fg, false);
    }
}
