import type { FormEvent } from "react";
import { useState, useEffect } from "react";
import { Box, Button, TextField, Alert, Stack } from "@mui/material";
import type { Product } from "../types/product";

type Props = {
  initial?: Product | null;
  onSubmit: (name?: string, price?: number) => Promise<void>;
  onDelete?: () => Promise<void> | undefined;
  isEdit?: boolean;
};

export default function ProductForm({ initial = null, onSubmit, onDelete, isEdit = false }: Props) {
  const [name, setName] = useState(initial?.name ?? "");
  const [price, setPrice] = useState(initial ? String(initial.price) : "");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setName(initial?.name ?? "");
    setPrice(initial ? String(initial.price) : "");
  }, [initial]);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      if (isEdit) {
        // Edit mode: allow optional fields
        const updatedName = name.trim() || undefined;
        const updatedPrice = price ? parseFloat(price) : undefined;
        
        if (!updatedName && !updatedPrice) {
          setError("Enter at least one field to update");
          setLoading(false);
          return;
        }
        
        if (updatedPrice !== undefined && updatedPrice <= 0) {
          setError("Price must be > 0");
          setLoading(false);
          return;
        }
        
        await onSubmit(updatedName, updatedPrice);
      } else {
        // Create mode: require both fields
        const trimmedName = name.trim();
        const parsedPrice = parseFloat(price);
        
        if (!trimmedName) throw new Error("Product name required");
        if (parsedPrice <= 0) throw new Error("Price must be > 0");
        
        await onSubmit(trimmedName, parsedPrice);
      }
      setSuccess("Saved");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error saving product");
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete() {
    if (!onDelete) return;
    setError(null);
    setSuccess(null);
    setLoading(true);
    try {
      await onDelete();
      setSuccess("Deleted");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error deleting product");
    } finally {
      setLoading(false);
    }
  }

  const isSubmitDisabled = loading || (!isEdit && (!name || !price));

  return (
    <Box sx={{ maxWidth: 420, mx: "auto", mt: 2, p: 2 }}>
      <form onSubmit={handleSubmit}>
        <Stack spacing={2}>
          {error && <Alert severity="error">{error}</Alert>}
          {success && <Alert severity="success">{success}</Alert>}
          <TextField
            fullWidth
            label="Product Name"
            placeholder={isEdit ? "Leave empty to keep current" : ""}
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={loading}
          />
          <TextField
            fullWidth
            label="Price (€)"
            type="number"
            placeholder={isEdit ? "Leave empty to keep current" : ""}
            slotProps={{ htmlInput: { step: "0.01", min: "0" } }}
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            disabled={loading}
          />
          <Button variant="contained" type="submit" disabled={isSubmitDisabled}>
            {loading ? "Saving..." : "Save"}
          </Button>
          {onDelete && (
            <Button color="error" variant="outlined" onClick={handleDelete} disabled={loading}>
              Delete
            </Button>
          )}
        </Stack>
      </form>
    </Box>
  );
}
