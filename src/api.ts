import { invoke } from "@tauri-apps/api/core";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { Product } from "./types/product";
import type { CartItem } from "./types/cart";
import type { Order, ProductSalesCount } from "./types/order";

import type { Category, Tab } from "./types/category";
import type { BackupWorkerConfig, SyncWorkerConfig } from "./types/config";
import type { BackupState } from "./types/backup";

export async function checkout(
  items: CartItem[],
  paymentMethod: "cash" | "card",
  comment: string = ""
): Promise<Order> {
  if (!items.length) throw new Error("Cart empty");
  return invoke<Order>("checkout", {
    items,
    paymentMethod,
    comment,
  });
}

export async function getOrders(): Promise<Order[]> {
  return invoke<Order[]>("get_orders");
}

// Products
export async function addProduct(
  name: string,
  price: number,
  category: string = "Default",
  tabIds: number[] = [1],
  salesLimit?: number | null,
): Promise<Product> {
  if (!name.trim()) throw new Error("Product name required");
  if (price < 0) throw new Error("Price must be >= 0");
  return invoke<Product>("add_product", { name: name.trim(), price, category, tabIds, salesLimit });
}

export async function getProducts(): Promise<Product[]> {
  return invoke<Product[]>("get_products");
}

export async function getProductSalesCount(): Promise<ProductSalesCount[]> {
  return invoke<ProductSalesCount[]>("get_product_sales_count");
}

export async function getProductById(id: number): Promise<Product> {
  return invoke<Product>("get_product", { id });
}

export async function updateProduct(
  id: number,
  name?: string,
  price?: number,
  category?: string,
  tabIds?: number[],
  salesLimit?: number | null,
): Promise<Product> {
  // console.log("Updating product", { id, name, price });
  if (name !== undefined && !name.trim()) throw new Error("Product name cannot be empty");
  if (price !== undefined && price < 0) throw new Error("Price must be >= 0");
  const payload = {
    id,
    name: name?.trim(),
    price,
    category: category?.trim(),
    tabIds,
    salesLimit,
    salesLimitChanged: salesLimit !== undefined,
  };
  const res = await invoke<Product>("update_product", payload);
  return res;
}

export async function deleteProduct(id: number): Promise<void> {
  return invoke<void>("delete_product", { id });
}

// Categories
export async function addCategory(name: string): Promise<Category> {
  if (!name.trim()) throw new Error("Category name required");
  return invoke<Category>("add_category", { name: name.trim() });
}

export async function getCategories(): Promise<Category[]> {
  return invoke<Category[]>("get_categories");
}

export async function updateCategory(id: number, name: string): Promise<Category> {
  if (!name.trim()) throw new Error("Category name required");
  return invoke<Category>("update_category", { id, name: name.trim() });
}

export async function deleteCategory(id: number): Promise<void> {
  return invoke<void>("delete_category", { id });
}

// Tabs
export async function getTabs(): Promise<Tab[]> {
  return invoke<Tab[]>("get_tabs");
}

export async function addTab(name: string): Promise<Tab> {
  if (!name.trim()) throw new Error("Tab name required");
  return invoke<Tab>("add_tab", { name: name.trim() });
}

export async function updateTab(id: number, name: string): Promise<Tab> {
  if (!name.trim()) throw new Error("Tab name required");
  return invoke<Tab>("update_tab", { id, name: name.trim() });
}

export async function deleteTab(id: number): Promise<void> {
  return invoke<void>("delete_tab", { id });
}


// Exporting
export async function exportOrders(): Promise<string> {
  return invoke<string>("export_orders_csv", { orders: await getOrders() });
}

export async function exportCSV(csv = "", path = "orders.csv"): Promise<void> {
  console.log("Exporting CSV to", path);
  try {
    await writeTextFile(path, csv);
    console.log("CSV export successful");
  } catch (error) {
    console.error("CSV export failed", error);
    throw error;
  }
}

// Backup
export async function setBackupConfig(config: BackupWorkerConfig): Promise<void> {
  await invoke<void>("set_backup_config", { config });
}

export async function getBackupConfig(): Promise<BackupWorkerConfig> {
  return invoke<BackupWorkerConfig>("get_backup_config");
}

export async function runBackupNow(): Promise<BackupState> {
  return invoke<BackupState>("run_backup_now");
}

// Sync
export async function getSyncConfig(): Promise<SyncWorkerConfig> {
  return invoke<SyncWorkerConfig>("get_sync_config");
}

export async function setSyncConfig(config: SyncWorkerConfig): Promise<void> {
  await invoke<void>("set_sync_config", { config });
}