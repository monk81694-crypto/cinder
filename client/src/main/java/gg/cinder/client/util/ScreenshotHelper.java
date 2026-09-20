// Cinder client mod -- original code.
// Screenshot helper: copies the newest screenshot to the OS clipboard and can
// open the screenshots folder. Clipboard work is AWT-based and fully guarded
// (headless JVMs simply report unavailability instead of crashing).
package gg.cinder.client.util;

import java.awt.GraphicsEnvironment;
import java.awt.Toolkit;
import java.awt.datatransfer.DataFlavor;
import java.awt.datatransfer.Transferable;
import java.awt.datatransfer.UnsupportedFlavorException;
import java.awt.image.BufferedImage;
import java.io.File;
import java.util.Arrays;
import java.util.Comparator;
import javax.imageio.ImageIO;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import net.minecraft.util.Util;
import org.lwjgl.glfw.GLFW;

public final class ScreenshotHelper {

    private static KeyBinding snapshotKey;

    private ScreenshotHelper() {
    }

    /** Registers the screenshot key (default F9). Idempotent. */
    public static void registerKey() {
        if (snapshotKey != null) {
            return;
        }
        snapshotKey = KeyBindingHelper.registerKeyBinding(new KeyBinding(
                "key.cinder.screenshot", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_F9,
                KeyBinding.Category.create(Identifier.of("cinder", "general"))));
    }

    /** Polls the key; call every client tick. */
    public static void pollKeys(MinecraftClient mc) {
        if (snapshotKey == null) {
            return;
        }
        while (snapshotKey.wasPressed()) {
            String status = snapshotClipboard();
            if (mc != null && mc.player != null) {
                mc.player.sendMessage(Text.literal(status), false);
            }
        }
    }

    /** Copies the newest screenshots/*.png to the clipboard. Never throws. */
    public static String snapshotClipboard() {
        try {
            if (GraphicsEnvironment.isHeadless()) {
                return "Clipboard unavailable (headless)";
            }
            MinecraftClient mc = MinecraftClient.getInstance();
            if (mc == null || mc.runDirectory == null) {
                return "Screenshots folder not found";
            }
            File dir = new File(mc.runDirectory, "screenshots");
            File[] pngs = dir.listFiles((d, name) -> name.toLowerCase().endsWith(".png"));
            if (pngs == null || pngs.length == 0) {
                return "No screenshots yet — press F2 first";
            }
            Arrays.sort(pngs, Comparator.comparingLong(File::lastModified));
            BufferedImage image = ImageIO.read(pngs[pngs.length - 1]);
            if (image == null) {
                return "Could not read screenshot";
            }
            Toolkit.getDefaultToolkit().getSystemClipboard().setContents(new ImageTransferable(image), null);
            return "Copied " + pngs[pngs.length - 1].getName();
        } catch (Exception e) {
            return "Screenshot copy failed";
        }
    }

    /** Opens the screenshots folder in the OS file manager. Never throws. */
    public static void openScreenshotsFolder() {
        try {
            MinecraftClient mc = MinecraftClient.getInstance();
            if (mc == null || mc.runDirectory == null) {
                return;
            }
            File dir = new File(mc.runDirectory, "screenshots");
            dir.mkdirs();
            Util.getOperatingSystem().open(dir);
        } catch (Exception e) {
            try {
                if (!GraphicsEnvironment.isHeadless()) {
                    java.awt.Desktop.getDesktop().open(new File(MinecraftClient.getInstance().runDirectory, "screenshots"));
                }
            } catch (Exception ignored) {
            }
        }
    }

    /** Minimal image Transferable (no external deps). */
    private static final class ImageTransferable implements Transferable {
        private final BufferedImage image;

        ImageTransferable(BufferedImage image) {
            this.image = image;
        }

        @Override
        public DataFlavor[] getTransferDataFlavors() {
            return new DataFlavor[] { DataFlavor.imageFlavor };
        }

        @Override
        public boolean isDataFlavorSupported(DataFlavor flavor) {
            return DataFlavor.imageFlavor.equals(flavor);
        }

        @Override
        public Object getTransferData(DataFlavor flavor) throws UnsupportedFlavorException {
            if (!isDataFlavorSupported(flavor)) {
                throw new UnsupportedFlavorException(flavor);
            }
            return image;
        }
    }
}
