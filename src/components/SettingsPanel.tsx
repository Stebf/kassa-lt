import { Alert, Box, Button, Divider, FormControl, InputLabel, MenuItem, Select, Stack, Switch, TextField, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, type FormEvent } from 'react';
import { getBackupConfig, getSyncConfig, runBackupNow, setBackupConfig, setSyncConfig} from '../api';
import type { BackupState } from '../types/backup';

function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    if (Number.isNaN(date.getTime())) {
        return timestamp;
    }

    return date.toLocaleString(undefined, {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        timeZoneName: 'short'
    });
}

function printBackupState(state: BackupState): string {
    switch (state.type) {
        case "NotRunYet":
            return "Backup has not been run yet.";
        case "Successful":
            return `Last backup was successful at ${formatTimestamp(state.timestamp)}.`;
        case "Failed":
            return `Last backup failed at ${formatTimestamp(state.timestamp)} with error: ${state.error}`;
    }
}

function getBackupStatusAlert(params: {
    isLoadingConfig: boolean;
    isLoadingBackupState: boolean;
    backupEnabled: boolean;
    isRunningBackup: boolean;
    backupState: BackupState;
}): { severity: "info" | "success" | "warning" | "error"; message: string } {
    const { isLoadingConfig, isLoadingBackupState, backupEnabled, isRunningBackup, backupState } = params;

    if (isLoadingConfig || isLoadingBackupState) {
        return {
            severity: "info",
            message: "Loading backup settings and current backup status...",
        };
    }

    if (!backupEnabled) {
        return {
            severity: "warning",
            message: "Backup module is disabled. Scheduled backups and manual runs are blocked.",
        };
    }

    if (isRunningBackup) {
        return {
            severity: "info",
            message: "Manual backup is currently running. This can take a while on slower devices.",
        };
    }

    switch (backupState.type) {
        case "Successful":
            return {
                severity: "success",
                message: `Last backup was successful at ${formatTimestamp(backupState.timestamp)}.`,
            };
        case "Failed":
            return {
                severity: "error",
                message: `Last backup failed at ${formatTimestamp(backupState.timestamp)} with error: ${backupState.error}`,
            };
        case "NotRunYet":
        default:
            return {
                severity: "info",
                message: "Backup has not been run yet.",
            };
    }
}

