import { useEffect, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  IconButton,
  List,
  ListItem,
  ListItemSecondaryAction,
  ListItemText,
  Stack,
  TextField,
} from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import type { Category } from "../types/category";
import { addCategory, deleteCategory, getCategories, updateCategory } from "../api";

type Props = {
  open: boolean;
  onClose: () => void;
  onChange?: () => void;
};

export default function CategoryManager({ open, onClose, onChange }: Props) {
  const [categories, setCategories] = useState<Category[]>([]);
  const [newName, setNewName] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingName, setEditingName] = useState("");
  const [loading, setLoading] = useState(false);
  const [savingId, setSavingId] = useState<number | "new" | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setEditingId(null);
      setEditingName("");
      setNewName("");
      setError(null);
      return;
    }

    void loadCategories();
  }, [open]);

  async function loadCategories() {
    setLoading(true);
    setError(null);

    try {
      const data = await getCategories();
      setCategories(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load categories");
    } finally {
      setLoading(false);
    }
  }

  async function handleAdd() {
    setError(null);
    setSavingId("new");

    try {
      await addCategory(newName);
      setNewName("");
      await loadCategories();
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add category");
    } finally {
      setSavingId(null);
    }
  }

  async function handleSave(id: number) {
    setError(null);
    setSavingId(id);

    try {
      await updateCategory(id, editingName);
      setEditingId(null);
      setEditingName("");
      await loadCategories();
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update category");
    } finally {
      setSavingId(null);
    }
  }

  async function handleDelete(id: number) {
    if (id === 1) {
      setError("Default category cannot be deleted");
      return;
    }

    const ok = window.confirm("Delete this category? Products will move to Default.");
    if (!ok) {
      return;
    }

    setError(null);
    setSavingId(id);

    try {
      await deleteCategory(id);
      await loadCategories();
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete category");
    } finally {
      setSavingId(null);
    }
  }

  function beginEdit(category: Category) {
    setEditingId(category.id);
    setEditingName(category.name);
    setError(null);
  }

  function cancelEdit() {
    setEditingId(null);
    setEditingName("");
  }

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>Kategorien verwalten</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          {error && <Alert severity="error">{error}</Alert>}

          <Stack direction="row" spacing={1}>
            <TextField
              fullWidth
              label="Neue Kategorie"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              disabled={loading || savingId === "new"}
            />
            <Button
              variant="contained"
              onClick={handleAdd}
              disabled={loading || savingId === "new" || !newName.trim()}
            >
              Hinzufügen
            </Button>
          </Stack>

          <Divider />

          <Box sx={{ maxHeight: 360, overflowY: "auto" }}>
            <List disablePadding>
              {categories.map((category) => (
                <ListItem key={category.id} divider>
                  {editingId === category.id ? (
                    <Stack direction="row" spacing={1} sx={{ width: "100%" }}>
                      <TextField
                        fullWidth
                        label="Kategorie"
                        value={editingName}
                        onChange={(e) => setEditingName(e.target.value)}
                        disabled={savingId === category.id}
                      />
                      <Button
                        variant="contained"
                        onClick={() => void handleSave(category.id)}
                        disabled={savingId === category.id || !editingName.trim()}
                      >
                        Speichern
                      </Button>
                      <Button
                        variant="text"
                        onClick={cancelEdit}
                        disabled={savingId === category.id}
                      >
                        Abbrechen
                      </Button>
                    </Stack>
                  ) : (
                    <>
                      <ListItemText
                        primary={category.name}
                        secondary={category.id === 1 ? "Standardkategorie" : undefined}
                      />
                      <ListItemSecondaryAction>
                        <IconButton edge="end" aria-label="edit" onClick={() => beginEdit(category)} disabled={loading}>
                          <EditIcon />
                        </IconButton>
                        <IconButton
                          edge="end"
                          aria-label="delete"
                          onClick={() => void handleDelete(category.id)}
                          disabled={loading || category.id === 1}
                        >
                          <DeleteIcon />
                        </IconButton>
                      </ListItemSecondaryAction>
                    </>
                  )}
                </ListItem>
              ))}
              {categories.length === 0 && !loading && (
                <ListItem>
                  <ListItemText primary="No categories" />
                </ListItem>
              )}
            </List>
          </Box>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Schließen</Button>
      </DialogActions>
    </Dialog>
  );
}