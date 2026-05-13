import { Routes, Route } from "react-router-dom";
import PosPage from "./pages/PosPage";
import OrdersPage from "./pages/OrdersPage";
import ProductsPage from "./pages/ProductsPage";
import ProductAddPage from "./pages/ProductAddPage";
import ProductEditPage from "./pages/ProductEditPage";

export default function Router() {
  return (
    <Routes>
      <Route path="/" element={<PosPage />} />
      <Route path="/orders" element={<OrdersPage />} />
      <Route path="/products" element={<ProductsPage />} />
      <Route path="/products/add" element={<ProductAddPage />} />
      <Route path="/products/:id/edit" element={<ProductEditPage />} />
    </Routes>
  );
}
