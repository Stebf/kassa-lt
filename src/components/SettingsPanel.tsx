import { useEffect, useState } from 'react';
import { useAdmin } from '../context/AdminContext';
import { TextField, Box, OutlinedInput, InputLabel, InputAdornment, IconButton, Switch} from '@mui/material';
import FormControl from '@mui/material/FormControl';
import FormGroup from '@mui/material/FormGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';

export default function SettingsPanel() {
    const { adminModeEnabled, setAdminMode } = useAdmin();
    const [showPassword, setShowPassword] = useState(false);

    const handleClickShowPassword = () => {
        setShowPassword(!showPassword);
    };

    const handleMouseDownPassword = (event: React.MouseEvent<HTMLButtonElement>) => {
        event.preventDefault();
    };

    const handleMouseUpPassword = (event: React.MouseEvent<HTMLButtonElement>) => {
        event.preventDefault();
    };

    const handleAdminChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        await setAdminMode(event.target.checked);
    };

    return (
        <Box sx={{ p: 2 }}>
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
            <Box>
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
            </Box>
        </Box>
    );
}