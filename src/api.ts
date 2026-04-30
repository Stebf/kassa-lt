import { invoke } from "@tauri-apps/api/core";
import type { Product } from "./types/product";
import type { CartItem } from "./types/cart";
import type { Order } from "./types/order";

export async function getProducts(): Promise<Product[]> {
  return invoke<Product[]>("get_products");
}

export async function addProduct(name: string, price: number): Promise<Product> {
  if (!name.trim()) throw new Error("Product name required");
  if (price <= 0) throw new Error("Price must be > 0");
  return invoke<Product>("add_product", { name: name.trim(), price });
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

export async function getProductById(id: number): Promise<Product> {
  return invoke<Product>("get_product", { id });
}

export async function updateProduct(id: number, name?: string, price?: number): Promise<Product> {
  // console.log("Updating product", { id, name, price });
  if (name !== undefined && !name.trim()) throw new Error("Product name cannot be empty");
  if (price !== undefined && price <= 0) throw new Error("Price must be > 0");
  return invoke<Product>("update_product", { id, name: name?.trim(), price });
}

export async function deleteProduct(id: number): Promise<void> {
  return invoke<void>("delete_product", { id });
}
