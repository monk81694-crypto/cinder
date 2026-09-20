// Cinder client mod -- original code.
// Performance screen: one-click open-source performance pack + undo.
package gg.cinder.client.menu;

import gg.cinder.client.perf.PerformanceInstaller;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public class PerfScreen extends Screen {

    private static final int TEXT_COLOR = 0xFFE8E8F5;
    private static final int DIM_TEXT = 0xFF9A9AB5;
    private static final int BACKGROUND = 0xEE101024;

    private final Screen parent;
    private String status = "Sodium, Lithium, FerriteCore — fetched from Modrinth, never bundled.";

    public PerfScreen(Screen parent) {
        super(Text.literal("Performance"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        ButtonWidget installButton = ButtonWidget.builder(Text.literal("Install performance pack"), pressed -> {
                    status = "Downloading from Modrinth…";
                    PerformanceInstaller.install(false);
                    status = "Working in the background — check the log.";
                })
                .dimensions(this.width / 2 - 155, 64, 150, 20)
                .build();
        ButtonWidget irisButton = ButtonWidget.builder(Text.literal("Install + Iris"), pressed -> {
                    status = "Downloading from Modrinth…";
                    PerformanceInstaller.install(true);
                    status = "Working in the background — check the log.";
                })
                .dimensions(this.width / 2 + 5, 64, 150, 20)
                .build();
        ButtonWidget undoButton = ButtonWidget.builder(Text.literal("Undo installs"), pressed -> {
                    List<String> removed = PerformanceInstaller.undo();
                    status = removed.isEmpty() ? "Nothing to remove." : "Removed: " + String.join(", ", removed);
                })
                .dimensions(this.width / 2 - 155, 90, 150, 20)
                .build();
        ButtonWidget doneButton = ButtonWidget.builder(Text.literal("Done"), pressed -> this.close())
                .dimensions(this.width / 2 + 5, 90, 150, 20)
                .build();
        this.addDrawableChild(installButton);
        this.addDrawableChild(irisButton);
        this.addDrawableChild(undoButton);
        this.addDrawableChild(doneButton);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, this.width, this.height, BACKGROUND);
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.getTitle(), this.width / 2, 24, TEXT_COLOR);
        context.drawCenteredTextWithShadow(this.textRenderer, status, this.width / 2, 118, DIM_TEXT);
    }

    @Override
    public void close() {
        if (this.client != null) {
            this.client.setScreen(this.parent);
        }
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
