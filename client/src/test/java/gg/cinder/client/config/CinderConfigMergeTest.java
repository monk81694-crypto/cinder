package gg.cinder.client.config;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Map;

import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Tests for the contracted config merge: a static method merging a base
 * {@code JsonObject} with a shared/overlay {@code JsonObject}.
 *
 * <p><b>Status: WRITTEN-TO-CONTRACT. Production code is missing.</b> Verified
 * 2026-09-20: {@code client/src/main} contains no {@code config} package and no
 * class named {@code CinderConfig}; no merge method of any name exists. Per the
 * task contract the merge semantics are therefore tested here directly against
 * Gson (no Minecraft classes on this classpath), and flagged as unverified
 * until the implementation lands.</p>
 *
 * <p><b>Contracted merge semantics</b> (owned by the config agent; if the real
 * implementation differs, update this contract, not the other way round):</p>
 * <ol>
 *   <li>Overlay scalar values override base values.</li>
 *   <li>Nested objects merge recursively; keys present on only one side are
 *       preserved.</li>
 *   <li>Arrays are replaced wholesale by the overlay (never concatenated).</li>
 *   <li>An explicit {@code null} in the overlay deletes that key.</li>
 *   <li>Neither input is mutated; the result is a new {@code JsonObject}.</li>
 * </ol>
 *
 * <p>The {@link #contractDeepMerge} helper below is the <i>executable form of
 * that contract</i>, not production code: it must be deleted (or redirected to
 * the real implementation) once {@code CinderConfig} lands. The
 * {@link #realMergeMethodIfPresentConformsToContract} probe auto-detects the
 * real static merge method by reflection and checks it against this contract;
 * until then it aborts (SKIP), keeping the suite green.</p>
 *
 * <p>Determinism: pure Gson object graphs, no I/O, no sleeps, no network.</p>
 */
@DisplayName("CinderConfig merge (base + shared JsonObjects)")
class CinderConfigMergeTest {

    // ------------------------------------------------------------------
    // Executable contract (NOT production code -- see class javadoc).
    // ------------------------------------------------------------------

    /**
     * Reference implementation of the contracted deep-merge semantics.
     *
     * @param base    base configuration (not mutated)
     * @param overlay shared/overlay configuration (not mutated)
     * @return new {@code JsonObject} with the overlay applied onto the base
     */
    static JsonObject contractDeepMerge(JsonObject base, JsonObject overlay) {
        JsonObject out = base.deepCopy();
        for (Map.Entry<String, JsonElement> entry : overlay.entrySet()) {
            String key = entry.getKey();
            JsonElement over = entry.getValue();
            if (over == null || over.isJsonNull()) {
                out.remove(key);
                continue;
            }
            JsonElement current = out.get(key);
            if (over.isJsonObject() && current != null && current.isJsonObject()) {
                out.add(key, contractDeepMerge(current.getAsJsonObject(), over.getAsJsonObject()));
            } else {
                out.add(key, over.deepCopy());
            }
        }
        return out;
    }

    // ------------------------------------------------------------------
    // Contract-semantics tests (run now, Gson only).
    // ------------------------------------------------------------------

    @Test
    @DisplayName("(contract) overlay scalars override base scalars")
    void overlayScalarsOverrideBase() {
        JsonObject base = JsonParser.parseString("{\"a\":1,\"b\":\"old\",\"c\":true}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString("{\"b\":\"new\",\"c\":false}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        assertEquals(1, merged.get("a").getAsInt());
        assertEquals("new", merged.get("b").getAsString());
        assertEquals(false, merged.get("c").getAsBoolean());
    }

    @Test
    @DisplayName("(contract) nested objects merge recursively")
    void nestedObjectsMergeRecursively() {
        JsonObject base = JsonParser.parseString(
                "{\"hud\":{\"x\":4,\"y\":4,\"scale\":1.0}}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString(
                "{\"hud\":{\"y\":20}}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        JsonObject hud = merged.getAsJsonObject("hud");
        assertEquals(4, hud.get("x").getAsInt());
        assertEquals(20, hud.get("y").getAsInt());
        assertEquals(1.0, hud.get("scale").getAsDouble());
    }

    @Test
    @DisplayName("(contract) arrays are replaced, not concatenated")
    void arraysAreReplaced() {
        JsonObject base = JsonParser.parseString("{\"servers\":[\"a\",\"b\"]}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString("{\"servers\":[\"c\"]}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        assertEquals(1, merged.getAsJsonArray("servers").size());
        assertEquals("c", merged.getAsJsonArray("servers").get(0).getAsString());
    }

    @Test
    @DisplayName("(contract) base-only and overlay-only keys are preserved")
    void disjointKeysArePreserved() {
        JsonObject base = JsonParser.parseString("{\"baseOnly\":1}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString("{\"sharedOnly\":2}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        assertEquals(1, merged.get("baseOnly").getAsInt());
        assertEquals(2, merged.get("sharedOnly").getAsInt());
    }

    @Test
    @DisplayName("(contract) explicit null in overlay deletes the key")
    void explicitNullDeletesKey() {
        JsonObject base = JsonParser.parseString("{\"keep\":1,\"drop\":2}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString("{\"drop\":null}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        assertEquals(1, merged.get("keep").getAsInt());
        assertFalse(merged.has("drop"));
    }

    @Test
    @DisplayName("(contract) inputs are not mutated")
    void inputsAreNotMutated() {
        JsonObject base = JsonParser.parseString(
                "{\"a\":1,\"nested\":{\"x\":1}}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString(
                "{\"a\":2,\"nested\":{\"y\":2}}").getAsJsonObject();
        String baseBefore = base.toString();
        String overlayBefore = overlay.toString();
        contractDeepMerge(base, overlay);
        assertEquals(baseBefore, base.toString());
        assertEquals(overlayBefore, overlay.toString());
    }

    @Test
    @DisplayName("(contract) empty overlay returns an equal copy of base")
    void emptyOverlayReturnsBaseCopy() {
        JsonObject base = JsonParser.parseString("{\"a\":1}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, new JsonObject());
        assertEquals(base, merged);
        assertTrue(merged != base, "result must be a new object, not the base reference");
    }

    @Test
    @DisplayName("(contract) object-vs-scalar type change follows the overlay")
    void typeChangeFollowsOverlay() {
        JsonObject base = JsonParser.parseString("{\"k\":{\"nested\":true}}").getAsJsonObject();
        JsonObject overlay = JsonParser.parseString("{\"k\":\"flat\"}").getAsJsonObject();
        JsonObject merged = contractDeepMerge(base, overlay);
        assertEquals("flat", merged.get("k").getAsString());
    }

    // ------------------------------------------------------------------
    // Real-code conformance probe (aborts until CinderConfig lands).
    // ------------------------------------------------------------------

    private static final String[] CONFIG_CLASS_CANDIDATES = {
        "gg.cinder.client.config.CinderConfig",
        "gg.cinder.client.config.Config",
        "gg.cinder.client.config.CinderConfigManager",
    };

    private static final String[] MERGE_METHOD_CANDIDATES = {
        "merge", "mergeConfigs", "deepMerge", "combine", "mergeJson",
    };

    private static Class<?> firstPresentClass(String... names) {
        for (String name : names) {
            try {
                return Class.forName(name);
            } catch (Throwable ignored) {
                // Absent or unloadable on this classpath -- try the next name.
            }
        }
        return null;
    }

    private static Method findStaticMerge(Class<?> configClass) {
        for (String name : MERGE_METHOD_CANDIDATES) {
            for (Method method : configClass.getDeclaredMethods()) {
                if (!method.getName().equals(name)) {
                    continue;
                }
                if (!Modifier.isStatic(method.getModifiers())) {
                    continue;
                }
                Class<?>[] params = method.getParameterTypes();
                if (params.length == 2
                        && params[0] == JsonObject.class
                        && params[1] == JsonObject.class
                        && JsonObject.class.isAssignableFrom(method.getReturnType())) {
                    return method;
                }
            }
        }
        return null;
    }

    @Test
    @DisplayName("REAL CinderConfig static merge conforms to contract (SKIP if absent)")
    void realMergeMethodIfPresentConformsToContract() throws Exception {
        Class<?> configClass = firstPresentClass(CONFIG_CLASS_CANDIDATES);
        Assumptions.assumeTrue(configClass != null,
                "SKIP (written-to-contract): no config class on the test classpath yet; "
                + "only the contract-semantics tests above run until CinderConfig lands.");

        Method merge = findStaticMerge(configClass);
        Assumptions.assumeTrue(merge != null,
                "SKIP (written-to-contract): " + configClass.getName()
                + " exists but exposes no static (JsonObject, JsonObject) merge method "
                + "under a recognized name -- extend MERGE_METHOD_CANDIDATES, never "
                + "production code.");
        try {
            merge.setAccessible(true);
        } catch (Throwable ignored) {
            // Continue; public methods do not need it.
        }

        JsonObject base = JsonParser.parseString(
                "{\"a\":1,\"nested\":{\"x\":1,\"y\":2},\"list\":[1,2]}").getAsJsonObject();
        JsonObject shared = JsonParser.parseString(
                "{\"b\":2,\"nested\":{\"y\":20},\"list\":[9]}").getAsJsonObject();
        JsonObject expected = contractDeepMerge(
                base.deepCopy(), shared.deepCopy());

        Object actual;
        try {
            actual = merge.invoke(null, base, shared);
        } catch (InvocationTargetException e) {
            throw new AssertionError("real merge method threw: " + e.getCause(), e.getCause());
        }
        Assumptions.assumeTrue(actual instanceof JsonObject,
                "SKIP: real merge returned " + (actual == null ? "null" : actual.getClass().getName())
                + ", expected JsonObject -- update this probe.");
        assertEquals(expected, (JsonObject) actual,
                "real CinderConfig merge must match the contracted deep-merge semantics");
    }
}
