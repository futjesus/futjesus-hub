import { CSSProperties } from "react";
import { Avatar, Badge, cn } from "@konstructio/ui";

import { Account, initials } from "../lib/api";

type Props = {
  account: Account;
  index: number;
  isActive: boolean;
  isHibernated: boolean;
  unread: number;
  onSelect: (id: string) => void;
  onContextMenu: (account: Account) => void;
};

export const AccountButton = ({
  account,
  index,
  isActive,
  isHibernated,
  unread,
  onSelect,
  onContextMenu,
}: Props) => {
  const shortcut = index < 9 ? ` (⌘${index + 1})` : "";
  const tooltip = [
    account.name + shortcut,
    account.email,
    isHibernated && "Hibernada: haz clic para despertarla",
    "Clic derecho: más opciones",
  ]
    .filter(Boolean)
    .join("\n");

  return (
    <button
      type="button"
      // The native title tooltip draws above the Gmail webview; an HTML one would be hidden.
      title={tooltip}
      aria-label={account.name}
      aria-current={isActive}
      onClick={() => onSelect(account.id)}
      onContextMenu={(event) => {
        event.preventDefault();
        onContextMenu(account);
      }}
      style={{ "--account": account.color } as CSSProperties}
      className={cn(
        "relative flex cursor-pointer items-center justify-center rounded-full p-0.5 ring-2 transition-opacity",
        isActive ? "ring-(--account)" : "opacity-70 ring-transparent hover:opacity-100",
        isHibernated && "opacity-40 grayscale",
      )}
    >
      {isActive && (
        <span className="absolute top-1/2 -left-3 h-7 w-1 -translate-y-1/2 rounded-r bg-(--account)" />
      )}
      <Avatar
        alt={account.name}
        src={account.avatar ?? undefined}
        fallback={initials(account.name)}
        size="md"
        className="bg-(--account) font-semibold text-white"
      />
      {unread > 0 && (
        <Badge
          label={unread > 99 ? "99+" : String(unread)}
          variant="danger"
          isSelectable={false}
          className="absolute -top-1.5 -right-2.5 min-w-5 justify-center px-1 py-0 text-[10px] leading-4"
        />
      )}
    </button>
  );
};
