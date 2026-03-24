export interface DiscordActivity {
    title: string;
    details: string;
    imageUrl?: string | null;
    startTime?: number | null;
    endTime?: number | null;
    isVideo: boolean;
}