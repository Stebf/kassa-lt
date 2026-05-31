import type { Tab } from "./category";

export type Product = {
  id: number;
  name: string;
  price: number;
  category_id: number;
  category_name: string;
  sales_limit: number | null;
  sales_used: number;
  tabs: Tab[];
};