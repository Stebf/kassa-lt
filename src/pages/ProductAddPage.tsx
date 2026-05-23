import { useNavigate } from "react-router-dom";
import { Box, Typography, Button } from "@mui/material";
import ProductForm from "../components/ProductForm";
import { addProduct } from "../api";

export default function ProductAddPage() {
  const navigate = useNavigate();

  async function handleSubmit(name?: string, price?: number, category?: string, tabIds?: number[]) {
    if (!name || price === undefined) {
      throw new Error("Name und Preis benötigt");
    }

    await addProduct(name, price, category, tabIds);
    navigate("/products");
  }

  return (
    <Box sx={{ maxWidth: 400, mx: "auto", mt: 4, p: 2 }}>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Produkt hinzufügen
      </Typography>
      <ProductForm onSubmit={handleSubmit} />
      <Button sx={{ mt: 2 }} onClick={() => navigate("/products")}>
        Abbrechen
      </Button>
    </Box>
  );
}
