import { Box, Switch } from "@mui/material";
import FormGroup from '@mui/material/FormGroup';
import FormControlLabel from '@mui/material/FormControlLabel';
import { useAdmin } from "../context/AdminContext";
// import { useState } from "react";

// import FormControl from '@mui/material/FormControl';

// import Visibility from '@mui/icons-material/Visibility';
// import VisibilityOff from '@mui/icons-material/VisibilityOff';

export default function SettingsAdminModePanel() {
    const { adminModeEnabled, setAdminMode } = useAdmin();
    // const [showPassword, setShowPassword] = useState(false);

    const handleAdminChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        await setAdminMode(event.target.checked);
    };

    // const handleClickShowPassword = () => {
    //     setShowPassword(!showPassword);
    // };

    // const handleMouseDownPassword = (event: React.MouseEvent<HTMLButtonElement>) => {
    //     event.preventDefault();
    // };

    // const handleMouseUpPassword = (event: React.MouseEvent<HTMLButtonElement>) => {
    //     event.preventDefault();
    // };

    return (
    <Box sx={{ display: 'flex', alignItems: 'center' }}>
        <FormGroup>

            <FormControlLabel
                control={<Switch 
                    checked={adminModeEnabled}
                    onChange={handleAdminChange}
                    />} 
                label="Admin Modus"
                labelPlacement='start'
                />

        </FormGroup>
        {/* <Box>
                <FormControl sx={{ m: 1, width: '25ch' }} variant="outlined">
                <InputLabel htmlFor={`admin-input`}>Passwort</InputLabel>
                <OutlinedInput
                    disabled
                    id={`admin-input`}
                    type={showPassword ? 'text' : 'password'}
                    endAdornment={
                    <InputAdornment position="end">
                        <IconButton
                        aria-label={
                            showPassword ? 'hide the password' : 'display the password'
                        }
                        onClick={handleClickShowPassword}
                        onMouseDown={handleMouseDownPassword}
                        onMouseUp={handleMouseUpPassword}
                        edge="end"
                        >
                        {showPassword ? <VisibilityOff /> : <Visibility />}
                        </IconButton>
                    </InputAdornment>
                    }
                    label="Passwort"
                />
                </FormControl>
            </Box> */}
    </Box>
    );
}