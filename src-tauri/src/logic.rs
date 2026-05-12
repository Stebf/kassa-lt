use crate::models::{CartItem, OrderItem};

pub fn normalize_product_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Product name cannot be empty".to_string());
    }

    Ok(trimmed.to_string())
}

pub fn validate_price(price: f64) -> Result<(), String> {
    if price < 0.0 {
        return Err("Price must be greater than or equal to 0".to_string());
    }

    Ok(())
}

pub fn price_to_cents(price: f64) -> i32 {
    (price * 100.0).round() as i32
}

pub fn cart_total_cents(items: &[CartItem]) -> i32 {
    items
        .iter()
        .map(|item| (item.price * item.quantity as f64 * 100.0).round() as i32)
        .sum()
}

pub fn order_items_from_cart(items: &[CartItem]) -> Vec<OrderItem> {
    items
        .iter()
        .map(|item| OrderItem {
            name: item.name.clone(),
            price: item.price,
            quantity: item.quantity,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_product_name_trims_and_rejects_empty_names() {
        assert_eq!(normalize_product_name("  Coffee  ").unwrap(), "Coffee");
        assert_eq!(
            normalize_product_name("   ").unwrap_err(),
            "Product name cannot be empty"
        );
    }

    #[test]
    fn validate_price_rejects_negative_values() {
        assert!(validate_price(1.25).is_ok());
        assert_eq!(
            validate_price(0.0).unwrap_err(),
            "Price must be greater than or equal to 0"
        );
        assert_eq!(
            validate_price(-0.5).unwrap_err(),
            "Price must be greater than or equal to 0"
        );
    }

    #[test]
    fn price_and_cart_totals_round_to_cents() {
        assert_eq!(price_to_cents(1.234), 123);
        assert_eq!(price_to_cents(1.235), 124);

        let items = vec![
            CartItem {
                id: 1,
                name: "Tea".to_string(),
                price: 1.25,
                quantity: 2,
            },
            CartItem {
                id: 2,
                name: "Cake".to_string(),
                price: 2.10,
                quantity: 1,
            },
        ];

        assert_eq!(cart_total_cents(&items), 460);
    }

    #[test]
    fn order_items_are_copied_from_cart_items() {
        let items = vec![CartItem {
            id: 1,
            name: "Water".to_string(),
            price: 0.99,
            quantity: 3,
        }];

        let order_items = order_items_from_cart(&items);

        assert_eq!(order_items.len(), 1);
        assert_eq!(order_items[0].name, "Water");
        assert_eq!(order_items[0].price, 0.99);
        assert_eq!(order_items[0].quantity, 3);
    }
}
