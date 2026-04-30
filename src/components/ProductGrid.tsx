import {
  Grid,
  Paper,
  Button,
  CircularProgress,
  Box,
  Typography,
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

  const productsByCategory = products.reduce((groups, product) => {
    const categoryName = product.category_name || "Uncategorized";
    const existing = groups.get(categoryName) ?? [];

    existing.push(product);
    groups.set(categoryName, existing);

    return groups;
  }, new Map<string, Product[]>());

  const categoryEntries = Array.from(productsByCategory.entries()).sort(([left], [right]) =>
    left.localeCompare(right),
  );

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
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      {categoryEntries.map(([categoryName, categoryProducts]) => (
        <Box key={categoryName}>
          <Typography variant="h6" sx={{ mb: 1, px: 0.5 }}>
            {categoryName}
          </Typography>
          <Grid container spacing={2}>
            {categoryProducts.map((item) => (
              <Grid size={{ xs: 6, md: 4 }} key={item.id}>
                <Paper elevation={1}>
                  <Button
                    fullWidth
                    onClick={() => handleAdd(item.id)}
                    sx={{
                      height: 100,
                      fontSize: 20,
                      fontWeight: 600,
                      display: "flex",
                      flexDirection: "column",
                      alignItems: "center",
                      justifyContent: "center",
                      textTransform: "none",
                    }}
                  >
                    <span>{item.name}</span>
                    <span>{item.price.toFixed(2)} €</span>
                  </Button>
                </Paper>
              </Grid>
            ))}
          </Grid>
        </Box>
      ))}
    </Box>
  );
}