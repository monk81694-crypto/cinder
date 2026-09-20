package gg.cinder.client.perf;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.InputStream;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Tests for the contracted perf version matching: given a Modrinth
 * version-list JSON payload, select the correct file for MC 26.2 + Fabric.
 *
 * <p><b>Status: PARTLY WRITTEN-TO-CONTRACT. Matcher production code is
 * missing.</b> Verified 2026-09-20: {@code client/src/main} contains no
 * {@code perf} package and no Modrinth matcher class. Per the task contract,
 * the matcher itself is <b>not</b> re-implemented here (no production code is
 * invented); the reflection probe below aborts (SKIP) until the real matcher
 * lands.</p>
 *
 * <p>What <i>does</i> run now is fixture validation, using only Gson: the
 * embedded {@code modrinth-versions.json} resource parses, contains exactly one
 * version supporting MC 26.2 + Fabric, that version carries a primary file with
 * the expected URL, and the distractor entries (wrong MC, wrong loader) are
 * present so a future matcher is actually discriminating. These checks validate
 * the <i>fixture</i>, not a matcher -- the file the real matcher will be
 * tested against.</p>
 *
 * <p>Determinism: fixture is read from test resources, no network, no sleeps,
 * no randomness. Expected selection: version {@code v-c3}, primary file
 * {@code perf-1.1.0-mc26.2-fabric.jar}.</p>
 */
@DisplayName("perf version matching (MC 26.2 + Fabric)")
class ModrinthVersionMatcherTest {

    static final String TARGET_MC = "26.2";
    static final String TARGET_LOADER = "fabric";

    static final String EXPECTED_VERSION_ID = "v-c3";
    static final String EXPECTED_PRIMARY_URL =
            "https://cdn.modrinth.com/data/cinder-perf/versions/v-c3/perf-1.1.0-mc26.2-fabric.jar";

