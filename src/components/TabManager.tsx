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
import type { Tab } from "../types/category";
import { addTab, deleteTab, getTabs, updateTab } from "../api";
import { getTabVisual } from "../theme/tabColors";
import { useUiStore } from "../store/uiStore";

type Props = {
  open: boolean;
  onClose: () => void;
  onChange?: () => void;
};

export default function TabManager({ open, onClose, onChange }: Props) {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [newName, setNewName] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingName, setEditingName] = useState("");
  const [loading, setLoading] = useState(false);
  const [savingId, setSavingId] = useState<number | "new" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const notify = useUiStore((s) => s.notify);

  useEffect(() => {
    if (!open) {
      setEditingId(null);
      setEditingName("");
      setNewName("");
      setError(null);
      return;
    }

    void loadTabs();
  }, [open]);

  async function loadTabs() {
    setLoading(true);
    setError(null);

    try {
      const data = await getTabs();
      setTabs(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Laden der Tabs");
    } finally {
      setLoading(false);
    }
  }

  async function handleAdd() {
    setError(null);
    setSavingId("new");

    try {
      await addTab(newName);
      setNewName("");
      await loadTabs();
      onChange?.();
      notify.success("Tab erfolgreich hinzugefügt");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Hinzufügen des Tabs");
    } finally {
      setSavingId(null);
    }
  }

  async function handleSave(id: number) {
    setError(null);
    setSavingId(id);

    try {
      await updateTab(id, editingName);
      setEditingId(null);
      setEditingName("");
      await loadTabs();
      onChange?.();
      notify.success("Tab erfolgreich aktualisiert");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Aktualisieren des Tabs");
    } finally {
      setSavingId(null);
    }
  }

  async function handleDelete(id: number) {
    if (id === 1) {
      setError("Standard-Tab kann nicht gelöscht werden");
      return;
    }

    const ok = window.confirm("Tab löschen? Produkte werden 'Alle' zugeordnet.");
    if (!ok) {
      return;
    }

    setError(null);
    setSavingId(id);

    try {
      await deleteTab(id);
      await loadTabs();
      onChange?.();
      notify.success("Tab erfolgreich gelöscht");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Fehler beim Löschen des Tabs");
    } finally {
      setSavingId(null);
    }
  }

  function beginEdit(tab: Tab) {
    setEditingId(tab.id);
    setEditingName(tab.name);
    setError(null);
  }

  function cancelEdit() {
    setEditingId(null);
    setEditingName("");
  }

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>Tabs verwalten</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          {error && <Alert severity="error">{error}</Alert>}

          <Stack direction="row" spacing={1}>
            <TextField
              fullWidth
              label="Neuer Tab"
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
              {tabs.map((tab) => (
                <ListItem
                  key={tab.id}
                  divider
                  sx={{ borderLeft: `6px solid ${getTabVisual(tab.id).backgroundColor}`, pl: 1.5 }}
                >
                  {editingId === tab.id ? (
                    <Stack direction="row" spacing={1} sx={{ width: "100%", alignItems: "center" }}>
                      <TextField
                        size="small"
                        fullWidth
                        value={editingName}
                        onChange={(e) => setEditingName(e.target.value)}
                        disabled={savingId === tab.id}
                      />
                      <Button
                        variant="contained"
                        size="small"
                        onClick={() => void handleSave(tab.id)}
                        disabled={savingId === tab.id || !editingName.trim()}
                      >
                        Speichern
                      </Button>
                      <Button
                        variant="text"
                        size="small"
                        onClick={cancelEdit}
                        disabled={savingId === tab.id}
                      >
                        Abbrechen
                      </Button>
                    </Stack>
                  ) : (
                    <>
                      <ListItemText primary={tab.name} secondary={tab.id === 1 ? "Standard" : undefined} />
                      <ListItemSecondaryAction>
                        <IconButton edge="end" onClick={() => beginEdit(tab)} disabled={savingId !== null}>
                          <EditIcon />
                        </IconButton>
                        <IconButton
                          edge="end"
                          onClick={() => void handleDelete(tab.id)}
                          disabled={savingId !== null || tab.id === 1}
                        >
                          <DeleteIcon />
                        </IconButton>
                      </ListItemSecondaryAction>
                    </>
                  )}
                </ListItem>
              ))}
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
