import type { FormEvent } from "react";
import { useState, useEffect } from "react";
import { Box, Button, TextField, Alert, Stack, Select, MenuItem, FormControl, InputLabel } from "@mui/material";
import type { Product } from "../types/product";
import { getCategories, getTabs } from "../api";
import type { Category, Tab } from "../types/category";

type Props = {
  initial?: Product | null;
  onSubmit: (name?: string, price?: number, category?: string, tabIds?: number[], categoryId?: number) => Promise<void>;
  onDelete?: () => Promise<void> | undefined;
  isEdit?: boolean;
};

export default function ProductForm({ initial = null, onSubmit, onDelete, isEdit = false }: Props) {
  const [name, setName] = useState(initial?.name ?? "");
  const [price, setPrice] = useState(initial ? String(initial.price) : "");
  const [categoryId, setCategoryId] = useState(initial?.category_id ?? 1);
  const [tabIds, setTabIds] = useState<number[]>(initial?.tabs.map((tab) => tab.id) ?? [1]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [categoriesLoading, setCategoriesLoading] = useState(true);
  const [tabsLoading, setTabsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const [cats, availableTabs] = await Promise.all([getCategories(), getTabs()]);
        setCategories(cats);
        setTabs(availableTabs);
      } catch {
        setError("Failed to load categories or tabs");
      } finally {
        setCategoriesLoading(false);
        setTabsLoading(false);
      }
    }
    load();
    setName(initial?.name ?? "");
    setPrice(initial ? String(initial.price) : "");
    setCategoryId(initial?.category_id ?? 1);
    setTabIds(initial?.tabs.map((tab) => tab.id) ?? [1]);
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
        const normalizedTabIds = Array.from(new Set(tabIds)).sort((left, right) => left - right);
        const initialTabIds = Array.from(new Set(initial?.tabs.map((tab) => tab.id) ?? [1])).sort(
          (left, right) => left - right,
        );
        const tabChanged =
          normalizedTabIds.length !== initialTabIds.length ||
          normalizedTabIds.some((value, index) => value !== initialTabIds[index]);

        if (!updatedName && !updatedPrice && !categoryChanged && !tabChanged) {
          setError("Mindestens ein Feld muss geändert werden");
          return;
        }

        if (updatedPrice !== undefined && updatedPrice < 0) {
          setError("Preis muss >= 0 sein");
          return;
        }

        await onSubmit(
          updatedName,
          updatedPrice,
          categoryChanged ? selectedCategory : undefined,
          tabChanged ? normalizedTabIds : undefined,
          categoryChanged ? categoryId : undefined,
        );
      } else {
        const trimmedName = name.trim();
        const parsedPrice = parseFloat(price);
        const selectedCategory = categories.find((c) => c.id === categoryId)?.name;
        const normalizedTabIds = Array.from(new Set(tabIds)).sort((left, right) => left - right);

        if (!trimmedName) throw new Error("Produktname benötigt");
        if (!normalizedTabIds.length) throw new Error("Tab benötigt");
        if (parsedPrice < 0) throw new Error("Preis muss >= 0 sein");
        if (!selectedCategory) throw new Error("Kategorie benötigt");

        await onSubmit(trimmedName, parsedPrice, selectedCategory, normalizedTabIds, categoryId);
      }
      setSuccess("Gespeichert");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Speichern des Produkts");
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
      setSuccess("Gelöscht");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Löschen des Produkts");
    } finally {
      setLoading(false);
    }
  }

  const isSubmitDisabled = loading || categoriesLoading || tabsLoading || (!isEdit && (!name || !price));

  return (
    <Box sx={{ maxWidth: 420, mx: "auto", mt: 2, p: 2 }}>
      <form onSubmit={handleSubmit}>
        <Stack spacing={2}>
          {error && <Alert severity="error">{error}</Alert>}
          {success && <Alert severity="success">{success}</Alert>}
          <TextField
            fullWidth
            label="Produktname"
            placeholder={isEdit ? "Beibehalten" : ""}
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={loading}
          />
          <TextField
            fullWidth
            label="Preis (€)"
            type="number"
            placeholder={isEdit ? "Beibehalten" : ""}
            slotProps={{ htmlInput: { step: "0.01", min: "0" } }}
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            disabled={loading}
          />
          <FormControl fullWidth disabled={loading || categoriesLoading}>
            <InputLabel>Kategorie</InputLabel>
            <Select
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value as number)}
              label="Kategorie"
            >
              {categories.map((cat) => (
                <MenuItem key={cat.id} value={cat.id}>
                  {cat.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <FormControl fullWidth disabled={loading || tabsLoading}>
            <InputLabel>Tab</InputLabel>
            <Select
              multiple
              value={tabIds}
              onChange={(e) => setTabIds(e.target.value as number[])}
              label="Tab"
              renderValue={(selected) =>
                tabs
                  .filter((tab) => (selected as number[]).includes(tab.id))
                  .map((tab) => tab.name)
                  .join(", ")
              }
            >
              {tabs.map((tab) => (
                <MenuItem key={tab.id} value={tab.id}>
                  {tab.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <Button variant="contained" type="submit" disabled={isSubmitDisabled}>
            {loading ? "wird gespeichert..." : "Speichern"}
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
