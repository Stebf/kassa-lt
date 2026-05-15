import { Routes, Route } from "react-router-dom";
import PosPage from "./pages/PosPage";
import OrdersPage from "./pages/OrdersPage";
import ProductsPage from "./pages/ProductsPage";
import ProductAddPage from "./pages/ProductAddPage";
import ProductEditPage from "./pages/ProductEditPage";
import SettingsPage from "./pages/SettingsPage";
import { useAdmin } from "./context/AdminContext";

export default function Router() {
  const { adminModeEnabled } = useAdmin();
  return (
    <Routes>
      <Route path="/" element={<PosPage />} />
      <Route path="/orders" element={<OrdersPage />} />
      {adminModeEnabled && (
        <>
          <Route path="/products" element={<ProductsPage />} />
          <Route path="/products/add" element={<ProductAddPage />} />
          <Route path="/products/:id/edit" element={<ProductEditPage />} />
        </>
      )}
      <Route path="/settings" element={<SettingsPage />} />
    </Routes>
  );
}
