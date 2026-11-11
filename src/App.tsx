import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ConfirmDialog } from "@konstructio/ui";

import { AccountModal } from "./components/AccountModal";
import { EmptyState } from "./components/EmptyState";
import { MemoryModal } from "./components/MemoryModal";
import { SettingsModal } from "./components/SettingsModal";
import { Sidebar } from "./components/Sidebar";
import { useHub } from "./hooks/useHub";
import { showAccountMenu } from "./lib/accountMenu";
import { ACCOUNT_COLORS, Account, api } from "./lib/api";

type ModalState =
  | { kind: "add" }
  | { kind: "settings" }
  | { kind: "memory" }
  | { kind: "edit"; account: Account; fromSettings: boolean }
  | { kind: "remove"; account: Account; fromSettings: boolean };

const App = () => {
  const hub = useHub();
  const [modal, setModal] = useState<ModalState | null>(null);
  const isModalOpen = modal !== null;

  // Gmail webviews are native views drawn above this page, so they are
  // hidden while a modal is open or they would cover it.
  useEffect(() => {
    api.setOverlay(isModalOpen);
  }, [isModalOpen]);

  // Opened from the native "Cuentas" menu.
  useEffect(() => {
    const unlisten = listen("show-memory", () => setModal({ kind: "memory" }));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const closeModal = () => setModal(null);
  // Edit and remove return to the settings modal when opened from it.
  const closeChild = (fromSettings: boolean) =>
    setModal(fromSettings ? { kind: "settings" } : null);
  const nextColor = ACCOUNT_COLORS[hub.accounts.length % ACCOUNT_COLORS.length];

  return (
    <div className="flex h-full bg-white text-metal-900 dark:bg-metal-900 dark:text-metal-50">
      <Sidebar
        accounts={hub.accounts}
        active={hub.active}
        unread={hub.unread}
        hibernated={hub.hibernated}
        onSelect={hub.select}
        onReorder={hub.reorder}
        onContextMenu={(account) =>
          showAccountMenu({
            // The visible account cannot hibernate; switch away from it first.
            canHibernate: account.id !== hub.active && !hub.hibernated.includes(account.id),
            onHibernate: () => hub.hibernate(account.id),
            onReload: () => hub.reload(account.id),
            onEdit: () => setModal({ kind: "edit", account, fromSettings: false }),
            onRemove: () => setModal({ kind: "remove", account, fromSettings: false }),
          })
        }
        onAdd={() => setModal({ kind: "add" })}
        onSettings={() => setModal({ kind: "settings" })}
      />

      <main className="relative flex-1">
        <div data-tauri-drag-region className="absolute inset-x-0 top-0 h-10" />
        {hub.loaded && hub.accounts.length === 0 && (
          <EmptyState onAdd={() => setModal({ kind: "add" })} />
        )}
      </main>

      {modal?.kind === "add" && (
        <AccountModal defaultColor={nextColor} onClose={closeModal} onSubmit={hub.add} />
      )}

      {modal?.kind === "edit" && (
        <AccountModal
          account={modal.account}
          defaultColor={modal.account.color}
          onClose={() => closeChild(modal.fromSettings)}
          onSubmit={(name, color) => hub.update(modal.account.id, name, color)}
        />
      )}

      {modal?.kind === "settings" && (
        <SettingsModal
          accounts={hub.accounts}
          settings={hub.settings}
          onClose={closeModal}
          onSaveSettings={hub.saveSettings}
          onEdit={(account) => setModal({ kind: "edit", account, fromSettings: true })}
          onRemove={(account) => setModal({ kind: "remove", account, fromSettings: true })}
          onReload={hub.reload}
          onMove={hub.move}
        />
      )}

      {modal?.kind === "memory" && <MemoryModal onClose={closeModal} onReload={hub.reload} />}

      {modal?.kind === "remove" && (
        <ConfirmDialog
          isOpen
          variant="danger"
          title={`¿Eliminar "${modal.account.name}"?`}
          description="Se cerrará la sesión y se borrarán las cookies y datos guardados de esta cuenta en la app. Tu cuenta de Gmail no se modifica."
          confirmLabel="Eliminar"
          cancelLabel="Cancelar"
          onClose={() => closeChild(modal.fromSettings)}
          onConfirm={async () => {
            await hub.remove(modal.account.id);
            closeChild(modal.fromSettings);
          }}
        />
      )}
    </div>
  );
};

export default App;
