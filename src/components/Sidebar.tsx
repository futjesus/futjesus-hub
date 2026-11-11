import { useRef, useState } from "react";
import { Reorder } from "motion/react";
import { CogOutlineIcon, PlusIcon } from "@konstructio/ui/icons";

import { Account } from "../lib/api";
import { AccountButton } from "./AccountButton";
import { IconButton } from "./IconButton";

type Props = {
  accounts: Account[];
  active: string | null;
  unread: Record<string, number>;
  hibernated: string[];
  onSelect: (id: string) => void;
  onReorder: (ids: string[]) => Promise<void>;
  onContextMenu: (account: Account) => void;
  onAdd: () => void;
  onSettings: () => void;
};

export const Sidebar = ({
  accounts,
  active,
  unread,
  hibernated,
  onSelect,
  onReorder,
  onContextMenu,
  onAdd,
  onSettings,
}: Props) => {
  // Order shown while dragging; otherwise the accounts from props are rendered as-is.
  const [dragOrder, setDragOrder] = useState<Account[] | null>(null);
  const orderRef = useRef(accounts);
  const isDragging = useRef(false);
  const order = dragOrder ?? accounts;

  const handleReorder = (next: Account[]) => {
    orderRef.current = next;
    setDragOrder(next);
  };

  const handleDragEnd = async () => {
    // Releasing the pointer also fires a click; ignore it so a drag never switches accounts.
    setTimeout(() => {
      isDragging.current = false;
    }, 50);
    await onReorder(orderRef.current.map((account) => account.id));
    setDragOrder(null);
  };

  return (
    <aside className="flex h-full w-[72px] shrink-0 flex-col items-center border-r border-metal-200 bg-metal-50 dark:border-metal-800 dark:bg-metal-950">
      {/* Room for the macOS traffic lights; also lets the window be dragged. */}
      <div data-tauri-drag-region className="h-10 w-full shrink-0" />

      <nav className="flex w-full flex-1 flex-col items-center gap-4 overflow-x-hidden overflow-y-auto pt-2 pb-4">
        <Reorder.Group
          as="div"
          axis="y"
          values={order}
          onReorder={handleReorder}
          className="flex w-full flex-col items-center gap-4"
        >
          {order.map((account, index) => (
            <Reorder.Item
              as="div"
              key={account.id}
              value={account}
              whileDrag={{ scale: 1.1, zIndex: 10 }}
              onDragStart={() => {
                isDragging.current = true;
                orderRef.current = order;
              }}
              onDragEnd={handleDragEnd}
              className="relative cursor-grab active:cursor-grabbing"
            >
              <AccountButton
                account={account}
                index={index}
                isActive={account.id === active}
                isHibernated={hibernated.includes(account.id)}
                unread={unread[account.id] ?? 0}
                onSelect={(id) => {
                  if (!isDragging.current) onSelect(id);
                }}
                onContextMenu={onContextMenu}
              />
            </Reorder.Item>
          ))}
        </Reorder.Group>
        <IconButton label="Agregar cuenta de Gmail" size="large" onClick={onAdd}>
          <PlusIcon className="size-5" />
        </IconButton>
      </nav>

      <div className="flex shrink-0 flex-col items-center pb-4">
        <IconButton label="Configuración" size="large" onClick={onSettings}>
          <CogOutlineIcon className="size-5" />
        </IconButton>
      </div>
    </aside>
  );
};
