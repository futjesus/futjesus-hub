import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTheme } from "@konstructio/ui";

import { api, Account, AccountStatus, Settings, Snapshot } from "../lib/api";

const DEFAULT_SETTINGS: Settings = {
  linkTarget: "browser",
  theme: "light",
  hibernateAfter: null,
};

export const useHub = () => {
  const { setTheme } = useTheme();
  const [loaded, setLoaded] = useState(false);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [unread, setUnread] = useState<Record<string, number>>({});
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [hibernated, setHibernated] = useState<string[]>([]);

  const applySnapshot = useCallback(
    (snapshot: Snapshot) => {
      setAccounts(snapshot.accounts);
      setActive(snapshot.active);
      setUnread(snapshot.unread);
      setSettings(snapshot.settings);
      setHibernated(snapshot.hibernated);
      setTheme?.(snapshot.settings.theme);
    },
    [setTheme],
  );

  useEffect(() => {
    api.getSnapshot().then((snapshot) => {
      applySnapshot(snapshot);
      setLoaded(true);
    });

    const unlisteners = [
      listen<AccountStatus>("account-status", ({ payload }) => {
        const { id, unread: count, email, avatar } = payload;
        if (count !== null) {
          setUnread((prev) =>
            prev[id] === count ? prev : { ...prev, [id]: count },
          );
        }
        const changes: Partial<Account> = {};
        if (email) changes.email = email;
        if (avatar) changes.avatar = avatar;
        setAccounts((prev) =>
          prev.some(
            (a) =>
              a.id === id &&
              Object.entries(changes).some(([key, value]) => a[key as keyof Account] !== value),
          )
            ? prev.map((a) => (a.id === id ? { ...a, ...changes } : a))
            : prev,
        );
      }),
      listen<{ id: string; hibernated: boolean }>("account-hibernated", ({ payload }) => {
        setHibernated((prev) =>
          payload.hibernated
            ? [...new Set([...prev, payload.id])]
            : prev.filter((id) => id !== payload.id),
        );
        if (payload.hibernated) {
          setUnread(({ [payload.id]: _, ...rest }) => rest);
        }
      }),
      // Fired when an account is selected from the native menu (⌘1…⌘9).
      listen<string>("active-changed", ({ payload }) => setActive(payload)),
    ];

    return () => {
      unlisteners.forEach((unlisten) => unlisten.then((fn) => fn()));
    };
  }, [applySnapshot]);

  const select = useCallback((id: string) => {
    setActive(id);
    api.selectAccount(id);
  }, []);

  const add = useCallback(async (name: string, color: string) => {
    const account = await api.addAccount(name, color);
    setAccounts((prev) => [...prev, account]);
    setActive(account.id);
  }, []);

  const update = useCallback(async (id: string, name: string, color: string) => {
    setAccounts(await api.updateAccount(id, name, color));
  }, []);

  const remove = useCallback(
    async (id: string) => applySnapshot(await api.removeAccount(id)),
    [applySnapshot],
  );

  const reorder = useCallback(async (ids: string[]) => {
    setAccounts(await api.reorderAccounts(ids));
  }, []);

  const move = useCallback(
    (id: string, offset: -1 | 1) => {
      const ids = accounts.map((a) => a.id);
      const from = ids.indexOf(id);
      const to = from + offset;
      if (from < 0 || to < 0 || to >= ids.length) return;
      [ids[from], ids[to]] = [ids[to], ids[from]];
      return reorder(ids);
    },
    [accounts, reorder],
  );

  const saveSettings = useCallback(
    async (next: Settings) => {
      setSettings(await api.updateSettings(next));
      setTheme?.(next.theme);
    },
    [setTheme],
  );

  return {
    loaded,
    accounts,
    active,
    unread,
    settings,
    hibernated,
    hibernate: api.hibernateAccount,
    select,
    add,
    update,
    remove,
    reorder,
    move,
    reload: api.reloadAccount,
    saveSettings,
  };
};
