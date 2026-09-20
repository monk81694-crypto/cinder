package gg.cinder.client.util;

import java.util.ArrayDeque;

/**
 * PURE Java click-per-second counter. No Minecraft imports; unit-testable.
 *
 * <p>ORIGINAL code. Records click timestamps with {@link System#nanoTime()} and
 * counts how many fall inside the trailing 1000&nbsp;ms window. Old stamps are
 * pruned lazily on every call, so there is no background thread or timer.</p>
 */
public final class CpsCounter {
    /** Window length in nanoseconds (1000 ms). */
    private static final long WINDOW_NS = 1_000_000_000L;

    /** Click timestamps in insertion order (oldest first). */
    private final ArrayDeque<Long> stamps = new ArrayDeque<>();

    /**
     * Records one click at the current {@link System#nanoTime()}.
     */
    public void recordClick() {
        stamps.addLast(System.nanoTime());
        prune();
    }

    /**
     * Returns the number of recorded clicks within the last 1000&nbsp;ms.
     *
     * @return click count in the trailing one-second window
     */
    public double getCps() {
        prune();
        return stamps.size();
    }

    /**
     * Drops every stamp older than the trailing window.
     */
    private void prune() {
        long cutoff = System.nanoTime() - WINDOW_NS;
        while (!stamps.isEmpty()) {
            Long first = stamps.peekFirst();
            if (first == null || first.longValue() >= cutoff) {
                break;
            }
            stamps.removeFirst();
        }
    }
}
