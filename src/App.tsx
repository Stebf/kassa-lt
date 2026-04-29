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
            Kassa
          </Typography>
          <Button color="inherit" onClick={() => navigate("/")}>
            POS
          </Button>
          <Button color="inherit" onClick={() => navigate("/dashboard")}>
            Orders
          </Button>
          <Button color="inherit" onClick={() => navigate("/products/new")}>
            Add Product
          </Button>
        </Toolbar>
      </AppBar>
      <Router />
    </ThemeProvider>
  );
}

export default App;
