import { ThemeProvider } from "@mui/material/styles";
import { CssBaseline, AppBar, Toolbar, Typography, Button } from "@mui/material";
import { useNavigate } from "react-router-dom";
import Router from "./Router";
import { theme } from "./theme/theme";
import { useAdmin } from "./context/AdminContext";
import GlobalSnackbar from "./components/GlobalSnackbar";



function App() {
  const navigate = useNavigate();
  const { adminModeEnabled } = useAdmin();

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <AppBar sx={{ backgroundColor: "#9BD8A0", color: "#333333" }} position="static">
        <Toolbar>
          <Typography variant="h6" sx={{ flexGrow: 1, cursor: "pointer" }} onClick={() => navigate("/")}>
            Kassa-LT
          </Typography>
          <Button color="inherit" onClick={() => navigate("/")}>
            Kasse
          </Button>
          <Button color="inherit" onClick={() => navigate("/orders")}>
            Bestellungen
          </Button>
          <Button color="inherit" onClick={() => navigate("/products")} disabled={!adminModeEnabled}>
            Produkte
          </Button>
          <Button color="inherit" onClick={() => navigate("/settings")}>
            Einstellungen
          </Button>
        </Toolbar>
      </AppBar>
      <Router />
      <GlobalSnackbar />
    </ThemeProvider>
  );
}

export default App;
