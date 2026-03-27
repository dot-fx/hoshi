import { call } from "@/api/client";
import type { ListBackupMeta } from "./types";

export const backupsApi = {
    getAll() {
        return call<ListBackupMeta[]>({
            tauri: { cmd: "list_backups" },
        });
    },

    createManual() {
        return call<ListBackupMeta>({
            tauri: { cmd: "create_manual_backup" },
        });
    },

    remove_b(backupId: number) {
        return call<void>({
            tauri: { cmd: "delete_backup", args: { backupId } },
        });
    },

    restore_b(backupId: number) {
        return call<void>({
            tauri: { cmd: "restore_backup", args: { backupId } },
        });
    },

    download(backupId: number) {
        return call<void>({
            tauri: { cmd: "download_backup", args: { backupId } },
        });
    },
};