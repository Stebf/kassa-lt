import { createContext, useContext, useEffect, useState } from 'react';
import { LazyStore } from '@tauri-apps/plugin-store';

interface AdminContextType {
  adminModeEnabled: boolean;
  setAdminMode: (enabled: boolean) => Promise<void>;
}

const AdminContext = createContext<AdminContextType | undefined>(undefined);

export function AdminProvider({ children }: { children: React.ReactNode }) {
  const [adminModeEnabled, setAdminModeEnabled] = useState(false);

  useEffect(() => {
    const fetchAdminMode = async () => {
      const store = new LazyStore('settings.json');
      const adminMode = await store.get<{ adminMode: boolean }>('adminMode');
      setAdminModeEnabled(adminMode?.adminMode || false);
    };
    fetchAdminMode();
  }, []);

  const setAdminMode = async (enabled: boolean) => {
    setAdminModeEnabled(enabled);
    const store = new LazyStore('settings.json');
    await store.set('adminMode', { adminMode: enabled });
  };

  return (
    <AdminContext.Provider value={{ adminModeEnabled, setAdminMode }}>
      {children}
    </AdminContext.Provider>
  );
}

export function useAdmin() {
  const context = useContext(AdminContext);
  if (context === undefined) {
    throw new Error('useAdmin must be used within an AdminProvider');
  }
  return context;
}
