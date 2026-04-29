import { create } from "zustand";
import type { CartItem } from "../types/cart";
import type { Product } from "../types/product";

type CartStore = {
  items: CartItem[];
  setItems: (items: CartItem[]) => void;
  enqueueAdd: (product: Product) => void;
  enqueueRemove: (item: CartItem) => void;
};

export const useCartStore = create<CartStore>((set) => ({
  items: [],

  setItems: (items) => set({ items }),

  enqueueAdd: (product) => {
    set((state) => {
      const existing = state.items.find((item) => item.id === product.id);
      
      if (existing) {
        return {
          items: state.items.map((item) =>
            item.id === product.id
              ? { ...item, quantity: item.quantity + 1 }
              : item
          )
        };
      }

      return {
        items: [
          ...state.items,
          {
            id: product.id,
            name: product.name,
            price: product.price,
            quantity: 1
          }
        ]
      };
    });
  },

  enqueueRemove: (item) => {
    set((state) => {
      const existing = state.items.find((entry) => entry.id === item.id);
      
      if (!existing) return state;
      
      if (existing.quantity <= 1) {
        return {
          items: state.items.filter((entry) => entry.id !== item.id)
        };
      }

      return {
        items: state.items.map((entry) =>
          entry.id === item.id
            ? { ...entry, quantity: entry.quantity - 1 }
            : entry
        )
      };
    });
  }
}));