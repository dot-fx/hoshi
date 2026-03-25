import { call } from "@/api/client";
import type { DiscordActivity } from "./types";

export const discordApi = {

    setActivity(activity: DiscordActivity) {
        return call<void>({
            http:  { path: "discord/activity", method: "POST", body: activity },
            tauri: { cmd: "set_activity", args: activity },
        });
    },

    clearActivity() {
        return call<void>({
            http:  { path: "discord/activity", method: "DELETE" },
            tauri: { cmd: "clear_activity" },
        });
    }
};