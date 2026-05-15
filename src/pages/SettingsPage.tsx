import { Box, Typography } from "@mui/material";
import SettingsPanel from "../components/SettingsPanel";

export default function SettingsPage() {
    return (
        <Box sx={{ width: "80%", maxWidth: "none", mx: "auto", mt: 4, p: 2 }}>
            <Typography variant="h5">Einstellungen</Typography>
            <SettingsPanel />
        </Box>
    );
}