import {
  Alert,
  CircularProgress,
  Paper, Table, TableBody, TableCell,
  TableContainer, TableHead, TableRow, Typography
} from "@mui/material";
import { useEffect, useState } from "react";
import type { Order } from "../types/order";
import { getOrders } from "../api";

export default function OrdersTable() {
  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadOrders();
  }, []);

  async function loadOrders() {
    try {
      setLoading(true);
      const data = await getOrders();
      setOrders(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load orders");
    } finally {
      setLoading(false);
    }
  }

  if (loading) return <CircularProgress sx={{ display: "block", mx: "auto", mt: 4 }} />;

  return (<>
    {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

    {orders.length === 0 ? (
      <Typography color="textSecondary">Noch keine Bestellungen</Typography>
    ) : (
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow sx={{ backgroundColor: "#f5f5f5" }}>
              <TableCell>Order ID</TableCell>
              <TableCell>UUID</TableCell>
              <TableCell>Zeitpunkt</TableCell>
              <TableCell>Summe</TableCell>
              <TableCell>Methode</TableCell>
              <TableCell>Produkte</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {orders.map((order) => (
              <TableRow key={order.id}>
                <TableCell>
                  {order.id}
                </TableCell>
                <TableCell sx={{ fontFamily: "monospace", fontSize: 12 }}>
                  {order.uuid}
                </TableCell>
                <TableCell>
                  {new Date(order.created_at).toLocaleString("de-DE", {
                    dateStyle: "short",
                    timeStyle: "short",
                  })}
                </TableCell>
                <TableCell>€{order.total.toFixed(2)}</TableCell>
                <TableCell>{order.payment_method}</TableCell>
                <TableCell>
                  {order.items.map((item) => `${item.name} x${item.quantity}`).join(", ")}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    )}
  </>)
}
