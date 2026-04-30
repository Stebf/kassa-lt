import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import type { Product } from "../types/product";
import { getProductById, updateProduct } from "../api";
import ProductForm from "../components/ProductForm";
import { Box, Typography, Button } from "@mui/material";

export default function ProductEditPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [product, setProduct] = useState<Product | null>(null);

  useEffect(() => {
    if (!id) return;
    getProductById(Number(id))
      .then((p) => setProduct(p))
      .catch(() => setProduct(null));
  }, [id]);

  async function handleSubmit(name?: string, price?: number, category?: string) {
    if (!product) return;
    await updateProduct(product.id, name, price, category || undefined);
    navigate("/products");
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
