import {
  Typography,
  Paper,
  Divider,
  Button,
  Stack,
  List,
  ListItem,
  ListItemText
} from "@mui/material";

import DeleteIcon from '@mui/icons-material/Delete';
import PaymentsIcon from '@mui/icons-material/Payments';
import CreditCardIcon from '@mui/icons-material/CreditCard';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';

import { useState } from "react";
import { useCartStore } from "../store/cartStore";
import { checkout } from "../api";
import type { CartItem } from "../types/cart";
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
  const [checkoutDialogOpen, setCheckoutDialogOpen] = useState(false);
  const [checkoutLoading, setCheckoutLoading] = useState(false);
  const [checkoutError, setCheckoutError] = useState<string | null>(null);

  async function handleCheckout(type: "cash" | "card") {
    if (!items.length) return;

    try {
      setCheckoutLoading(true);
      setCheckoutError(null);
      await checkout(items, type);
      // Clear cart after successful checkout
      setItems([]);
      setCheckoutDialogOpen(false);
    } catch (err) {
      console.error("Checkout failed:", err);
      setCheckoutError(getErrorMessage(err));
    } finally {
      setCheckoutLoading(false);
    }
  }

  async function handleCashConfirm() {
    await handleCheckout("cash");
  }

  function clear() {
    setItems([]);
  }

  const total = items.reduce(
    (sum, i) => sum + i.price * i.quantity,
    0
  );

  return (
    <Paper sx={{ p: 2, height: "100%" }}>
      <Typography variant="h5">
        <ShoppingCartIcon /> Warenkorb
      </Typography>

      <List sx={{ minHeight: 300 }}>
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
          onClick={() => setCheckoutDialogOpen(true)}
          disabled={!items.length || checkoutLoading}
          startIcon={<PaymentsIcon />}
        >
          Bar bezahlen
        </Button>

        <Button
          variant="outlined"
          onClick={() => handleCheckout("card")}
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
        open={checkoutDialogOpen}
        total={total}
        onClose={() => setCheckoutDialogOpen(false)}
        onConfirm={handleCashConfirm}
      />
    </Paper>
  );
}
