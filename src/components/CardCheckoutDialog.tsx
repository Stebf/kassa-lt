import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    Typography,
    Box,
    Stack
} from "@mui/material";

type Props = {
    open: boolean;
    total: number;
    onClose: () => void;
    onConfirm: () => void;
};

export default function CardCheckoutDialog({
    open,
    total,
    onClose,
    onConfirm
}: Props) {
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
                Kartenzahlung
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

                    <Typography variant="body1" sx={{ whiteSpace: "pre-line" }}>
                        Bitte gib den Betrag in das Kartenterminal ein.
                    </Typography>
                </Stack>
            </DialogContent>

            <DialogActions sx={{ px: 3, py: 2.5, bgcolor: "action.hover" }}>
                <Button onClick={onClose} size="large" sx={{ minWidth: 120 }}>
                    Abbrechen
                </Button>
                <Button
                    variant="contained"
                    size="large"
                    onClick={onConfirm}
                    sx={{ minWidth: 120 }}
                >
                    Bezahlt
                </Button>
            </DialogActions>
        </Dialog>
    );
}