    private static String fixtureText() throws Exception {
        try (InputStream in =
                ModrinthVersionMatcherTest.class.getResourceAsStream("/modrinth-versions.json")) {
            assertNotNull(in, "test fixture /modrinth-versions.json is missing from test resources");
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    private static JsonArray fixtureVersions() throws Exception {
        JsonElement parsed = JsonParser.parseString(fixtureText());
        assertTrue(parsed.isJsonArray(), "fixture root must be a JSON array (Modrinth v2 version list)");
        return parsed.getAsJsonArray();
    }

    private static boolean supports(JsonObject version, String mc, String loader) {
        return containsString(version.getAsJsonArray("game_versions"), mc)
                && containsString(version.getAsJsonArray("loaders"), loader);
    }

    private static boolean containsString(JsonArray array, String value) {
        if (array == null) {
            return false;
        }
        for (JsonElement element : array) {
            if (element.isJsonPrimitive() && value.equals(element.getAsString())) {
                return true;
            }
        }
        return false;
    }

    private static String primaryUrl(JsonObject version) {
        JsonArray files = version.getAsJsonArray("files");
        assertNotNull(files, "version " + version.get("id") + " must list files");
        for (JsonElement file : files) {
            JsonObject entry = file.getAsJsonObject();
            if (entry.has("primary") && entry.get("primary").getAsBoolean()) {
                return entry.get("url").getAsString();
            }
        }
        // No explicit primary flag: fall back to the single file when there is one.
        if (files.size() == 1) {
            return files.get(0).getAsJsonObject().get("url").getAsString();
        }
        throw new AssertionError("version " + version.get("id") + " has no primary file");
    }

    // ------------------------------------------------------------------
    // Fixture validation (runs now, Gson only).
    // ------------------------------------------------------------------

    @Test
    @DisplayName("fixture parses as a non-empty Modrinth version list")
    void fixtureParsesAsVersionList() throws Exception {
        JsonArray versions = fixtureVersions();
        assertTrue(versions.size() >= 2, "fixture must carry distractors, not just the match");
        for (JsonElement element : versions) {
            JsonObject version = element.getAsJsonObject();
            assertTrue(version.has("id"), "each version needs an id");
            assertTrue(version.has("game_versions"), "each version needs game_versions");
            assertTrue(version.has("loaders"), "each version needs loaders");
            assertTrue(version.has("files"), "each version needs files");
        }
    }

    @Test
    @DisplayName("exactly one fixture version supports MC 26.2 + Fabric")
    void exactlyOneVersionSupportsTarget() throws Exception {
        int matches = 0;
        String matchedId = null;
        for (JsonElement element : fixtureVersions()) {
            JsonObject version = element.getAsJsonObject();
            if (supports(version, TARGET_MC, TARGET_LOADER)) {
                matches++;
                matchedId = version.get("id").getAsString();
            }
        }
        assertEquals(1, matches, "fixture must isolate a single 26.2+Fabric candidate");
        assertEquals(EXPECTED_VERSION_ID, matchedId);
    }

    @Test
    @DisplayName("target version primary file has the expected URL")
    void targetVersionPrimaryFileHasExpectedUrl() throws Exception {
        JsonObject target = null;
        for (JsonElement element : fixtureVersions()) {
            JsonObject version = element.getAsJsonObject();
            if (EXPECTED_VERSION_ID.equals(version.get("id").getAsString())) {
                target = version;
            }
        }
        assertNotNull(target, "fixture must contain version " + EXPECTED_VERSION_ID);
        assertTrue(supports(target, TARGET_MC, TARGET_LOADER));
        assertEquals(EXPECTED_PRIMARY_URL, primaryUrl(target));
    }

    @Test
    @DisplayName("distractors cover wrong-MC and wrong-loader cases")
    void distractorsCoverWrongMcAndWrongLoader() throws Exception {
        boolean wrongMcPresent = false;
        boolean wrongLoaderPresent = false;
        for (JsonElement element : fixtureVersions()) {
            JsonObject version = element.getAsJsonObject();
            if (EXPECTED_VERSION_ID.equals(version.get("id").getAsString())) {
                continue;
            }
            if (!containsString(version.getAsJsonArray("game_versions"), TARGET_MC)) {
                wrongMcPresent = true;
            }
            if (containsString(version.getAsJsonArray("game_versions"), TARGET_MC)
                    && !containsString(version.getAsJsonArray("loaders"), TARGET_LOADER)) {
                wrongLoaderPresent = true;
            }
        }
        assertTrue(wrongMcPresent, "fixture needs a wrong-MC distractor");
        assertTrue(wrongLoaderPresent, "fixture needs a same-MC wrong-loader distractor");
    }

    // ------------------------------------------------------------------
    // Real-code matcher probe (aborts until the matcher lands).
    // ------------------------------------------------------------------

    private static final String[] MATCHER_CLASS_CANDIDATES = {
        "gg.cinder.client.perf.ModrinthVersionMatcher",
        "gg.cinder.client.perf.VersionMatcher",
        "gg.cinder.client.perf.ModrinthResolver",
    };

    private static final String[] MATCHER_METHOD_NAMES = {
        "select", "selectFile", "resolve", "resolveFile",
        "pickBest", "findBest", "matchVersion", "match",
    };

    private static String normalizeSelection(Object result) {
        if (result == null) {
            return null;
        }
        if (result instanceof java.util.Optional) {
            java.util.Optional<?> optional = (java.util.Optional<?>) result;
            return optional.map(ModrinthVersionMatcherTest::normalizeSelection).orElse(null);
        }
        if (result instanceof java.net.URL || result instanceof java.net.URI) {
            return result.toString();
        }
        if (result instanceof JsonObject) {
            JsonObject file = (JsonObject) result;
            if (file.has("url")) {
                return file.get("url").getAsString();
            }
            return file.toString();
        }
        if (result instanceof JsonElement) {
            JsonElement element = (JsonElement) result;
            if (element.isJsonObject() && element.getAsJsonObject().has("url")) {
                return element.getAsJsonObject().get("url").getAsString();
            }
            if (element.isJsonPrimitive()) {
                return element.getAsString();
            }
            return element.toString();
        }
        return result.toString();
    }

    @Test
    @DisplayName("REAL matcher selects the 26.2+Fabric primary file (SKIP if absent)")
    void realMatcherSelectsTargetFileIfPresent() throws Exception {
        Class<?> matcher = null;
        for (String name : MATCHER_CLASS_CANDIDATES) {
            try {
                matcher = Class.forName(name);
                break;
            } catch (Throwable ignored) {
                // Absent or unloadable on this classpath -- try the next name.
            }
        }
        Assumptions.assumeTrue(matcher != null,
                "SKIP (unwritten): no Modrinth matcher class on the test classpath yet; "
                + "only fixture-validation tests above run. No production code invented here.");

        String json = fixtureText();
        JsonArray versions = JsonParser.parseString(json).getAsJsonArray();
        String selected = null;
        boolean invoked = false;
        for (Method method : matcher.getDeclaredMethods()) {
            if (!Modifier.isStatic(method.getModifiers())) {
                continue;
            }
            boolean nameOk = false;
            for (String candidate : MATCHER_METHOD_NAMES) {
                if (method.getName().equals(candidate)) {
                    nameOk = true;
                    break;
                }
            }
            if (!nameOk) {
                continue;
            }
            Class<?>[] params = method.getParameterTypes();
            Object[] args = null;
            if (params.length == 3
                    && params[0] == String.class
                    && params[1] == String.class
                    && params[2] == String.class) {
                args = new Object[] {json, TARGET_MC, TARGET_LOADER};
            } else if (params.length == 3
                    && JsonArray.class.isAssignableFrom(params[0])
                    && params[1] == String.class
                    && params[2] == String.class) {
                args = new Object[] {versions, TARGET_MC, TARGET_LOADER};
            } else if (params.length == 3
                    && JsonElement.class.isAssignableFrom(params[0])
                    && params[1] == String.class
                    && params[2] == String.class) {
                args = new Object[] {versions, TARGET_MC, TARGET_LOADER};
            }
            if (args == null) {
                continue;
            }
            try {
                try {
                    method.setAccessible(true);
                } catch (Throwable ignored) {
                    // Continue; public methods do not need it.
                }
                selected = normalizeSelection(method.invoke(null, args));
                invoked = true;
                break;
            } catch (java.lang.reflect.InvocationTargetException e) {
                throw new AssertionError("real matcher threw: " + e.getCause(), e.getCause());
            }
        }
        Assumptions.assumeTrue(invoked,
                "SKIP: " + matcher.getName() + " exists but exposes no recognized "
                + "(payload, mcVersion, loader) selection method -- extend "
                + "MATCHER_METHOD_NAMES instead of adding production code.");
        assertEquals(EXPECTED_PRIMARY_URL, selected,
                "real matcher must select the 26.2+Fabric primary file");
    }
}
