export interface Subtitle {
    id: string;
    url: string;
    language: string;
    type: string;
}

export interface Chapter {
    start: number;
    end: number;
    title: string;
}