package gg.cinder.client.hud.modules;

import gg.cinder.client.hud.HudModule;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import net.minecraft.client.gui.DrawContext;

/**
 * Clock overlay: line one is the client-local system clock {@code "HH:MM"},
 * line two is the in-game day count {@code "Day 12"}.
 *
 * <p>ORIGINAL fair-play code: {@code java.time.LocalTime.now()} for the wall
 * clock (client-local zone) plus {@code world.getDayTime() / 24000 + 1}
 * for the day number (1-indexed so a fresh world shows Day 1, not Day 0).
 * Display only.</p>
 *
 * <p>No per-frame allocations in the steady state: the wall-clock string is
 * cached and refreshed at most once per second; one reused pre-sized
 * {@link StringBuilder} builds each line.</p>
 */
public final class ClockModule extends HudModule {
    private static final int TEXT_COLOR = 0xFFFFFFFF;
    private static final int LINE_H = 10;

    private final StringBuilder Text = new StringBuilder(16);
    private final DateTimeFormatter clockFormat = DateTimeFormatter.ofPattern("HH:mm");

    private String cachedClock = "--:--";
    private long lastClockMs = 0L;

    public ClockModule() {
        super("Clock", 4, 58, 80, LINE_H * 2 + 2);
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
        long nowMs = System.currentTimeMillis();
        if (nowMs - lastClockMs >= 1000L) {
            cachedClock = LocalTime.now().format(clockFormat);
            lastClockMs = nowMs;
        }
        Text.setLength(0);
        Text.append(cachedClock);
        context.drawText(mc.textRenderer, Text.toString(), x, y, TEXT_COLOR, true);

        long day = mc.world.getTimeOfDay() / 24000L + 1L;
        Text.setLength(0);
        Text.append("Day ");
        Text.append(day);
        context.drawText(mc.textRenderer, Text.toString(), x, y + LINE_H, TEXT_COLOR, true);
    }
}
