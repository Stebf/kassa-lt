import {
  Grid,
  Paper,
  Button,
  CircularProgress,
  Box,
  Typography,
  Tabs,
  Tab as MuiTab,
} from "@mui/material";

import { useEffect, useMemo, useState } from "react";
import { useCartStore } from "../store/cartStore";
import { getProducts, getTabs } from "../api";
import type { Product } from "../types/product";
import type { Tab as ProductTab } from "../types/category";
import { getTabVisual } from "../theme/tabColors";

type ProductGridProps = {
  reloadKey?: number;
};

export default function ProductGrid({ reloadKey = 0 }: ProductGridProps) {
  const enqueueAdd = useCartStore((s) => s.enqueueAdd);

  const [products, setProducts] = useState<Product[]>([]);
  const [tabs, setTabs] = useState<ProductTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    load();
  }, [reloadKey]);

  async function load() {
    try {
      const [productData, tabData] = await Promise.all([getProducts(), getTabs()]);
      setProducts(productData);
      setTabs(tabData);
      setLoadError(null);
    } catch {
      setProducts([]);
      setTabs([]);
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

  const tabEntries = useMemo(
    () => tabs.map((tab) => ({ tabId: tab.id, tabName: tab.name, visual: getTabVisual(tab.id) })),
    [tabs],
  );

  useEffect(() => {
    if (tabEntries.length === 0) {
      if (activeTabId !== null) {
        setActiveTabId(null);
      }
      return;
    }

    const exists = tabEntries.some((entry) => entry.tabId === activeTabId);

    if (!exists) {
      setActiveTabId(tabEntries[0].tabId);
    }
  }, [tabEntries, activeTabId]);

  const selectedTabProducts =
    activeTabId === 1
      ? products
      : products.filter((product) => product.tabs.some((tab) => tab.id === activeTabId));

  const selectedProductsByCategory = useMemo(() => {
    const groups = selectedTabProducts.reduce((acc, product) => {
      const categoryName = product.category_name || "Uncategorized";
      const existing = acc.get(categoryName) ?? [];

      existing.push(product);
      acc.set(categoryName, existing);

      return acc;
    }, new Map<string, Product[]>());

    return Array.from(groups.entries()).sort(([left], [right]) => left.localeCompare(right));
  }, [selectedTabProducts]);

  const activeTabVisual = getTabVisual(activeTabId ?? 1);

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
      {tabEntries.length > 0 ? (
        <>
          <Tabs
            value={activeTabId}
            onChange={(_, newValue: number) => setActiveTabId(newValue)}
            variant="scrollable"
            scrollButtons="auto"
            aria-label="Produkttabs"
            sx={{
              minHeight: 48,
              "& .MuiTabs-indicator": {
                backgroundColor: activeTabVisual.backgroundColor,
                height: 3,
                borderRadius: 999,
              },
            }}
          >
            {tabEntries.map((entry) => (
              <MuiTab
                key={entry.tabId}
                value={entry.tabId}
                label={entry.tabName}
                sx={{
                  minHeight: 48,
                  textTransform: "none",
                  fontWeight: 500,
                  color: "text.primary",
                  "&.Mui-selected": {
                    color: activeTabVisual.backgroundColor,
                    fontWeight: 700,
                  },
                }}
              />
            ))}
          </Tabs>

          {selectedProductsByCategory.map(([categoryName, categoryProducts]) => (
            <Box key={categoryName}>
              <Typography variant="h6" sx={{ mb: 1, px: 0.5 }}>
                {categoryName}
              </Typography>
              <Grid container spacing={2}>
                {categoryProducts.map((item) => (
                  <Grid size={{ xs: 6, md: 4 }} key={item.id}>
                    <Paper
                      elevation={1}
                      sx={{
                        overflow: "hidden",
                         border: `1px solid ${activeTabVisual.backgroundColor}22`,
                         backgroundColor: "background.paper",
                      }}
                    >
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
                          color: activeTabVisual.backgroundColor,
                           backgroundColor: "transparent",
                          transition: "transform 160ms ease, opacity 160ms ease",
                          "&:hover": {
                             backgroundColor: `${activeTabVisual.backgroundColor}10`,
                            opacity: 0.94,
                          },
                          border: 4,
                          borderColor: `${activeTabVisual.backgroundColor}60`,
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
        </>
      ) : (
        <Typography sx={{ px: 0.5 }}>Keine Produkte gefunden.</Typography>
      )}
    </Box>
  );
}