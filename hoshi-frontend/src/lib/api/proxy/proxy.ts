import { api } from "@/api/client";

export interface ProxyParams {
    url: string;
    referer?: string;
    origin?: string;
    userAgent?: string;
}
export const proxyApi = {
    fetch(params: ProxyParams) {
        return api<Blob>("proxy", {
            method: "GET",
            params,
        });
    },
};