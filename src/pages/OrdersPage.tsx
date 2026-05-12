import {
  Box,
  Typography,
  Button,
  Stack,
} from "@mui/material";
import { exportCSV, exportOrders } from "../api";
import OrdersTable from "../components/OrdersTable";
import OrdersStatisticsTable from "../components/OrdersStatisticsTable";

export default function OrdersPage() {
  return (
    <Box sx={{ p: 2 }}>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Bestellungen
      </Typography>
      <Box sx={{ mb: 2 }}>
        <Button variant="contained" onClick={async () => exportCSV(await exportOrders())}>
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
