package gg.cinder.client.hud;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Tests for the contracted HUD profile layout math (editor grid-snap function).
 *
 * <p><b>Status: GAP -- no snap function exists in real code.</b> Verified
 * 2026-09-20: {@code client/src/main} contains {@code hud/HudModule.java} plus
 * four modules under {@code hud/modules/}, and none of them exposes a grid-snap
 * method. The only grid-like logic in the tree is the precomputed
 * {@code KeystrokesModule} key-box offsets, which are instance fields computed
 * in a constructor that requires live Minecraft classes -- not reachable from
 * the pure-Java (+ Gson) test classpath.</p>
 *
 * <p>Per the task contract this file contains <b>no mirror logic</b>: there is
 * deliberately no local copy of a snap implementation to test against. It holds
 * only (1) a reflection probe that auto-detects a real snap function if/when
 * the HUD agent adds one and checks the weakest universal snap invariants, and
 * (2) a probe documenting why {@code HudModule} positioning itself cannot be
 * covered here. Both abort (SKIP) until testable real code exists, keeping the
 * suite green.</p>
 *
 * <p>Contracted invariants asserted once a real snap exists (hold for
 * round/floor/ceil-to-grid alike, so they cannot over-constrain the HUD
 * agent): for grid &gt; 0 the result is a multiple of the grid, snapping is
 * idempotent, and already-aligned values are unchanged.</p>
 */
@DisplayName("HUD profile layout math (grid snap)")
class HudLayoutSnapTest {

    /** Holder classes the HUD agent might place a snap function in. */
    private static final String[] SNAP_HOLDER_CANDIDATES = {
        "gg.cinder.client.hud.HudLayout",
        "gg.cinder.client.hud.HudEditor",
        "gg.cinder.client.hud.HudManager",
        "gg.cinder.client.hud.HudGrid",
        "gg.cinder.client.hud.HudModule",
    };

    private static Method findSnapMethod() {
        for (String className : SNAP_HOLDER_CANDIDATES) {
            Class<?> holder;
            try {
                holder = Class.forName(className);
            } catch (Throwable ignored) {
                continue;
            }
            Method[] methods;
            try {
                methods = holder.getDeclaredMethods();
            } catch (Throwable ignored) {
                continue;
            }
            for (Method method : methods) {
                if (!Modifier.isStatic(method.getModifiers())) {
                    continue;
                }
                if (!method.getName().toLowerCase().contains("snap")) {
                    continue;
                }
                return method;
            }
        }
        return null;
    }

    @Test
    @DisplayName("grid snap matches contracted invariants (SKIP if absent)")
    void gridSnapMatchesContractIfPresent() throws Exception {
        Method snap = findSnapMethod();
        Assumptions.assumeTrue(snap != null,
                "SKIP (gap): no grid-snap function found in real code -- the HUD agent "
                + "has not added one yet. This test intentionally contains no local "
                + "mirror implementation to fall back on.");

        Class<?>[] params = snap.getParameterTypes();
        Assumptions.assumeTrue(snap.getReturnType() == int.class
                && params.length == 2
                && params[0] == int.class
                && params[1] == int.class,
                "SKIP: real snap method " + snap + " has an unrecognized shape -- "
                + "this probe covers static int snap(int value, int grid); extend it "
                + "instead of adding production code.");
        try {
            snap.setAccessible(true);
        } catch (Throwable ignored) {
            // Continue; public methods do not need it.
        }

        int grid = 4;
        int snapped = (int) snap.invoke(null, 7, grid);
        assertTrue(snapped % grid == 0,
                "snap(7, 4) must land on the grid, was " + snapped);
        assertEquals(snapped, (int) snap.invoke(null, snapped, grid),
                "snap must be idempotent");
        assertEquals(8, (int) snap.invoke(null, 8, grid),
                "already-aligned values must be unchanged");
        assertEquals(0, (int) snap.invoke(null, 0, grid),
                "origin must be unchanged");
    }

    @Test
    @DisplayName("HudModule positioning coverage documents its MC-stub requirement (SKIP)")
    void hudModulePositioningRequiresMcStubs() {
        Class<?> hudModule;
        try {
            hudModule = Class.forName("gg.cinder.client.hud.HudModule");
        } catch (Throwable t) {
            Assumptions.abort("SKIP (gap): HudModule is not loadable on the pure-Java "
                    + "test classpath (" + t.getClass().getSimpleName() + "); position "
                    + "round-trip tests belong in an MC-stubbed suite.");
            return;
        }
        boolean hasPureStaticLayoutHelper = false;
        try {
            for (Method method : hudModule.getDeclaredMethods()) {
                if (!Modifier.isStatic(method.getModifiers())) {
                    continue;
                }
                String name = method.getName().toLowerCase();
                if (name.contains("snap") || name.contains("grid") || name.contains("clamp")) {
                    hasPureStaticLayoutHelper = true;
                }
            }
        } catch (Throwable t) {
            Assumptions.abort("SKIP (gap): HudModule methods not introspectable here ("
                    + t.getClass().getSimpleName() + ").");
            return;
        }
        Assumptions.abort(hasPureStaticLayoutHelper
                ? "HudModule gained a static layout helper -- extend gridSnapMatchesContractIfPresent "
                + "to cover it; not inventing expectations here."
                : "SKIP (gap): HudModule exposes no static pure-Java layout math "
                + "(only instance x/y/w/h behind a MinecraftClient constructor), so there "
                + "is no real code to assert on this classpath. No mirror logic added.");
    }
}
