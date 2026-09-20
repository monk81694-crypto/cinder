package gg.cinder.client.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Unit tests for {@link CpsCounter}.
 *
 * <p><b>Status: VERIFIED-AGAINST-REAL-CODE.</b> {@code CpsCounter} exists at
 * {@code client/src/main/java/gg/cinder/client/util/CpsCounter.java}. It is
 * pure Java (only {@link System#nanoTime()} plus an {@code ArrayDeque}) with no
 * Minecraft imports, so these tests run on the plain JUnit classpath.</p>
 *
 * <p>Contract under test: {@code recordClick()} stamps one click at the current
 * {@code nanoTime}; {@code getCps()} returns the count of stamps inside the
 * trailing 1000 ms window (lazy prune, no background thread).</p>
 *
 * <p>Determinism / flakiness guards:</p>
 * <ul>
 *   <li>No sleep longer than 50 ms anywhere (longest sleep: 25 ms, a 40x safety
 *       margin below the 1000 ms window).</li>
 *   <li>No network, no randomness, no wall-clock assertions.</li>
 *   <li>{@code nanoTime()} is monotonic, so the short "still inside the window"
 *       check cannot flake on wall-clock adjustments.</li>
 * </ul>
 *
 * <p><b>Known gap (could not test):</b> true sliding-window <i>expiry</i> --
 * stamps older than 1000 ms dropping out of the count -- is not covered. It
 * would need either a sleep over 1000 ms (banned: slow and flaky) or a time
 * seam, and {@code CpsCounter} calls {@code System.nanoTime()} directly with
 * no injection point. Recommendation for the owning agent: add an overload such
 * as {@code recordClick(long nanoTime)} (or a {@code LongSupplier} clock) so
 * expiry becomes testable with zero sleeps.</p>
 */
@DisplayName("CpsCounter (sliding 1s window)")
class CpsCounterTest {

    @Test
    @DisplayName("fresh counter reads 0 CPS")
    void freshCounterReadsZero() {
        assertEquals(0.0, new CpsCounter().getCps());
    }

    @Test
    @DisplayName("single click reads 1 CPS")
    void singleClickReadsOne() {
        CpsCounter counter = new CpsCounter();
        counter.recordClick();
        assertEquals(1.0, counter.getCps());
    }

    @Test
    @DisplayName("rapid burst: every click in a tight loop is counted")
    void rapidBurstAllClicksCounted() {
        CpsCounter counter = new CpsCounter();
        int clicks = 25;
        for (int i = 0; i < clicks; i++) {
            counter.recordClick();
        }
        assertEquals((double) clicks, counter.getCps());
    }

    @Test
    @DisplayName("consecutive reads are stable without new clicks (no sleep)")
    void consecutiveReadsAreStable() {
        CpsCounter counter = new CpsCounter();
        for (int i = 0; i < 5; i++) {
            counter.recordClick();
        }
        double first = counter.getCps();
        double second = counter.getCps();
        assertEquals(5.0, first);
        assertEquals(first, second);
    }

    @Test
    @DisplayName("getCps() returns an integral double value")
    void cpsValueIsIntegral() {
        CpsCounter counter = new CpsCounter();
        for (int i = 0; i < 7; i++) {
            counter.recordClick();
        }
        double cps = counter.getCps();
        assertEquals(7.0, cps);
        assertTrue(cps % 1.0 == 0.0, "CPS must be a whole count, was " + cps);
    }

    @Test
    @DisplayName("clicks recorded well inside the window are not pruned early")
    void clicksSurviveWellInsideWindow() throws InterruptedException {
        CpsCounter counter = new CpsCounter();
        for (int i = 0; i < 10; i++) {
            counter.recordClick();
        }
        // 25 ms is far below the 1000 ms window: proves prune() keeps fresh
        // stamps without paying for a full-window wait.
        Thread.sleep(25);
        assertEquals(10.0, counter.getCps());
    }

    @Test
    @DisplayName("high-volume burst (500 clicks) loses nothing")
    void highVolumeBurstLosesNothing() {
        CpsCounter counter = new CpsCounter();
        for (int i = 0; i < 500; i++) {
            counter.recordClick();
        }
        assertEquals(500.0, counter.getCps());
    }
}
