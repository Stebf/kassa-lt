export type BackupWorkerConfig = {
    webdav_url: string;
    username: string;
    password: string;
    auth_method: string;
    enabled?: boolean;
}