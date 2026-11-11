import { invoke } from "@tauri-apps/api/core";

export type Account = {
  id: string;
  name: string;
  color: string;
  email?: string | null;
  avatar?: string | null;
  zoom: number;
};

export type LinkTarget = "browser" | "app";
export type ThemeMode = "light" | "dark";

export type Settings = {
  linkTarget: LinkTarget;
  theme: ThemeMode;
  /** Minutes without opening an account before it hibernates; null disables it. */
  hibernateAfter: number | null;
};

export type Snapshot = {
  accounts: Account[];
  settings: Settings;
  active: string | null;
  unread: Record<string, number>;
  hibernated: string[];
};

export type AccountStatus = {
  id: string;
  unread: number | null;
  email: string | null;
  avatar: string | null;
};

export type MemoryEntry = {
  id: string | null;
  label: string;
  bytes: number;
};

export type MemoryReport = {
  entries: MemoryEntry[];
  total: number;
};

export const ACCOUNT_COLORS = [
  "#ea4335",
  "#fbbc04",
  "#34a853",
  "#4285f4",
  "#a142f4",
  "#f06292",
  "#00acc1",
  "#5f6368",
];

export const api = {
  getSnapshot: () => invoke<Snapshot>("get_snapshot"),
  addAccount: (name: string, color: string) =>
    invoke<Account>("add_account", { name, color }),
  updateAccount: (id: string, name: string, color: string) =>
    invoke<Account[]>("update_account", { id, name, color }),
  removeAccount: (id: string) => invoke<Snapshot>("remove_account", { id }),
  reorderAccounts: (ids: string[]) =>
    invoke<Account[]>("reorder_accounts", { ids }),
  selectAccount: (id: string) => invoke<void>("select_account", { id }),
  reloadAccount: (id: string) => invoke<void>("reload_account", { id }),
  hibernateAccount: (id: string) => invoke<void>("hibernate_account", { id }),
  setOverlay: (visible: boolean) => invoke<void>("set_overlay", { visible }),
  updateSettings: (settings: Settings) =>
    invoke<Settings>("update_settings", { settings }),
  getMemoryUsage: () => invoke<MemoryReport>("get_memory_usage"),
};

export const formatBytes = (bytes: number) => {
  const mb = bytes / 1024 / 1024;
  return mb >= 1024 ? `${(mb / 1024).toFixed(2)} GB` : `${mb.toFixed(0)} MB`;
};

export const initials = (name: string) =>
  name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase())
    .join("") || "?";
