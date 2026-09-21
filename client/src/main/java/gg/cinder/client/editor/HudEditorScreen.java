// Cinder client mod -- original code.
// Package: gg.cinder.client.editor
// Target: Minecraft 1.21.11 / Yarn mappings / Fabric API / Java 21.
package gg.cinder.client.editor;

import gg.cinder.client.hud.HudManager;
import gg.cinder.client.hud.HudModule;
import gg.cinder.client.util.Draw;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.input.KeyInput;
import net.minecraft.text.Text;
import org.lwjgl.glfw.GLFW;

import java.io.IOException;
import java.util.List;

/**
 * Drag-and-drop layout editor for the Cinder HUD.
 *
 * <p>Interactions:
 * <ul>
 *   <li>Left-drag a module box to <b>move</b> it.</li>
 *   <li>{@code Shift}+drag anywhere on a box, or drag its bottom-right corner
 *       handle (8x8 px), to <b>resize</b> it.</li>
 *   <li>Positions snap to an 8px grid while snap is on (toggle button or {@code G}).</li>
 *   <li>Bottom bar: profile Text field plus Save / Load buttons, snap toggle, Done.</li>
 *   <li>{@code Esc} closes (same as Done).</li>
 * </ul>
 *
 * <p>No-conflict rule: while this screen is open, {@link HudManager#renderAll} early-outs
 * (see {@link HudManager#setEditorOpen}), so live module rendering can never fight the
 * editor. This screen draws <b>selection boxes only</b> -- it never calls
 * {@code HudModule.render}. Module boxes are drawn from live x/y/w/h, so dragging
 * updates exactly what will be saved.
 *
 * <p>Performance: render does zero per-frame allocations (indexed loops, prebuilt
 * hint string, status kept in a field updated only on user actions).
 */
public final class HudEditorScreen extends Screen {

    /** Snap grid size in scaled pixels. */
    private static final int GRID = 8;

    /** Bottom-right resize handle edge length in scaled pixels. */
    private static final int HANDLE = 8;

    private static final int MIN_W = 24;
    private static final int MIN_H = 12;

    private static final int BAR_Y_OFFSET = 28;
    private static final int FIELD_W = 148;
    private static final int BUTTON_W = 64;
    private static final int ROW_H = 20;

    private static final String HINT =
            "Drag: move  |  Shift+drag / corner: resize  |  G: snap  |  Esc: done";

    private final Screen parent;

    private HudModule dragging;
    private boolean resizing;
    private int grabDX;
    private int grabDY;
    private HudModule selected;
    private boolean snap = true;

    private ButtonWidget snapButtonWidget;
    private ButtonWidget profileButton;

    /** Active profile name; cycled with the < > buttons (text entry needs a
     * full widget, so the editor cycles saved profiles instead). */
    private String profileName = "default";

    /** Status line; only reassigned on user actions, never per frame. */
    private String status = "";

    public HudEditorScreen(Screen parent) {
        super(Text.literal("Cinder HUD Editor"));
        this.parent = parent;
    }

    // ------------------------------------------------------------------ setup

    @Override
    protected void init() {
        HudManager.setEditorOpen(true);
        dragging = null;
        resizing = false;

        int c = this.width / 2;
        int y = this.height - BAR_Y_OFFSET;

        ButtonWidget saveButton = ButtonWidget.builder(Text.literal("Save"), b -> onSaveProfile())
                .dimensions(c - 210, y, BUTTON_W, ROW_H).build();
        this.addDrawableChild(saveButton);

        this.addDrawableChild(ButtonWidget.builder(Text.literal("Load"), b -> onLoadProfile())
                .dimensions(c - 142, y, BUTTON_W, ROW_H).build());

        this.addDrawableChild(ButtonWidget.builder(Text.literal("<"), b -> cycleProfile(-1))
                .dimensions(c - 74, y, 28, ROW_H).build());
        profileButton = ButtonWidget.builder(Text.literal(profileLabel()), b -> cycleProfile(1))
                .dimensions(c - 44, y, 120, ROW_H).build();
        this.addDrawableChild(profileButton);
        this.addDrawableChild(ButtonWidget.builder(Text.literal(">"), b -> cycleProfile(1))
                .dimensions(c + 78, y, 28, ROW_H).build());

        snapButtonWidget = ButtonWidget.builder(Text.literal(snapLabel()), b -> toggleSnap())
                .dimensions(c + 110, y, BUTTON_W, ROW_H).build();
        this.addDrawableChild(snapButtonWidget);

        this.addDrawableChild(ButtonWidget.builder(Text.literal("Done"), b -> close())
                .dimensions(c + 178, y, BUTTON_W, ROW_H).build());
    }

    private String profileLabel() {
        return "Profile: " + profileName;
    }

    private void cycleProfile(int direction) {
        try {
            java.util.List<String> names = HudManager.listProfiles();
            if (names.isEmpty()) {
                status = "No saved profiles yet — Save stores '" + profileName + "'.";
                return;
            }
            int i = names.indexOf(profileName);
            if (i < 0) {
                i = 0;
            } else {
                i = (i + direction + names.size()) % names.size();
            }
            profileName = names.get(i);
            if (profileButton != null) {
                profileButton.setMessage(Text.literal(profileLabel()));
            }
            status = "Profile: " + profileName;
        } catch (RuntimeException e) {
            status = "Could not list profiles.";
        }
    }

    private String snapLabel() {
        return snap ? "Snap: ON" : "Snap: OFF";
    }

    private void toggleSnap() {
        snap = !snap;
        if (snapButtonWidget != null) {
            snapButtonWidget.setMessage(Text.literal(snapLabel()));
        }
        status = snap ? "Snap to 8px grid: on" : "Snap to 8px grid: off";
    }

