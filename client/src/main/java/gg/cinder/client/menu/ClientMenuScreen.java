package gg.cinder.client.menu;

import gg.cinder.client.CinderClient;
import gg.cinder.client.editor.HudEditorScreen;
import gg.cinder.client.hud.HudManager;
import gg.cinder.client.hud.HudModule;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

/**
 * Main Cinder client menu screen, opened with Right Shift.
 *
 * <p>Original code, no external GUI libraries: the background, title and
 * hover highlights are drawn manually with {@link DrawContext#fill} using
 * the Cinder brand palette (charcoal {@code #101024} background, ember
 * {@code #FF6B1A} hover borders, {@code #E8E8F5} text).</p>
 *
 * <p>The screen lists one toggle button per HUD module id queried from
 * {@code HudManager.getModules()}, plus {@code "HUD Editor"},
 * {@code "Performance"} and {@code "Done"} buttons.</p>
 *
 * <p>Cross-agent API assumptions (HudManager, HudModule, HudEditorScreen
 * and PerfScreen are owned by other agents):</p>
 * <ul>
 *   <li>{@code HudManager.getModules()} returns a
 *       {@code Collection<HudModule>} (never null in practice; null and
 *       failures are tolerated here).</li>
 *   <li>{@code HudModule} exposes {@code String getId()},
 *       {@code boolean isEnabled()} and {@code void setEnabled(boolean)}.</li>
 *   <li>{@code gg.cinder.client.editor.HudEditorScreen} has a
 *       {@code HudEditorScreen(Screen parent)} constructor.</li>
 *   <li>{@code gg.cinder.client.menu.PerfScreen} (perf agent) has a
 *       {@code PerfScreen(Screen parent)} constructor.</li>
 * </ul>
 */
public class ClientMenuScreen extends Screen {

    /** Charcoal background with EE alpha. */
    private static final int BACKGROUND = 0xEE101024;

    /** Near-white text. */
    private static final int TEXT_COLOR = 0xFFE8E8F5;

    /** Dimmed subtitle text. */
    private static final int DIM_TEXT = 0xFF9A9AB5;

    /** Ember accent for hover borders and the title rule. */
    private static final int EMBER = 0xFFFF6B1A;

    private static final int MODULE_BUTTON_W = 150;
    private static final int MODULE_BUTTON_H = 20;
    private static final int MODULE_COL_GAP = 8;
    private static final int MODULE_ROW_GAP = 6;
    private static final int MODULE_LIST_TOP = 56;

    private final Screen parent;
    private final List<ButtonWidget> moduleButtons = new ArrayList<>();
    private final List<ButtonWidget> navButtons = new ArrayList<>();

    /**
     * Creates the menu.
     *
     * @param parent screen to return to when closed; may be null
     */
    public ClientMenuScreen(Screen parent) {
        super(Text.literal("Cinder Client"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        moduleButtons.clear();
        navButtons.clear();

        Collection<HudModule> modules = queryModules();
        int listWidth = MODULE_BUTTON_W * 2 + MODULE_COL_GAP;
        int startX = this.width / 2 - listWidth / 2;
        int index = 0;
        for (HudModule module : modules) {
            final HudModule entry = module;
            final String id = entry.getId();
            int col = index % 2;
            int row = index / 2;
            int x = startX + col * (MODULE_BUTTON_W + MODULE_COL_GAP);
            int y = MODULE_LIST_TOP + row * (MODULE_BUTTON_H + MODULE_ROW_GAP);
            ButtonWidget button = ButtonWidget.builder(labelFor(id, entry.isEnabled()), pressed -> {
                        entry.setEnabled(!entry.isEnabled());
                        CinderClient.getConfig().setModuleEnabled(id, entry.isEnabled());
                        pressed.setMessage(labelFor(id, entry.isEnabled()));
                    })
                    .dimensions(x, y, MODULE_BUTTON_W, MODULE_BUTTON_H)
                    .build();
            this.addDrawableChild(button);
            moduleButtons.add(button);
            index++;
        }

        int navY = this.height - 32;
        ButtonWidget editorButton = ButtonWidget.builder(Text.literal("HUD Editor"), pressed -> openHudEditor())
                .dimensions(this.width / 2 - 163, navY, 110, 20)
                .build();
        ButtonWidget perfButton = ButtonWidget.builder(Text.literal("Performance"), pressed -> openPerfScreen())
                .dimensions(this.width / 2 - 45, navY, 110, 20)
                .build();
        ButtonWidget doneButton = ButtonWidget.builder(Text.literal("Done"), pressed -> this.close())
                .dimensions(this.width / 2 + 73, navY, 90, 20)
                .build();
        this.addDrawableChild(editorButton);
        this.addDrawableChild(perfButton);
        this.addDrawableChild(doneButton);
        navButtons.add(editorButton);
        navButtons.add(perfButton);
        navButtons.add(doneButton);
    }

    private static Collection<HudModule> queryModules() {
        try {
            Collection<HudModule> modules = HudManager.getModules();
            return modules != null ? modules : Collections.emptyList();
        } catch (Exception e) {
            CinderClient.LOGGER.error("Cinder Client: failed to query HUD modules", e);
            return Collections.emptyList();
        }
    }

    private static Text labelFor(String id, boolean enabled) {
        return Text.literal(id + ": " + (enabled ? "ON" : "OFF"));
    }

    private void openHudEditor() {
        if (this.client != null) {
            this.client.setScreen(new HudEditorScreen(this));
        }
    }

    private void openPerfScreen() {
        if (this.client != null) {
            this.client.setScreen(new PerfScreen(this));
        }
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, this.width, this.height, BACKGROUND);
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.getTitle(), this.width / 2, 20, TEXT_COLOR);
        int ruleHalf = 60;
        context.fill(this.width / 2 - ruleHalf, 33, this.width / 2 + ruleHalf, 34, EMBER);
        if (!moduleButtons.isEmpty()) {
            context.drawCenteredTextWithShadow(
                    this.textRenderer, "HUD modules", this.width / 2, MODULE_LIST_TOP - 14, DIM_TEXT);
        } else {
            context.drawCenteredTextWithShadow(
                    this.textRenderer, "No HUD modules registered", this.width / 2, MODULE_LIST_TOP, DIM_TEXT);
        }
        for (ButtonWidget button : moduleButtons) {
            drawHoverBorder(context, button);
        }
        for (ButtonWidget button : navButtons) {
            drawHoverBorder(context, button);
        }
    }

    private static void drawHoverBorder(DrawContext context, ButtonWidget button) {
        if (button.isHovered()) {
            int x = button.getX();
            int y = button.getY();
            int right = x + button.getWidth();
            int bottom = y + button.getHeight();
            context.fill(x - 1, y - 1, right + 1, y, EMBER);
            context.fill(x - 1, bottom, right + 1, bottom + 1, EMBER);
            context.fill(x - 1, y, x, bottom, EMBER);
            context.fill(right, y, right + 1, bottom, EMBER);
        }
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
