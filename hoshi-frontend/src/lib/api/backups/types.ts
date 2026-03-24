export type BackupTrigger = "PRE_IMPORT" | "MANUAL" | "REMOTE_SYNC";

export interface ListBackupMeta {
    id: number;
    userId: number;
    trigger: BackupTrigger;
    trackerName?: string | null;
    filePath: string;
    entryCount: number;
    createdAt: number;
}