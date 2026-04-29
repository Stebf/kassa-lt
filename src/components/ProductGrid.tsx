import {
  Grid,
  Paper,
  Button,
  CircularProgress,
  Box
} from "@mui/material";

import { useEffect, useState } from "react";
import { useCartStore } from "../store/cartStore";
import { getProducts } from "../api";
import type { Product } from "../types/product";

type ProductGridProps = {
  reloadKey?: number;
};

export default function ProductGrid({ reloadKey = 0 }: ProductGridProps) {
  const enqueueAdd = useCartStore((s) => s.enqueueAdd);

  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    load();
  }, [reloadKey]);

  async function load() {
    try {
      const data = await getProducts();
      setProducts(data);
      setLoadError(null);
    } catch {
      setProducts([]);
      setLoadError("Produkte konnten nicht geladen werden.");
    } finally {
      setLoading(false);
    }
  }

  function handleAdd(id: number) {
    const selected = products.find((product) => product.id === id);

    if (!selected) {
      return;
    }

    enqueueAdd(selected);
  }

  if (loading) {
    return (
      <Box sx={{ p: 4 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (loadError) {
    return (
      <Box sx={{ p: 2 }}>
        {loadError}
      </Box>
    );
  }

  return (
    <Grid container spacing={2}>
      {products.map((item) => (
        <Grid size={{ xs: 6, md: 4 }} key={item.id}>
          <Paper elevation={1}>
            <Button
              fullWidth
              onClick={() => handleAdd(item.id)}
              sx={{
                height: 100,
                fontSize: 20,
                fontWeight: 600
              }}
            >
              {item.name}
              <br />
              {item.price.toFixed(2)} €
            </Button>
          </Paper>
        </Grid>
      ))}
    </Grid>
  );
}