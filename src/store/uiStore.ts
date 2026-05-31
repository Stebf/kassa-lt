import { create } from "zustand";

type UiStore = {
  lastActiveTabId: number | null;
  setLastActiveTabId: (id: number | null) => void;
  checkoutComment: string;
  setCheckoutComment: (comment: string) => void;
};

export const useUiStore = create<UiStore>((set) => ({
  lastActiveTabId: null,
  setLastActiveTabId: (id) => set({ lastActiveTabId: id }),
  checkoutComment: "",
  setCheckoutComment: (comment) => set({ checkoutComment: comment }),
}));
