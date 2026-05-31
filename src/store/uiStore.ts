import { create } from "zustand";

type UiStore = {
  lastActiveTabId: number | null;
  setLastActiveTabId: (id: number | null) => void;
  checkoutComment: string;
  setCheckoutComment: (comment: string) => void;
  productsReloadKey: number;
  bumpProductsReloadKey: () => void;
};

export const useUiStore = create<UiStore>((set) => ({
  lastActiveTabId: null,
  setLastActiveTabId: (id) => set({ lastActiveTabId: id }),
  checkoutComment: "",
  setCheckoutComment: (comment) => set({ checkoutComment: comment }),
  productsReloadKey: 0,
  bumpProductsReloadKey: () => set((s) => ({ productsReloadKey: s.productsReloadKey + 1 })),
}));
