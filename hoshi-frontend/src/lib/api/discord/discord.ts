import { call } from "@/api/client";
import type { DiscordActivity } from "./types";

export const discordApi = {
    /**
     * Actualiza el estado de Discord.
     * Si incluyes startTime y endTime, se mostrará la barra de progreso (Watching).
     * Si no los incluyes, se mostrará como una actividad normal (Reading).
     */
    setActivity(activity: DiscordActivity) {
        return call<void>({
            http:  { path: "discord/activity", method: "POST", body: activity },
            tauri: { cmd: "set_activity", args: activity },
        });
    },

    /**
     * Elimina la actividad actual de Discord.
     */
    clearActivity() {
        return call<void>({
            http:  { path: "discord/activity", method: "DELETE" },
            tauri: { cmd: "clear_activity" },
        });
    }
};