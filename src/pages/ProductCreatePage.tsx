import type { FormEvent } from "react";
import { useState } from "react";
import { Box, Button, TextField, Typography, Alert } from "@mui/material";
import { addProduct } from "../api";

export default function ProductCreatePage() {
  const [name, setName] = useState("");
  const [price, setPrice] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      const priceNum = parseFloat(price);
      await addProduct(name, priceNum);
      setSuccess(`Product "${name}" added`);
      setName("");
      setPrice("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error adding product");
    } finally {
      setLoading(false);
    }
  }

  return (
    <Box sx={{ maxWidth: 400, mx: "auto", mt: 4, p: 2 }}>
      <Typography variant="h5" sx={{ mb: 3 }}>
        Add Product
      </Typography>

      {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}
      {success && <Alert severity="success" sx={{ mb: 2 }}>{success}</Alert>}

      <form onSubmit={handleSubmit}>
        <TextField
          fullWidth
          label="Product Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          disabled={loading}
          sx={{ mb: 2 }}
        />
        <TextField
          fullWidth
          label="Price (€)"
          type="number"
          slotProps={{ htmlInput: { step: "0.01", min: "0" } }}
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          disabled={loading}
          sx={{ mb: 2 }}
        />
        <Button
          fullWidth
          variant="contained"
          type="submit"
          disabled={loading || !name || !price}
        >
          {loading ? "Adding..." : "Add Product"}
        </Button>
      </form>
    </Box>
  );
}
