package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import gg.cinder.client.util.CpsCounter;
import net.minecraft.client.gui.DrawContext;

/**
 * CPS overlay: draws {@code "LMB 8.2 | RMB 0.0"}.
 *
 * <p>ORIGINAL fair-play code: passive display only, never synthesizes clicks.</p>
 *
 * <p><b>Edge-detection choice (documented per contract):</b> click rising edges
 * are polled inside {@link #render} via {@code mc.options.attackKey} /
 * {@code mc.options.useKey} {@code isPressed()}, instead of registering a
 * separate {@code ClientTickEvents} listener. Rationale: (1) this repo is built
 * by several agents in parallel and extra global tick listeners risk
 * double-registration or ordering conflicts; (2) {@code render} already runs
 * every frame, so frame-granularity edge detection cannot miss a held-click
 * transition any worse than a tick listener would for display purposes;
 * (3) no event plumbing keeps the module self-contained and trivially
 * removable. Limitation: a press+release within a single frame (faster than
 * the frame rate) is not counted -- acceptable for a display-only meter.</p>
 *
 * <p>No per-frame allocations: counters, previous-state flags and one reused
 * {@link StringBuilder} are fields; {@code render} performs no {@code new},
 * no {@code String.format}, no boxing. (One {@code String} from
 * {@code StringBuilder#toString()} per frame is unavoidable -- the text API
 * takes a {@code String}.)</p>
 */
public final class CpsModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int SHADOW = 1; // placeholder to keep field layout explicit

    private final CpsCounter left = new CpsCounter();
    private final CpsCounter right = new CpsCounter();
    private final StringBuilder text = new StringBuilder(32);

    private boolean prevLeftPressed;
    private boolean prevRightPressed;

    public CpsModule() {
        super("CPS", 4, 4, 120, 12);
    }

    @Override
    public void render(DrawContext context, float tickDelta) {
        if (!enabled) {
            return;
        }
        if (mc.player == null || mc.world == null) {
            return;
        }

        boolean leftPressed = mc.options.attackKey.isPressed();
        if (leftPressed && !prevLeftPressed) {
            left.recordClick();
        }
        prevLeftPressed = leftPressed;

        boolean rightPressed = mc.options.useKey.isPressed();
        if (rightPressed && !prevRightPressed) {
            right.recordClick();
        }
        prevRightPressed = rightPressed;

        text.setLength(0);
        text.append("LMB ");
        appendOneDecimal(text, left.getCps());
        text.append(" | RMB ");
        appendOneDecimal(text, right.getCps());

        context.drawText(mc.textRenderer, text.toString(), x, y, TEXT_COLOR, true);
    }

    /**
     * Appends a non-negative value with exactly one decimal digit, using only
     * integer math (no {@code String.format}, no floating-point formatting).
     */
    private static void appendOneDecimal(StringBuilder out, double value) {
        if (value < 0.0) {
            value = 0.0;
        }
        long tenths = (long) (value * 10.0);
        out.append(tenths / 10L);
        out.append((char) 46);
        out.append((char) (48 + (tenths % 10L)));
    }
}
