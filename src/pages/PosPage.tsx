import { Box } from "@mui/material";
import ProductGrid from "../components/ProductGrid";
import CartPanel from "../components/CartPanel";

export default function PosPage() {
  return (
    <Box
      sx={{
        p: 2,
        height: "calc(100dvh - 64px)",
        display: "grid",
        gridTemplateColumns: { xs: "1fr", md: "2fr 1fr" },
        gridTemplateRows: { xs: "minmax(0, 1fr) auto", md: "1fr" },
        gap: 2,
        overflow: "hidden",
      }}
    >
      <Box sx={{ minHeight: 0, overflowY: "auto" }}>
          <ProductGrid />
      </Box>
        <Box sx={{ minHeight: 0, display: "flex", width: "100%", minWidth: 0 }}>
          <CartPanel />
      </Box>
    </Box>
  );
}
