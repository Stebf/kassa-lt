import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import type { Product } from "../types/product";
import { getProductById, updateProduct } from "../api";
import ProductForm from "../components/ProductForm";
import { Box, Typography, Button } from "@mui/material";
import { useUiStore } from "../store/uiStore";

export default function ProductEditPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [product, setProduct] = useState<Product | null>(null);
  const notify = useUiStore((s) => s.notify);

  useEffect(() => {
    if (!id) return;
    getProductById(Number(id))
      .then((p) => setProduct(p))
      .catch(() => setProduct(null));
  }, [id]);

  async function handleSubmit(name?: string, price?: number, category?: string, tabIds?: number[], _categoryId?: number, salesLimit?: number | null) {
    if (!product) return;
    await updateProduct(product.id, name, price, category || undefined, tabIds, salesLimit);
    navigate("/products");
    notify.success("Produkt erfolgreich aktualisiert");
  }

  return (
    <Box sx={{ maxWidth: 640, mx: "auto", mt: 4, p: 2 }}>
      <Typography variant="h5" sx={{ mb: 2 }}>
        Produkt bearbeiten
      </Typography>
      <ProductForm initial={product} onSubmit={handleSubmit} isEdit={true} />
      <Button sx={{ mt: 2 }} onClick={() => navigate("/products")}>
        Abbrechen
      </Button>
    </Box>
  );
}
