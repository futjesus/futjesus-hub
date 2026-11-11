import { CSSProperties } from "react";
import { Avatar, Modal, RadioGroup, Switch, Typography } from "@konstructio/ui";
import {
  ArrowDownIcon,
  ArrowUpIcon,
  DeleteIcon,
  EditIcon,
  RefreshIcon,
} from "@konstructio/ui/icons";

import { Account, LinkTarget, Settings, initials } from "../lib/api";
import { ICON_BUTTON_CLASSES, IconButton } from "./IconButton";

type Props = {
  accounts: Account[];
  settings: Settings;
  onClose: () => void;
  onSaveSettings: (settings: Settings) => void;
  onEdit: (account: Account) => void;
  onRemove: (account: Account) => void;
  onReload: (id: string) => void;
  onMove: (id: string, offset: -1 | 1) => void;
};

export const SettingsModal = ({
  accounts,
  settings,
  onClose,
  onSaveSettings,
  onEdit,
  onRemove,
  onReload,
  onMove,
}: Props) => (
  <Modal
    isOpen
    showCloseButton
    autoFocus={false}
    onClose={onClose}
    buttonCloseClassName={ICON_BUTTON_CLASSES}
    className="flex w-[560px] flex-col gap-5 p-6"
  >
    <Modal.Header>
      <Typography variant="h5">Configuración</Typography>
    </Modal.Header>
    <Modal.Body>
      <div className="flex max-h-[65vh] flex-col gap-6 overflow-y-auto pr-1 pb-1">
        <section className="flex flex-col gap-3">
          <Typography variant="subtitle2">Enlaces de los correos</Typography>
          <RadioGroup
            name="linkTarget"
            direction="col"
            value={settings.linkTarget}
            onValueChange={(value) =>
              onSaveSettings({ ...settings, linkTarget: value as LinkTarget })
            }
            options={[
              {
                value: "browser",
                label: "Abrir en el navegador del sistema",
                description: "Adjuntos e inicio de sesión de Google se siguen abriendo en la app.",
              },
              {
                value: "app",
                label: "Abrir dentro de la app",
                description: "En una ventana nueva que comparte la sesión de esa cuenta.",
              },
            ]}
          />
        </section>

        <section className="flex flex-col gap-3">
          <Typography variant="subtitle2">Hibernar cuentas sin usar</Typography>
          <Typography variant="body3" className="text-metal-500">
            Cierra las cuentas que no abres hace un tiempo para liberar memoria. Mientras están
            hibernadas no revisan correos nuevos; despiertan al hacer clic en ellas.
          </Typography>
          <RadioGroup
            name="hibernateAfter"
            direction="row"
            value={settings.hibernateAfter === null ? "off" : String(settings.hibernateAfter)}
            onValueChange={(value) =>
              onSaveSettings({
                ...settings,
                hibernateAfter: value === "off" ? null : Number(value),
              })
            }
            options={[
              { value: "off", label: "Nunca" },
              { value: "15", label: "15 min" },
              { value: "30", label: "30 min" },
              { value: "60", label: "1 hora" },
            ]}
          />
        </section>

        <section className="flex flex-col gap-3">
          <Typography variant="subtitle2">Apariencia</Typography>
          <Switch
            label="Tema oscuro"
            value={settings.theme === "dark"}
            onChange={(isDark) =>
              onSaveSettings({ ...settings, theme: isDark ? "dark" : "light" })
            }
          />
        </section>

        <section className="flex flex-col gap-3">
          <Typography variant="subtitle2">Cuentas</Typography>
          {accounts.length === 0 && (
            <Typography variant="body2">Todavía no agregaste ninguna cuenta.</Typography>
          )}
          <ul className="flex flex-col gap-2">
            {accounts.map((account, index) => (
              <li
                key={account.id}
                className="flex items-center gap-3 rounded-lg border border-metal-200 px-3 py-2 dark:border-metal-700"
              >
                <span
                  className="flex shrink-0"
                  style={{ "--account": account.color } as CSSProperties}
                >
                  <Avatar
                    alt={account.name}
                    src={account.avatar ?? undefined}
                    fallback={initials(account.name)}
                    size="sm"
                    className="bg-(--account) text-white"
                  />
                </span>
                <div className="flex min-w-0 flex-1 flex-col">
                  <Typography variant="body2" className="truncate font-semibold">
                    {account.name}
                    {index < 9 && (
                      <span className="ml-2 font-normal text-metal-400">⌘{index + 1}</span>
                    )}
                  </Typography>
                  <Typography variant="body3" className="truncate text-metal-500">
                    {account.email ?? "Sin iniciar sesión"}
                  </Typography>
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  <IconButton
                    label="Subir"
                    disabled={index === 0}
                    onClick={() => onMove(account.id, -1)}
                  >
                    <ArrowUpIcon className="size-4" />
                  </IconButton>
                  <IconButton
                    label="Bajar"
                    disabled={index === accounts.length - 1}
                    onClick={() => onMove(account.id, 1)}
                  >
                    <ArrowDownIcon className="size-4" />
                  </IconButton>
                  <IconButton label="Recargar" onClick={() => onReload(account.id)}>
                    <RefreshIcon className="size-4" />
                  </IconButton>
                  <IconButton label="Editar" onClick={() => onEdit(account)}>
                    <EditIcon className="size-4" />
                  </IconButton>
                  <IconButton label="Eliminar" onClick={() => onRemove(account)}>
                    <DeleteIcon className="size-4 text-red-600" />
                  </IconButton>
                </div>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </Modal.Body>
  </Modal>
);
