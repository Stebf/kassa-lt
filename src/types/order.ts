export type OrderItem = {
  name: string;
  price: number;
  quantity: number;
};

export type Order = {
  id: number;
  uuid: string;
  created_at: string;
  total: number;
  payment_method: "cash" | "card";
  items: OrderItem[];
};

export type ProductSalesCount = {
  product_name: string;
  count: number;
};
