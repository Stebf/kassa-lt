import { ThemeProvider } from "@mui/material/styles";
import { CssBaseline, AppBar, Toolbar, Typography, Button } from "@mui/material";
import { useNavigate } from "react-router-dom";
import Router from "./Router";
import { theme } from "./theme/theme";

function App() {
  const navigate = useNavigate();

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <AppBar position="static">
        <Toolbar>
          <Typography variant="h6" sx={{ flexGrow: 1, cursor: "pointer" }} onClick={() => navigate("/")}>
            Kassa-LT
          </Typography>
          <Button color="inherit" onClick={() => navigate("/")}>
            Kasse
          </Button>
          <Button color="inherit" onClick={() => navigate("/dashboard")}>
            Bestellungen
          </Button>
          <Button color="inherit" onClick={() => navigate("/products")}>
            Produkte
          </Button>
        </Toolbar>
      </AppBar>
      <Router />
    </ThemeProvider>
  );
}

export default App;
