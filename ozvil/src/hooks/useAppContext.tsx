import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppStateInfo,
  Session,
  SystemStatus,
  Settings,
  Profile,
} from "../types";

interface AppContextValue {
  appInfo: AppStateInfo | null;
  activeSession: Session | null;
  systemStatus: SystemStatus | null;
  settings: Settings | null;
  profiles: Profile[] | null;
  refreshStatus: () => void;
  refreshSession: () => void;
  refreshProfiles: () => void;
  refreshSettings: () => void;
  toggleGlobalPause: () => Promise<void>;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [appInfo, setAppInfo] = useState<AppStateInfo | null>(null);
  const [activeSession, setActiveSession] = useState<Session | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [profiles, setProfiles] = useState<Profile[] | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const status = await invoke<SystemStatus>("get_system_status");
      setSystemStatus(status);
    } catch {}
  }, []);

  const refreshSession = useCallback(async () => {
    try {
      const session = await invoke<Session | null>("get_active_session");
      setActiveSession(session);
    } catch {}
  }, []);

  const refreshProfiles = useCallback(async () => {
    try {
      const ps = await invoke<Profile[]>("get_profiles");
      setProfiles(ps);
    } catch {}
  }, []);

  const refreshSettings = useCallback(async () => {
    try {
      const s = await invoke<Settings>("get_settings");
      setSettings(s);
    } catch {}
  }, []);

  const toggleGlobalPause = useCallback(async () => {
    const newVal = await invoke<boolean>("toggle_global_pause");
    setSettings((prev) => prev ? { ...prev, global_pause: newVal } : prev);
    setAppInfo((prev) => prev ? { ...prev, global_pause: newVal } : prev);
  }, []);

  useEffect(() => {
    invoke<AppStateInfo>("get_app_state_info")
      .then(setAppInfo)
      .catch(() => {});

    refreshStatus();
    refreshSession();
    refreshProfiles();
    refreshSettings();

    const statusInterval = setInterval(refreshStatus, 5000);
    const sessionInterval = setInterval(refreshSession, 3000);

    return () => {
      clearInterval(statusInterval);
      clearInterval(sessionInterval);
    };
  }, []);

  return (
    <AppContext.Provider
      value={{
        appInfo,
        activeSession,
        systemStatus,
        settings,
        profiles,
        refreshStatus,
        refreshSession,
        refreshProfiles,
        refreshSettings,
        toggleGlobalPause,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useAppContext(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppContext must be used within AppProvider");
  return ctx;
}
