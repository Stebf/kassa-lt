import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { exportOrders } from "./api";

export async function exportOrdersCSV() {
  const csv = await exportOrders();
  const path = await save({ defaultPath: "orders.csv" });
  if (path) {
    await writeTextFile(path, csv);
  }
}
