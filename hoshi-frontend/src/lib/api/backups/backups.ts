import { call } from "@/api/client";
import type { ListBackupMeta } from "./types";

export const backupsApi = {
    getAll() {
        return call<ListBackupMeta[]>({
            http:  { path: "backups", method: "GET" },
            tauri: { cmd: "list_backups" },
        });
    },

    createManual() {
        return call<ListBackupMeta>({
            http:  { path: "backups", method: "POST" },
            tauri: { cmd: "create_manual_backup" },
        });
    },

    remove_b(backupId: number) {
        return call<void>({
            http:  { path: `backups/${backupId}`, method: "DELETE" },
            tauri: { cmd: "delete_backup", args: { backupId } },
        });
    },

    restore_b(backupId: number) {
        return call<void>({
            http:  { path: `backups/${backupId}/restore`, method: "POST" },
            tauri: { cmd: "restore_backup", args: { backupId } },
        });
    },

    download(backupId: number) {
        return call<void>({
            http:  { path: `backups/${backupId}/download`, method: "GET" },
            tauri: { cmd: "download_backup", args: { backupId } },
        });
    },
};