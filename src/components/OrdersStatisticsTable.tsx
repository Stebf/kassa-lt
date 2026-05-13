import {
  Alert,
  CircularProgress,
  Paper, Table, TableBody, TableCell,
  TableContainer, TableHead, TableRow, Typography
} from "@mui/material";
import { useEffect, useState } from "react";
import type { ProductSalesCount } from "../types/order";
import { getProductSalesCount } from "../api";

export default function OrdersStatisticsTable() {
  const [productSalesCount, setProductSalesCount] = useState<ProductSalesCount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadOrdersStatistics();
  }, []);

  async function loadOrdersStatistics() {
    try {
      setLoading(true);
      const data = await getProductSalesCount();
      setProductSalesCount(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load product sales count");
    } finally {
      setLoading(false);
    }
  }

  if (loading) return <CircularProgress sx={{ display: "block", mx: "auto", mt: 4 }} />;

  return (<>
    {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

    {productSalesCount.length === 0 ? (
      <Typography color="textSecondary">Noch keine Bestellungen</Typography>
    ) : (
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow sx={{ backgroundColor: "#f5f5f5" }}>
              <TableCell>Produkt</TableCell>
              <TableCell>Anzahl Verkäufe</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {productSalesCount.map((sales) => (
              <TableRow key={sales.product_name}>
                <TableCell>
                  {sales.product_name}
                </TableCell>
                <TableCell>{sales.count}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    )}
  </>)
}
