import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  TextField,
  Stack,
  Box
} from "@mui/material";
import { useEffect, useRef, useState } from "react";

type Props = {
  open: boolean;
  total: number;
  onClose: () => void;
  onConfirm: (amountReceived: number) => void;
};

export default function CheckoutDialog({
  open,
  total,
  onClose,
  onConfirm
}: Props) {
  const [amount, setAmount] = useState("");
  const amountInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    requestAnimationFrame(() => {
      amountInputRef.current?.focus();
      amountInputRef.current?.select();
    });
  }, [open]);

  const received = Number(amount || 0);
  const change = received - total;
  const canPay = received >= total;

  const quickAmounts = [1, 2, 5, 10, 20, 50];
  const keypadButtonSx = {
    minHeight: 44,
    fontSize: 14,
    fontWeight: 600,
    px: 1.5,
    py: 0.75
  };

  function handleConfirm() {
    onConfirm(received);
    setAmount("");
  }

  function addAmount(amountToAdd: number) {
    const current = Number(amount || 0);
    setAmount(String(current + amountToAdd));
  }

  function clearAmount() {
    setAmount("0");
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      fullWidth
      maxWidth="sm"
      slotProps={{
        paper: {
          sx: {
            borderRadius: 3,
            overflow: "hidden"
          }
        }
      }}
    >
      <DialogTitle sx={{ pb: 1.5, fontWeight: 700 }}>
        Barzahlung
      </DialogTitle>

      <DialogContent sx={{ pt: 1, pb: 2.5 }}>
        <Stack spacing={2.5} sx={{ mt: 0.5 }}>
          <Box
            sx={{
              borderRadius: 2,
              px: 2,
              py: 1.5,
              bgcolor: "action.hover",
              border: 1,
              borderColor: "divider"
            }}
          >
            <Typography variant="overline" color="text.secondary">
              Zu zahlen
            </Typography>
            <Typography variant="h4" sx={{ lineHeight: 1.1, fontWeight: 700 }}>
              {total.toFixed(2)} €
            </Typography>
          </Box>

          <TextField
            label="Erhalten"
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            fullWidth
            inputRef={amountInputRef}
            slotProps={{
              htmlInput: {
                style: {
                  fontSize: 28,
                  padding: 16
                }
              }
            }}
          />

          <Box
            sx={{
              border: 1,
              borderColor: "divider",
              borderRadius: 2,
              p: 2,
              bgcolor: "background.paper"
            }}
          >
            <Stack spacing={1.5}>
              <Box
                sx={{
                  display: "grid",
                  gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
                  gap: 0.75
                }}
              >
                <Button
                  variant="outlined"
                  color="error"
                  onClick={clearAmount}
                  sx={keypadButtonSx}
                >
                  Löschen
                </Button>

                {quickAmounts.map((value) => (
                  <Button
                    key={`add-${value}`}
                    variant="outlined"
                    onClick={() => addAmount(value)}
                    sx={keypadButtonSx}
                  >
                    +{value} €
                  </Button>
                ))}
              </Box>

              <Box
                sx={{
                  display: "grid",
                  gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
                  gap: 0.75
                }}
              >
                {quickAmounts.map((value) => (
                  <Button
                    key={`sub-${value}`}
                    variant="outlined"
                    onClick={() => addAmount(-value)}
                    sx={keypadButtonSx}
                  >
                    -{value} €
                  </Button>
                ))}
              </Box>
            </Stack>
          </Box>

          <Box
            sx={{
              borderRadius: 2,
              px: 2,
              py: 1.5,
              bgcolor: "action.hover",
              border: 1,
              borderColor: "divider"
            }}
          >
            <Typography
              variant="overline"
              color="text.secondary"
              sx={{ letterSpacing: 1 }}
            >
              Rückgeld
            </Typography>
            <Typography variant="h4" sx={{ lineHeight: 1.1, fontWeight: 700 }}>
              {change > 0 ? change.toFixed(2) : "0.00"} €
            </Typography>
          </Box>
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, py: 2.5, bgcolor: "action.hover" }}>
        <Button onClick={onClose} size="large" sx={{ minWidth: 120 }}>
          Abbrechen
        </Button>

        <Button
          variant="contained"
          size="large"
          disabled={!canPay}
          onClick={handleConfirm}
          sx={{ minWidth: 120 }}
        >
          Bezahlen
        </Button>
      </DialogActions>
    </Dialog>
  );
}