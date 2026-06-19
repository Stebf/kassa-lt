import { create } from "zustand";

export type UiNotificationSeverity = "success" | "info" | "warning" | "error";

export type UiNotificationApi = {
  show: (message: string, severity?: UiNotificationSeverity) => void;
  success: (message: string) => void;
  info: (message: string) => void;
  warning: (message: string) => void;
  error: (message: string) => void;
};

type UiNotification = {
  key: number;
  message: string;
  severity: UiNotificationSeverity;
};

type UiStoreSet = {
  (partial: UiStore | Partial<UiStore> | ((state: UiStore) => UiStore | Partial<UiStore>), replace?: false): void;
  (state: UiStore | ((state: UiStore) => UiStore), replace: true): void;
};

type UiStore = {
  lastActiveTabId: number | null;
  setLastActiveTabId: (id: number | null) => void;
  checkoutComment: string;
  setCheckoutComment: (comment: string) => void;
  productsReloadKey: number;
  bumpProductsReloadKey: () => void;
  notification: UiNotification | null;
  showNotification: (message: string, severity?: UiNotificationSeverity) => void;
  notify: UiNotificationApi;
  hideNotification: () => void;
};

let notificationKey = 0;

function createNotificationApi(set: UiStoreSet): UiNotificationApi {
  const show = (message: string, severity: UiNotificationSeverity = "info") => {
    set({
      notification: {
        key: ++notificationKey,
        message,
        severity,
      },
    });
  };

  return {
    show,
    success: (message) => show(message, "success"),
    info: (message) => show(message, "info"),
    warning: (message) => show(message, "warning"),
    error: (message) => show(message, "error"),
  };
}

export const useUiStore = create<UiStore>((set) => ({
  lastActiveTabId: null,
  setLastActiveTabId: (id) => set({ lastActiveTabId: id }),
  checkoutComment: "",
  setCheckoutComment: (comment) => set({ checkoutComment: comment }),
  productsReloadKey: 0,
  bumpProductsReloadKey: () => set((s) => ({ productsReloadKey: s.productsReloadKey + 1 })),
  notification: null,
  showNotification: createNotificationApi(set).show,
  notify: createNotificationApi(set),
  hideNotification: () => set({ notification: null }),
}));
