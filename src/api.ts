import { invoke } from "@tauri-apps/api/core";
import type { Product } from "./types/product";
import type { CartItem } from "./types/cart";
import type { Order, ProductSalesCount } from "./types/order";

import type { Category } from "./types/category";
export async function getProducts(): Promise<Product[]> {
  return invoke<Product[]>("get_products");
}

export async function getCategories(): Promise<Category[]> {
  return invoke<Category[]>("get_categories");
}

export async function addProduct(name: string, price: number, category: string = "Default"): Promise<Product> {
  if (!name.trim()) throw new Error("Product name required");
  if (price < 0) throw new Error("Price must be >= 0");
  return invoke<Product>("add_product", { name: name.trim(), price, category });
}

export async function checkout(
  items: CartItem[],
  paymentMethod: "cash" | "card"
): Promise<Order> {
  if (!items.length) throw new Error("Cart empty");
  return invoke<Order>("checkout", {
    items,
    paymentMethod,
  });
}

export async function getOrders(): Promise<Order[]> {
  return invoke<Order[]>("get_orders");
}

export async function getProductSalesCount(): Promise<ProductSalesCount[]> {
  return invoke<ProductSalesCount[]>("get_product_sales_count");
}

export async function getProductById(id: number): Promise<Product> {
  return invoke<Product>("get_product", { id });
}

export async function updateProduct(id: number, name?: string, price?: number, category?: string): Promise<Product> {
  // console.log("Updating product", { id, name, price });
  if (name !== undefined && !name.trim()) throw new Error("Product name cannot be empty");
  if (price !== undefined && price < 0) throw new Error("Price must be >= 0");
  return invoke<Product>("update_product", { id, name: name?.trim(), price, category: category?.trim() });
}

export async function deleteProduct(id: number): Promise<void> {
  return invoke<void>("delete_product", { id });
}

export async function addCategory(name: string): Promise<Category> {
  if (!name.trim()) throw new Error("Category name required");
  return invoke<Category>("add_category", { name: name.trim() });
}

export async function updateCategory(id: number, name: string): Promise<Category> {
  if (!name.trim()) throw new Error("Category name required");
  return invoke<Category>("update_category", { id, name: name.trim() });
}

export async function deleteCategory(id: number): Promise<void> {
  return invoke<void>("delete_category", { id });
}

export async function exportOrders(): Promise<string> {
  return invoke<string>("export_orders_csv", { orders: await getOrders() });
}

export async function exportCSV(csv = "", fileName = "orders.csv"): Promise<void> {
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);

  try {
    const link = document.createElement("a");
    link.href = url;
    link.download = fileName;
    link.style.display = "none";
    document.body.appendChild(link);
    link.click();
    link.remove();
  } finally {
    URL.revokeObjectURL(url);
  }
}