export function SettingsPanel() {
    const [authMethod, setAuthMethod] = useState("Basic");
    const [backupState, setBackupState] = useState<BackupState>({ type: "NotRunYet" });
    const [isRunningBackup, setIsRunningBackup] = useState(false);
    const [backupActionFeedback, setBackupActionFeedback] = useState<{ severity: "success" | "error"; message: string } | null>(null);
    const [backupEnabled, setBackupEnabled] = useState<boolean>(true);
    const [isLoadingConfig, setIsLoadingConfig] = useState(true);
    const [isLoadingBackupState, setIsLoadingBackupState] = useState(true);
    const [protocol, setProtocol] = useState<string>("WebDAV");

    async function handleSubmit(e: FormEvent<HTMLFormElement>) {
        e.preventDefault()

        // Get arguments from form submit event
        const formData = new FormData(e.currentTarget);
        const webdavUrl = formData.get("webdavUrl") as string;
        const username = formData.get("username") as string;
        const password = formData.get("password") as string;
        const authMethod = formData.get("authMethod") as string;
        const protocol = formData.get("protocol") as string;

        const config = { protocol: protocol, webdav_url: webdavUrl, username: username, password: password, auth_method: authMethod, enabled: backupEnabled };
        await setBackupConfig(config as any);
    }

    useEffect(() => {
        async function loadConfig() {
            setIsLoadingConfig(true);
            try {
                const config = await getBackupConfig();
                (document.querySelector('input[name="webdavUrl"]') as HTMLInputElement).value = config.webdav_url;
                (document.querySelector('input[name="username"]') as HTMLInputElement).value = config.username;
                (document.querySelector('input[name="password"]') as HTMLInputElement).value = config.password;
                setAuthMethod(config.auth_method);
                setProtocol(config.protocol);
                setBackupEnabled((config as any).enabled ?? true);
            }
            catch (e) {
                console.error("Failed to load backup config", e);
            } finally {
                setIsLoadingConfig(false);
            }
        }
        async function loadBackupState() {
            setIsLoadingBackupState(true);
            try {
                const state = await invoke<BackupState>("get_backup_state");
                setBackupState(state);
            }
            catch (e) {
                console.error("Failed to load backup state", e);
            } finally {
                setIsLoadingBackupState(false);
            }
        }
        loadBackupState();
        loadConfig();
    }, []);

    async function handleRunBackupNow() {
        setIsRunningBackup(true);
        setBackupActionFeedback(null);

        try {
            const state = await runBackupNow();
            setBackupState(state);

            switch (state.type) {
                case "Successful":
                    setBackupActionFeedback({ severity: "success", message: "Backup completed successfully." });
                    break;
                case "Failed":
                    setBackupActionFeedback({ severity: "error", message: `Backup failed: ${state.error}` });
                    break;
                default:
                    setBackupActionFeedback({ severity: "success", message: "Backup worker ran, but no state was returned." });
                    break;
            }
        } catch (error) {
            console.error("Failed to run backup now", error);
            setBackupActionFeedback({
                severity: "error",
                message: error instanceof Error ? error.message : "Failed to start backup.",
            });
        } finally {
            setIsRunningBackup(false);
        }
    }

    return (
        <Box sx={{ p: 2 }}>
            <Divider textAlign="left"> Backup Einstellungen</Divider>
            <Stack direction="row" spacing={2} sx={{ mt: 2, alignItems: 'center' }}>
                <Typography variant="body2">Backup aktiviert</Typography>
                <Switch checked={backupEnabled} onChange={(e) => setBackupEnabled(e.target.checked)} />
            </Stack>
            <Alert severity="info" variant="outlined" sx={{ mb: 2, textAlign: 'left' }}>
                Given a link to a shared folder of a NextCloud instance like <code>https://&lt;host&gt;/s/&lt;share_token&gt;</code>, the shared folder can be used for backups using the following configuration:
                <ul>
                    <li><strong>WebDav URL</strong>: The share token embedded in <code>https://&lt;host&gt;/public.php/dav/files/&lt;share_token&gt;</code></li>
                    <li><strong>Username</strong>: <code>anonymous</code></li>
                    <li><strong>Password</strong>: <code>""</code> unless password configured</li>
                    <li><strong>Auth Method</strong>: <code>Basic</code></li>
                </ul>
            </Alert>
            <form onSubmit={handleSubmit}>
                <Stack spacing={2}>
                    <FormControl fullWidth>
                        <InputLabel id="protocol-label">Protocol</InputLabel>
                        <Select
                            labelId="protocol-label"
                            name="protocol"
                            label="Protocol"
                            value={protocol}
                            onChange={(e) => setProtocol(e.target.value)}
                        >
                            <MenuItem value={"WebDAV"}>WebDAV</MenuItem>
                            <MenuItem value={"HttpPut"}>HTTP(S) PUT</MenuItem>
                        </Select>
                    </FormControl>
                    <TextField fullWidth label="URL" name="webdavUrl" />
                    <TextField fullWidth label="Username" name="username" />
                    <TextField fullWidth label="Password" name="password" type="password" />
                    <FormControl fullWidth>
                        <InputLabel id="auth-method-label">Auth Method (only relevant for WebDAV protocol)</InputLabel>
                        <Select
                            labelId="auth-method-label"
                            name="authMethod"
                            label="Auth Method"
                            value={authMethod}
                            onChange={(e) => setAuthMethod(e.target.value)}
                        >
                            <MenuItem value={"Basic"}>Basic</MenuItem>
                            <MenuItem value={"Digest"}>Digest</MenuItem>
                        </Select>
                    </FormControl>
                    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
                        <Button type="submit" variant="contained">Speichern</Button>
                        <Button
                            type="button"
                            variant="outlined"
                            onClick={handleRunBackupNow}
                            disabled={isLoadingConfig || isLoadingBackupState || isRunningBackup || !backupEnabled}
                        >
                            {isRunningBackup ? "Backup läuft..." : "Backup jetzt starten"}
                        </Button>
                    </Stack>
                </Stack >
            </form>
            <Alert
                severity={getBackupStatusAlert({
                    isLoadingConfig,
                    isLoadingBackupState,
                    backupEnabled,
                    isRunningBackup,
                    backupState,
                }).severity}
                sx={{ mt: 2 }}
            >
                {getBackupStatusAlert({
                    isLoadingConfig,
                    isLoadingBackupState,
                    backupEnabled,
                    isRunningBackup,
                    backupState,
                }).message}
            </Alert>
            {backupActionFeedback && (
                <Alert severity={backupActionFeedback.severity} sx={{ mt: 2 }}>
                    {backupActionFeedback.message}
                </Alert>
            )}
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                {printBackupState(backupState)}
            </Typography>
            <Divider></Divider>
        </Box>
    );
}

export function SyncSettingsPanel() {
    const [syncEnabled, setSyncEnabled] = useState<boolean>(true);
    const [isLoadingConfig, setIsLoadingConfig] = useState(true);
    const [centralApiBaseUrl, setCentralApiBaseUrl] = useState<string>("");

    useEffect(() => {
        async function loadConfig() {
            setIsLoadingConfig(true);
            try {
                const config = await getSyncConfig();
                setSyncEnabled(config.enabled);
                setCentralApiBaseUrl(config.central_api_base_url);
            }
            catch (e) {
                console.error("Failed to load sync config", e);
            } finally {
                setIsLoadingConfig(false);
            }
        }
        loadConfig();
    }, []);

    async function handleSyncEnabledChange(event: React.ChangeEvent<HTMLInputElement>) {
        const newValue = event.target.checked;
        setSyncEnabled(newValue);
        try {
            await setSyncConfig({
                enabled: newValue,
                central_api_base_url: centralApiBaseUrl,
            });
        } catch (e) {
            console.error("Failed to update sync enabled state", e);
        }
    }

    return (
        <Box sx={{ p: 2 }}>
            <Divider textAlign="left">Sync Einstellungen</Divider>
            <Stack direction="row" spacing={2} sx={{ mt: 2, alignItems: 'center' }}>
                <Typography variant="body2">Sync aktiviert</Typography>
                <Switch checked={syncEnabled} onChange={handleSyncEnabledChange} disabled={isLoadingConfig} />
                <TextField fullWidth label="Central API Base URL" name="centralApiBaseUrl" value={centralApiBaseUrl} onChange={(e) => setCentralApiBaseUrl(e.target.value)} disabled={isLoadingConfig} />
            </Stack>
        </Box>
    );
}