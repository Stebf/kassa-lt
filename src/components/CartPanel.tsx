import {
  Typography,
  Paper,
  Divider,
  Button,
  Stack,
  List,
  ListItem,
  ListItemText,
  TextField
} from "@mui/material";

import DeleteIcon from '@mui/icons-material/Delete';
import PaymentsIcon from '@mui/icons-material/Payments';
import CreditCardIcon from '@mui/icons-material/CreditCard';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';

import { useState } from "react";
import { useCartStore } from "../store/cartStore";
import { useUiStore } from "../store/uiStore";
import { checkout, getProducts } from "../api";
import type { CartItem } from "../types/cart";
import CardCheckoutDialog from "./CardCheckoutDialog";
import CashCheckoutDialog from "./CashCheckoutDialog";

function getErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) {
      return message;
    }
  }

  return "Unknown error";
}

export default function CartPanel() {
  const items = useCartStore((s) => s.items);
  const enqueueRemove = useCartStore((s) => s.enqueueRemove);
  const setItems = useCartStore((s) => s.setItems);
  const bumpProductsReloadKey = useUiStore((s) => s.bumpProductsReloadKey);
  const [openedCheckoutDialog, setOpenedCheckoutDialog] = useState<"cash" | "card" | null>(null);
  const [checkoutLoading, setCheckoutLoading] = useState(false);
  const [checkoutError, setCheckoutError] = useState<string | null>(null);
  const checkoutComment = useUiStore((s) => s.checkoutComment);
  const setCheckoutComment = useUiStore((s) => s.setCheckoutComment);

  async function handleCheckout(
    type: "cash" | "card",
    onSuccess?: () => void
  ) {
    if (!items.length) return;

    try {
      setCheckoutLoading(true);
      setCheckoutError(null);
      await checkout(items, type, checkoutComment.trim());
      setItems([]);
      // refresh products so sales_used changes are reflected in the grid
      bumpProductsReloadKey();
      setCheckoutComment("");
      onSuccess?.();
    } catch (err) {
      console.error("Checkout failed:", err);
      setCheckoutError(getErrorMessage(err));
    } finally {
      setCheckoutLoading(false);
    }
  }

  async function checkQuotas(): Promise<string | null> {
    if (!items.length) return null;

    try {
      const products = await getProducts();
      const byId = new Map<number, typeof products[0]>();
      for (const p of products) byId.set(p.id, p);

      const violations: string[] = [];

      for (const item of items) {
        const prod = byId.get(item.id);
        if (!prod) continue;

        if (prod.sales_limit !== null) {
          const projected = prod.sales_used + item.quantity;
          if (projected > prod.sales_limit) {
            violations.push(`${prod.name} (Limit: ${prod.sales_limit}, aktuell: ${prod.sales_used}, im Warenkorb: ${item.quantity})`);
          }
        }
      }

      if (violations.length) {
        return `Folgende Produkte überschreiten das Limit: ${violations.join(", ")}`;
      }

      return null;
    } catch (err) {
      console.error("Failed to check product quotas:", err);
      return "Fehler beim Überprüfen der Produktlimits.";
    }
  }

  async function handleCashConfirm() {
    await handleCheckout("cash", () => setOpenedCheckoutDialog(null));
  }

  async function handleCardConfirm() {
    await handleCheckout("card", () => setOpenedCheckoutDialog(null));
  }

  function clear() {
    setItems([]);
  }

  const total = items.reduce(
    (sum, i) => sum + i.price * i.quantity,
    0
  );

  return (
    <Paper sx={{ p: 2, width: "100%", height: "100%", display: "flex", flexDirection: "column", minHeight: 0, pb: 'calc(env(safe-area-inset-bottom, 0px) + 16px)' }}>
      <Typography variant="h5">
        <ShoppingCartIcon /> Warenkorb
      </Typography>

      <List sx={{ flex: 1, minHeight: 0, overflowY: "auto" }}>
        {items.map((item: CartItem) => (
          <ListItem key={item.id}>
            <ListItemText
              primary={`${item.name} x${item.quantity}`}
              secondary={`${(
                item.price * item.quantity
              ).toFixed(2)} €`}
            />
            <Button
              size="small"
              variant="text"
              onClick={() => enqueueRemove(item)}
            >
              <DeleteIcon />
            </Button>
          </ListItem>
        ))}
      </List>

      <TextField
        id="checkout-notes"
        label="Anmerkungen zur Bestellung"
        multiline
        rows={2}
        fullWidth
        value={checkoutComment}
        onChange={(e) => setCheckoutComment(e.target.value)}
      />

      <Divider sx={{ my: 2 }} />

      {checkoutError ? (
        <Typography variant="body2" color="error" sx={{ mb: 1 }}>
          Checkout fehlgeschlagen: {checkoutError}
        </Typography>
      ) : null}

      <Typography variant="h4">
        {total.toFixed(2)} €
      </Typography>

      <Stack spacing={2} sx={{ mt: 2 }}>
        <Button
          variant="contained"
          onClick={async () => {
            setCheckoutError(null);
            const err = await checkQuotas();
            if (err) {
              setCheckoutError(err);
              return;
            }
            setOpenedCheckoutDialog("cash");
          }}
          disabled={!items.length || checkoutLoading}
          startIcon={<PaymentsIcon />}
        >
          Bar bezahlen
        </Button>

        <Button
          variant="outlined"
          onClick={async () => {
            setCheckoutError(null);
            const err = await checkQuotas();
            if (err) {
              setCheckoutError(err);
              return;
            }
            setOpenedCheckoutDialog("card");
          }}
          disabled={!items.length || checkoutLoading}
          startIcon={<CreditCardIcon />}
        >
          Mit Karte bezahlen
        </Button>

        <Button
          color="error"
          onClick={() => clear()}
          disabled={!items.length}
        >
          Leeren
        </Button>
      </Stack>

      <CashCheckoutDialog
        open={openedCheckoutDialog === "cash"}
        total={total}
        onClose={() => setOpenedCheckoutDialog(null)}
        onConfirm={handleCashConfirm}
      />

      <CardCheckoutDialog
        open={openedCheckoutDialog === "card"}
        total={total}
        onClose={() => setOpenedCheckoutDialog(null)}
        onConfirm={handleCardConfirm}
      />
    </Paper>
  );
}
