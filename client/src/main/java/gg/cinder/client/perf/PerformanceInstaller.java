// Cinder client mod -- original code.
// One-click performance pack installer. Downloads well-known open-source
// performance mods (Sodium, Lithium, FerriteCore, optional Iris) from the
// Modrinth API into the instance mods folder. Their jars are NEVER bundled in
// this repo — they are fetched at the user's request, respecting each mod's
// own license. An undo manifest removes exactly what was installed.
package gg.cinder.client.perf;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import gg.cinder.client.CinderClient;
import java.io.IOException;
import java.io.InputStream;
import java.io.Reader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;
import net.fabricmc.loader.api.FabricLoader;

public final class PerformanceInstaller {

    /** Modrinth project slugs installed by the required pack. */
    public static final String[] REQUIRED_MODS = {"sodium", "lithium", "ferrite-core"};

    /** Optional addition (shaders support). */
    public static final String OPTIONAL_MOD = "iris";

    /** Target stabilized by the launcher (matches the client build). */
    public static final String TARGET_MC = "1.21.11";

    /** Loader filter for Modrinth queries. */
    public static final String TARGET_LOADER = "fabric";

    private static final String API = "https://api.modrinth.com/v2";
    private static final String USER_AGENT = "CinderClient/1.0 (github:cinder-client)";
    private static final Gson GSON = new Gson();

    private PerformanceInstaller() {
    }

    /** A single downloadable file picked from a Modrinth version list. */
    public static final class FilePick {
        public final String url;
        public final String filename;

        public FilePick(String url, String filename) {
            this.url = url;
            this.filename = filename;
        }
    }

    /**
     * Picks the best file for the given MC version + loader from a Modrinth
     * {@code /version} response array. Prefers release type, then beta.
     * Pure logic (no network) so unit tests can cover it.
     *
     * @return the pick, or null when nothing matches
     */
    public static FilePick select(JsonArray versions, String mcVersion, String loader) {
        if (versions == null) {
            return null;
        }
        FilePick fallback = null;
        for (JsonElement element : versions) {
            if (!element.isJsonObject()) {
                continue;
            }
            JsonObject version = element.getAsJsonObject();
            if (!stringArrayContains(version, "game_versions", mcVersion)) {
                continue;
            }
            if (!stringArrayContains(version, "loaders", loader)) {
                continue;
            }
            FilePick pick = primaryFile(version);
            if (pick == null) {
                continue;
            }
            String type = version.has("version_type") ? version.get("version_type").getAsString() : "";
            if ("release".equalsIgnoreCase(type)) {
                return pick;
            }
            if (fallback == null) {
                fallback = pick;
            }
        }
        return fallback;
    }

    private static boolean stringArrayContains(JsonObject version, String key, String value) {
        if (!version.has(key) || !version.get(key).isJsonArray()) {
            return false;
        }
        for (JsonElement element : version.getAsJsonArray(key)) {
            if (element.isJsonPrimitive() && value.equals(element.getAsString())) {
                return true;
            }
        }
        return false;
    }

    private static FilePick primaryFile(JsonObject version) {
        if (!version.has("files") || !version.get("files").isJsonArray()) {
            return null;
        }
        JsonArray files = version.getAsJsonArray("files");
        JsonObject chosen = null;
        for (JsonElement element : files) {
            if (!element.isJsonObject()) {
                continue;
            }
            JsonObject file = element.getAsJsonObject();
            if (chosen == null) {
                chosen = file;
            }
            if (file.has("primary") && file.get("primary").getAsBoolean()) {
                chosen = file;
                break;
            }
        }
        if (chosen == null || !chosen.has("url") || !chosen.has("filename")) {
            return null;
        }
        String name = chosen.get("filename").getAsString();
        if (!name.endsWith(".jar")) {
            return null;
        }
        return new FilePick(chosen.get("url").getAsString(), new java.io.File(name).getName());
    }

    /**
     * Installs the required pack on a background thread (never blocks the
     * client thread). Skips jars that are already present. Safe to call at
     * startup: with everything installed it only logs and returns.
     */
    public static void install() {
        install(false);
    }

    /** Installs the pack, optionally including Iris. Runs on a worker thread. */
    public static void install(boolean withIris) {
        Thread worker = new Thread(() -> {
            try {
                List<String> added = installBlocking(withIris);
                if (added.isEmpty()) {
                    CinderClient.LOGGER.info("[cinder] Performance pack already installed");
                } else {
                    CinderClient.LOGGER.info("[cinder] Performance pack installed: {}", added);
                }
            } catch (Exception e) {
                CinderClient.LOGGER.error("[cinder] Performance pack install failed", e);
            }
        }, "cinder-perf-install");
        worker.setDaemon(true);
        worker.start();
    }

