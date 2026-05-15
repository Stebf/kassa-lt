import { Box, Typography } from "@mui/material";
import SettingsPanel from "../components/SettingsPanel";
import { useAdmin } from '../context/AdminContext';
import SettingsAdminModePanel from "../components/SettingsAdminModePanel";

export default function SettingsPage() {
    const { adminModeEnabled, setAdminMode } = useAdmin();
    return (
        <Box sx={{ width: "80%", maxWidth: "none", mx: "auto", mt: 4, p: 2 }}>
            <Typography variant="h5">Einstellungen</Typography>
            <SettingsAdminModePanel />
            {adminModeEnabled && <SettingsPanel />}
        </Box>
    );
}