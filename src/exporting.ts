import { save } from "@tauri-apps/api/dialog";
import { writeFile } from "@tauri-apps/api/fs";
import { exportOrders } from "./api";

export async function exportOrdersCSV() {
    const csv = await exportOrders();
    const path = await save({ defaultPath: "orders.csv" });
    if (path) {
    await writeFile({ path, contents: csv });
    }
}