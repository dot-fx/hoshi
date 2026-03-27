import { call } from "@/api/client";
import type { DiscordActivity } from "./types";

export const discordApi = {

    setActivity(activity: DiscordActivity) {
        return call<void>({
            tauri: { cmd: "set_activity", args: activity },
        });
    },

    clearActivity() {
        return call<void>({
            tauri: { cmd: "clear_activity" },
        });
    }
};