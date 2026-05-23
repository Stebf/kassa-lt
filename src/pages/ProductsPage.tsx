import { useEffect, useState } from "react";
import { Box, Typography, List, ListItem, ListItemText, IconButton, Divider, Button, Stack } from "@mui/material";
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useNavigate } from "react-router-dom";
import { getProducts, deleteProduct } from "../api";
import type { Product } from "../types/product";
import CategoryManager from "../components/CategoryManager";
import TabManager from "../components/TabManager";

export default function ProductsPage() {
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const [categoriesOpen, setCategoriesOpen] = useState(false);
  const [tabsOpen, setTabsOpen] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    load();
  }, []);

  async function load() {
    setLoading(true);
    try {
      const data = await getProducts();
      setProducts(data);
    } catch {
      setProducts([]);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id: number) {
    const ok = window.confirm("Dieses Produkt wirklich löschen?");
    if (!ok) return;
    try {
      await deleteProduct(id);
      await load();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Löschen fehlgeschlagen");
    }
  }

  return (
    <Box sx={{ width: "80%", maxWidth: "none", mx: "auto", mt: 4, p: 2 }}>
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 1 }}>
        <Typography variant="h5">Produkte</Typography>
      </Box>
      <List>
        {products.map((p) => (
          <div key={p.id}>
            <ListItem
              secondaryAction={
                <>
                  <IconButton edge="end" aria-label="edit" onClick={() => navigate(`/products/${p.id}/edit`)}>
                    <EditIcon />
                  </IconButton>
                  <IconButton edge="end" aria-label="delete" onClick={() => handleDelete(p.id)}>
                    <DeleteIcon />
                  </IconButton>
                </>
              }
            >
              <ListItemText
                primary={p.name}
                secondary={`${p.price.toFixed(2)} € - Kategorie: ${p.category_name}, Tabs: ${p.tabs.map((tab) => tab.name).join(", ")}`}
              />
            </ListItem>
            <Divider />
          </div>
        ))}
        {products.length === 0 && !loading && (
          <ListItem>
            <ListItemText primary="Keine Produkte vorhanden" />
          </ListItem>
        )}
      </List>
      <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
        <Button variant="contained" onClick={() => navigate("/products/add") }>
          Produkt hinzufügen
        </Button>
        <Button variant="contained" color="secondary" onClick={() => setCategoriesOpen(true)}>
          Kategorien verwalten
        </Button>
        <Button variant="contained" color="secondary" onClick={() => setTabsOpen(true)}>
          Tabs verwalten
        </Button>
      </Stack>
      <CategoryManager
        open={categoriesOpen}
        onClose={() => setCategoriesOpen(false)}
        onChange={() => {
          void load();
        }}
      />
      <TabManager
        open={tabsOpen}
        onClose={() => setTabsOpen(false)}
        onChange={() => {
          void load();
        }}
      />
    </Box>
  );
}