    private void onSaveProfile() {
        try {
            HudManager.saveProfile(profileName);
            status = "Saved '" + profileName + "'.";
        } catch (IllegalArgumentException e) {
            status = "Bad name: use 1-32 chars A-Z 0-9 space _ -";
        } catch (IOException e) {
            status = "Save failed: file error";
        }
    }

    private void onLoadProfile() {
        try {
            HudManager.loadProfile(profileName);
            status = "Loaded '" + profileName + "'.";
        } catch (IllegalArgumentException e) {
            status = "Bad name: use 1-32 chars A-Z 0-9 space _ -";
        } catch (IOException e) {
            status = "Load failed: not found or corrupt";
        }
    }

    // ----------------------------------------------------------------- render

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        renderBackground(context, mouseX, mouseY, delta);

        // Dim the world behind the editor.
        context.fill(0, 0, this.width, this.height, 0x80000000);

        // Selection boxes only -- never module.render (see class javadoc).
        List<HudModule> modules = HudManager.getModules();
        for (int i = 0; i < modules.size(); i++) {
            HudModule m = modules.get(i);
            if (!m.isEnabled()) {
                continue;
            }
            boolean hot = m == selected || m == dragging;
            int fill = hot ? 0x604242D8 : Draw.SELECT_FILL;
            int border = hot ? 0xFF9D9DFF : Draw.SELECT_BORDER;
            Draw.drawBox(context, m.getX(), m.getY(), m.getW(), m.getH(), fill, border);
            Draw.drawTextWithShadow(context, this.client, m.label(), m.getX() + 3, m.getY() + 3, Draw.TEXT_WHITE);
            // Resize handle marker.
            int hx = m.getX() + m.getW() - HANDLE;
            int hy = m.getY() + m.getH() - HANDLE;
            Draw.fillRect(context, hx, hy, HANDLE, HANDLE, hot ? 0xFF9D9DFF : 0xFF6A6AF5);
        }

        Draw.drawTextWithShadow(context, this.client, HINT, 8, 8, 0xFFE8E8FF);
        if (!status.isEmpty()) {
            Draw.drawTextWithShadow(context, this.client, status, 8, this.height - BAR_Y_OFFSET - 16, 0xFFFFD479);
        }

        // Render the profile field manually since it's not an Element
        // Note: EditBox.render is not public; we skip for now
        // profileField.render(context, mouseX, mouseY, delta);

        // Widgets (buttons) on top.
        super.render(context, mouseX, mouseY, delta);
    }

    // ------------------------------------------------------------------ input

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        // Let widgets (buttons, Text field) consume clicks first.
        if (super.mouseClicked(click, doubled)) {
            return true;
        }
        if (click.button() != 0) {
            return false;
        }
        List<HudModule> modules = HudManager.getModules();
        boolean shift = (click.modifiers() & GLFW.GLFW_MOD_SHIFT) != 0;
        for (int i = modules.size() - 1; i >= 0; i--) {
            HudModule m = modules.get(i);
            if (!m.isEnabled()) {
                continue;
            }
            if (m.isCornerHandle(click.x(), click.y(), HANDLE) || (shift && m.contains(click.x(), click.y()))) {
                dragging = m;
                resizing = true;
                selected = m;
                return true;
            }
            if (m.contains(click.x(), click.y())) {
                dragging = m;
                resizing = false;
                grabDX = (int) click.x() - m.getX();
                grabDY = (int) click.y() - m.getY();
                selected = m;
                return true;
            }
        }
        selected = null;
        return false;
    }

    @Override
    public boolean mouseDragged(Click click, double deltaX, double deltaY) {
        if (super.mouseDragged(click, deltaX, deltaY)) {
            return true;
        }
        if (click.button() != 0 || dragging == null) {
            return false;
        }
        double mouseX = click.x();
        double mouseY = click.y();
        if (resizing) {
            int nw = (int) mouseX - dragging.getX();
            int nh = (int) mouseY - dragging.getY();
            if (snap) {
                nw = HudModule.snapTo(nw, GRID);
                nh = HudModule.snapTo(nh, GRID);
            }
            dragging.resizeTo(nw, nh, MIN_W, MIN_H, this.width, this.height);
        } else {
            int nx = (int) mouseX - grabDX;
            int ny = (int) mouseY - grabDY;
            if (snap) {
                nx = HudModule.snapTo(nx, GRID);
                ny = HudModule.snapTo(ny, GRID);
            }
            dragging.moveTo(nx, ny, this.width, this.height);
        }
        return true;
    }

    @Override
    public boolean mouseReleased(Click click) {
        if (super.mouseReleased(click)) {
            return true;
        }
        if (click.button() == 0 && dragging != null) {
            dragging = null;
            resizing = false;
            HudManager.save();
            status = "Layout saved.";
            return true;
        }
        return false;
    }

    @Override
    public boolean keyPressed(KeyInput input) {
        if (input.key() == GLFW.GLFW_KEY_G) {
            toggleSnap();
            return true;
        }
        if (input.key() == GLFW.GLFW_KEY_ESCAPE) {
            close();
            return true;
        }
        return super.keyPressed(input);
    }

    @Override
    public void close() {
        HudManager.setEditorOpen(false);
        HudManager.save();
        if (this.client != null) {
            this.client.setScreen(parent);
        }
    }

    @Override
    public void removed() {
        HudManager.setEditorOpen(false);
        super.removed();
    }

    @Override
    public boolean shouldPause() {
        // Do not pause the world (single-player) while arranging the HUD.
        return false;
    }
}