    private static List<String> installBlocking(boolean withIris) throws IOException {
        List<String> slugs = new ArrayList<>();
        for (String slug : REQUIRED_MODS) {
            slugs.add(slug);
        }
        if (withIris) {
            slugs.add(OPTIONAL_MOD);
        }
        Path modsDir = FabricLoader.getInstance().getGameDir().resolve("mods");
        Files.createDirectories(modsDir);
        List<String> added = new ArrayList<>();
        List<String> manifest = readManifest();
        for (String slug : slugs) {
            JsonArray versions = queryVersions(slug);
            FilePick pick = select(versions, TARGET_MC, TARGET_LOADER);
            if (pick == null) {
                CinderClient.LOGGER.warn("[cinder] No {} file for {} {}", slug, TARGET_MC, TARGET_LOADER);
                continue;
            }
            Path dest = modsDir.resolve(pick.filename);
            if (Files.isRegularFile(dest)) {
                continue;
            }
            download(pick.url, dest);
            added.add(pick.filename);
            manifest.add(pick.filename);
        }
        writeManifest(manifest);
        return added;
    }

    /** Removes exactly the files recorded in the undo manifest. */
    public static List<String> undo() {
        List<String> removed = new ArrayList<>();
        try {
            Path modsDir = FabricLoader.getInstance().getGameDir().resolve("mods");
            for (String name : readManifest()) {
                try {
                    if (Files.deleteIfExists(modsDir.resolve(new java.io.File(name).getName()))) {
                        removed.add(name);
                    }
                } catch (IOException ignored) {
                }
            }
            writeManifest(new ArrayList<>());
        } catch (Exception e) {
            CinderClient.LOGGER.error("[cinder] Performance pack undo failed", e);
        }
        return removed;
    }

    private static Path manifestPath() {
        return FabricLoader.getInstance().getConfigDir().resolve("cinder-perf-undo.json");
    }

    private static List<String> readManifest() {
        try {
            Path file = manifestPath();
            if (!Files.isRegularFile(file)) {
                return new ArrayList<>();
            }
            try (Reader reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) {
                JsonArray array = JsonParser.parseReader(reader).getAsJsonArray();
                List<String> out = new ArrayList<>();
                for (JsonElement element : array) {
                    if (element.isJsonPrimitive()) {
                        out.add(element.getAsString());
                    }
                }
                return out;
            }
        } catch (IOException | RuntimeException e) {
            return new ArrayList<>();
        }
    }

    private static void writeManifest(List<String> files) {
        try {
            Path file = manifestPath();
            Files.createDirectories(file.getParent());
            Files.write(file, GSON.toJson(files).getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) {
            CinderClient.LOGGER.error("[cinder] Could not write perf undo manifest", e);
        }
    }

    private static JsonArray queryVersions(String slug) throws IOException {
        String query = API + "/project/" + slug + "/version?game_versions=[\""
                + TARGET_MC + "\"]&loaders=[\"" + TARGET_LOADER + "\"]";
        HttpURLConnection connection = (HttpURLConnection) new URL(query).openConnection();
        connection.setRequestProperty("User-Agent", USER_AGENT);
        connection.setConnectTimeout(10000);
        connection.setReadTimeout(15000);
        try (InputStream in = connection.getInputStream();
                Reader reader = new java.io.InputStreamReader(in, StandardCharsets.UTF_8)) {
            JsonElement parsed = JsonParser.parseReader(reader);
            if (!parsed.isJsonArray()) {
                throw new IOException("Unexpected Modrinth response for " + slug);
            }
            return parsed.getAsJsonArray();
        } finally {
            connection.disconnect();
        }
    }

    private static void download(String url, Path dest) throws IOException {
        HttpURLConnection connection = (HttpURLConnection) new URL(url).openConnection();
        connection.setRequestProperty("User-Agent", USER_AGENT);
        connection.setConnectTimeout(10000);
        connection.setReadTimeout(30000);
        try (InputStream in = connection.getInputStream()) {
            Path tmp = dest.resolveSibling(dest.getFileName() + ".part");
            Files.copy(in, tmp, StandardCopyOption.REPLACE_EXISTING);
            try {
                Files.move(tmp, dest, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
            } catch (IOException e) {
                Files.move(tmp, dest, StandardCopyOption.REPLACE_EXISTING);
            }
        } finally {
            connection.disconnect();
        }
    }
}
