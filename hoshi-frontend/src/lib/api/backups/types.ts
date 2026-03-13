export type BackupTrigger = "PRE_IMPORT" | "MANUAL";

export interface ListBackupMeta {
    id: number;
    userId: number;
    trigger: BackupTrigger;
    trackerName?: string | null;
    filePath: string;
    entryCount: number;
    createdAt: number;
}