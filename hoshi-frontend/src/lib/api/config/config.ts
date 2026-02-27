import { api } from "@/api/client";
//import type {} from "./types";

export const configApi = {

    get() {
        return api<any>(`config`);
    },

    updateConfig(body: any){
        return api<any>(`config`, {
            method: "POST",
            body
        });
    },

    getSection() {
        return api<any>(`config`);
    },

    updateConfigSection(body: any){
        return api<any>(`config`, {
            method: "POST",
            body
        });
    },
};