import { Box } from "@mui/material";
import ProductGrid from "../components/ProductGrid";
import CartPanel from "../components/CartPanel";

export default function PosPage() {
  return (
    <Box
      sx={{
        p: 2,
        height: "calc(100vh - 64px)",
        display: "grid",
        gridTemplateColumns: { xs: "1fr", md: "2fr 1fr" },
        gap: 2,
      }}
    >
      <Box>
          <ProductGrid />
      </Box>
      <Box>
          <CartPanel />
      </Box>
    </Box>
  );
}
