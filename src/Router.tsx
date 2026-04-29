import { Routes, Route } from "react-router-dom";
import PosPage from "./pages/PosPage";
import DashboardPage from "./pages/DashboardPage";
import ProductCreatePage from "./pages/ProductCreatePage";

export default function Router() {
  return (
    <Routes>
      <Route path="/" element={<PosPage />} />
      <Route path="/dashboard" element={<DashboardPage />} />
      <Route path="/products/new" element={<ProductCreatePage />} />
    </Routes>
  );
}