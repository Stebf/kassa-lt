import type { FormEvent } from "react";
import { useState, useEffect } from "react";
import { Box, Button, TextField, Alert, Stack, Select, MenuItem, FormControl, InputLabel } from "@mui/material";
import type { Product } from "../types/product";
import { getCategories } from "../api";
import type { Category } from "../types/category";

type Props = {
  initial?: Product | null;
  onSubmit: (name?: string, price?: number, category?: string, categoryId?: number) => Promise<void>;
  onDelete?: () => Promise<void> | undefined;
  isEdit?: boolean;
};

export default function ProductForm({ initial = null, onSubmit, onDelete, isEdit = false }: Props) {
  const [name, setName] = useState(initial?.name ?? "");
  const [price, setPrice] = useState(initial ? String(initial.price) : "");
  const [categoryId, setCategoryId] = useState(initial?.category_id ?? 1);
  const [categories, setCategories] = useState<Category[]>([]);
  const [categoriesLoading, setCategoriesLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const cats = await getCategories();
        setCategories(cats);
      } catch {
        setError("Failed to load categories");
      } finally {
        setCategoriesLoading(false);
      }
    }
    load();
    setName(initial?.name ?? "");
    setPrice(initial ? String(initial.price) : "");
    setCategoryId(initial?.category_id ?? 1);
  }, [initial]);

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      if (isEdit) {
        const updatedName = name.trim() || undefined;
        const updatedPrice = price ? parseFloat(price) : undefined;
        const selectedCategory = categories.find((c) => c.id === categoryId)?.name;
        const categoryChanged = categoryId !== (initial?.category_id ?? 1);

        if (!updatedName && !updatedPrice && !categoryChanged) {
          setError("Enter at least one field to update");
          return;
        }

        if (updatedPrice !== undefined && updatedPrice <= 0) {
          setError("Price must be > 0");
          return;
        }

        await onSubmit(
          updatedName,
          updatedPrice,
          categoryChanged ? selectedCategory : undefined,
          categoryChanged ? categoryId : undefined,
        );
      } else {
        const trimmedName = name.trim();
        const parsedPrice = parseFloat(price);
        const selectedCategory = categories.find((c) => c.id === categoryId)?.name;

        if (!trimmedName) throw new Error("Product name required");
        if (parsedPrice <= 0) throw new Error("Price must be > 0");
        if (!selectedCategory) throw new Error("Category required");

        await onSubmit(trimmedName, parsedPrice, selectedCategory, categoryId);
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

  const isSubmitDisabled = loading || categoriesLoading || (!isEdit && (!name || !price));

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
          <FormControl fullWidth disabled={loading || categoriesLoading}>
            <InputLabel>Category</InputLabel>
            <Select
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value as number)}
              label="Category"
            >
              {categories.map((cat) => (
                <MenuItem key={cat.id} value={cat.id}>
                  {cat.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <Button variant="contained" type="submit" disabled={isSubmitDisabled}>
            {loading ? "Saving..." : "Speichern"}
          </Button>
          {onDelete && (
            <Button color="error" variant="outlined" onClick={handleDelete} disabled={loading}>
              Löschen
            </Button>
          )}
        </Stack>
      </form>
    </Box>
  );
}
