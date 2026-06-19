import { Alert, Snackbar } from "@mui/material";
import { useUiStore } from "../store/uiStore";

export default function GlobalSnackbar() {
  const notification = useUiStore((s) => s.notification);
  const hideNotification = useUiStore((s) => s.hideNotification);

  return (
    <Snackbar
      key={notification?.key ?? "hidden"}
      open={notification !== null}
      autoHideDuration={2000}
      onClose={hideNotification}
      anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
    >
      <Alert
        severity={notification?.severity ?? "info"}
        variant="filled"
        onClose={hideNotification}
        sx={{ width: "100%" }}
      >
        {notification?.message}
      </Alert>
    </Snackbar>
  );
}