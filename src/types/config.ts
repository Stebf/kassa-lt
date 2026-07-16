export type BackupWorkerConfig = {
    protocol: string;
    webdav_url: string;
    username: string;
    password: string;
    auth_method: string;
    enabled?: boolean;
}

export type SyncWorkerConfig = {
    central_api_base_url: string;
    enabled: boolean;
}