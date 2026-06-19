import {
  Box,
  Typography,
  Button,
  Stack,
} from "@mui/material";
import { exportCSV, exportOrders } from "../api";
import OrdersTable from "../components/OrdersTable";
import OrdersStatisticsTable from "../components/OrdersStatisticsTable";
import { save } from '@tauri-apps/plugin-dialog';
import { useUiStore } from "../store/uiStore";

export default function OrdersPage() {
  const notify = useUiStore((s) => s.notify);

  function handleExport() {
    const exportPath = save({
      title: "Exportiere Bestellungen als CSV",
      defaultPath: "orders_export.csv",
      filters: [
        { name: "CSV Dateien", extensions: ["csv"] },
        { name: "Alle Dateien", extensions: ["*"] }
      ]
    });

    exportPath.then((path) => {
      if (path) {
        exportOrders().then((orders) => {
          exportCSV(orders, path);
          notify.success("Export erfolgreich.");
        });
      }
    });
  }

  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Bestellungen
      </Typography>
      <Box sx={{ mb: 2 }}>
        <Button variant="contained" onClick={handleExport}>
          Export
        </Button>
      </Box>

      <Stack spacing={4}>
        <OrdersStatisticsTable />
        <OrdersTable />
      </Stack>

    </Box>
  );
}
