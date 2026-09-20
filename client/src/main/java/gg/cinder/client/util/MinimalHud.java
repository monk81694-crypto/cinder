// Cinder client mod -- original code.
// Minimal-hud flags: cosmetic visibility preferences with a documented hook.
// Hiding vanilla scoreboard/chat elements needs dedicated mixins; this class
// only stores the preference so a future mixin can consult it without touching
// config files. HudManager may call shouldHideScoreboard()/shouldHideChat().
package gg.cinder.client.util;

public final class MinimalHud {

    private static boolean hideScoreboard;
    private static boolean hideChat;

    private MinimalHud() {
    }

    public static boolean shouldHideScoreboard() {
        return hideScoreboard;
    }

    public static boolean shouldHideChat() {
        return hideChat;
    }

    public static void setHideScoreboard(boolean hide) {
        hideScoreboard = hide;
    }

    public static void setHideChat(boolean hide) {
        hideChat = hide;
    }
}
