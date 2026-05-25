import { Alert, Box, Button, FormControl, InputLabel, MenuItem, Select, Stack, TextField, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, type FormEvent } from 'react';
import { getBackupConfig, runBackupNow, setBackupConfig} from '../api';
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

export default function SettingsPanel() {
    const [authMethod, setAuthMethod] = useState("Basic");
    const [backupState, setBackupState] = useState<BackupState>({ type: "NotRunYet" });
    const [isRunningBackup, setIsRunningBackup] = useState(false);
    const [backupActionFeedback, setBackupActionFeedback] = useState<{ severity: "success" | "error"; message: string } | null>(null);

    async function handleSubmit(e: FormEvent<HTMLFormElement>) {
        e.preventDefault()

        // Get arguments from form submit event
        const formData = new FormData(e.currentTarget);
        const webdavUrl = formData.get("webdavUrl") as string;
        const username = formData.get("username") as string;
        const password = formData.get("password") as string;
        const authMethod = formData.get("authMethod") as string;

        const config = { webdav_url: webdavUrl, username: username, password: password, auth_method: authMethod };
        await setBackupConfig(config);
    }

    useEffect(() => {
        async function loadConfig() {
            try {
                const config = await getBackupConfig();
                (document.querySelector('input[name="webdavUrl"]') as HTMLInputElement).value = config.webdav_url;
                (document.querySelector('input[name="username"]') as HTMLInputElement).value = config.username;
                (document.querySelector('input[name="password"]') as HTMLInputElement).value = config.password;
                setAuthMethod(config.auth_method);
            }
            catch (e) {
                console.error("Failed to load backup config", e);
            }
        }
        async function loadBackupState() {
            try {
                const state = await invoke<BackupState>("get_backup_state");
                setBackupState(state);
            }
            catch (e) {
                console.error("Failed to load backup state", e);
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
                    <TextField fullWidth label="WebDAV URL" name="webdavUrl" />
                    <TextField fullWidth label="Username" name="username" />
                    <TextField fullWidth label="Password" name="password" type="password" />
                    <FormControl fullWidth>
                        <InputLabel id="auth-method-label">Auth Method</InputLabel>
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
                            disabled={isRunningBackup}
                        >
                            {isRunningBackup ? "Backup läuft..." : "Backup jetzt starten"}
                        </Button>
                    </Stack>
                </Stack >
            </form>
            {backupActionFeedback && (
                <Alert severity={backupActionFeedback.severity} sx={{ mt: 2 }}>
                    {backupActionFeedback.message}
                </Alert>
            )}
            <Typography variant="body2" color="text.secondary">
                {printBackupState(backupState)}
            </Typography>
        </Box>
    );
}
