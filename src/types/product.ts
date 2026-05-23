import type { Tab } from "./category";

export type Product = {
  id: number;
  name: string;
  price: number;
  category_id: number;
  category_name: string;
  tabs: Tab[];
};