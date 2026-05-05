use log::info;

use crate::models::Order;

pub fn create_csv(orders: &[Order]) -> Result<String, String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["Order ID", "Date", "Total (cents)", "Items"])
        .map_err(|e| e.to_string())?;

    for order in orders {
        let items_str = order
            .items
            .iter()
            .map(|item| format!("{} x{}", item.name, item.quantity))
            .collect::<Vec<String>>()
            .join("; ");
        wtr.write_record(&[
            order.id.to_string(),
            order.created_at.clone(),
            order.total.to_string(),
            items_str,
        ])
        .map_err(|e| e.to_string())?;
    }

    let data = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_orders_csv(orders: Vec<Order>) -> Result<String, String> {
    info!("Exporting {} orders to CSV", orders.len());
    create_csv(&orders)
}
