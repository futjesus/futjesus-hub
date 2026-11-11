import { FormEvent, useState } from "react";
import { Button, Input, Modal, Typography, cn } from "@konstructio/ui";

import { ACCOUNT_COLORS, Account } from "../lib/api";
import { ICON_BUTTON_CLASSES } from "./IconButton";

type Props = {
  /** The account being edited; omitted when adding a new one. */
  account?: Account;
  defaultColor: string;
  onClose: () => void;
  onSubmit: (name: string, color: string) => Promise<void>;
};

export const AccountModal = ({ account, defaultColor, onClose, onSubmit }: Props) => {
  const [name, setName] = useState(account?.name ?? "");
  const [color, setColor] = useState(account?.color ?? defaultColor);
  const [error, setError] = useState<string>();
  const [isSaving, setIsSaving] = useState(false);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    if (!name.trim()) {
      setError("Escribe un nombre para la cuenta");
      return;
    }
    setIsSaving(true);
    try {
      await onSubmit(name.trim(), color);
      onClose();
    } catch (e) {
      setError(String(e));
      setIsSaving(false);
    }
  };

  return (
    <Modal
      isOpen
      showCloseButton
      onClose={onClose}
      buttonCloseClassName={ICON_BUTTON_CLASSES}
      className="w-[420px] p-6"
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-5">
        <Modal.Header>
          <Typography variant="h5">
            {account ? "Editar cuenta" : "Agregar cuenta de Gmail"}
          </Typography>
        </Modal.Header>
        <Modal.Body>
          <div className="flex flex-col gap-5">
            <Input
              label="Nombre"
              placeholder="Personal, Trabajo…"
              value={name}
              error={error}
              helperText={
                account ? undefined : "Después inicias sesión con Google en la nueva pestaña."
              }
              autoFocus
              isRequired
              onChange={(e) => {
                setName(e.target.value);
                setError(undefined);
              }}
            />
            <div className="flex flex-col gap-2">
              <Typography variant="labelLarge" component="span">
                Color
              </Typography>
              <div role="radiogroup" aria-label="Color" className="flex flex-wrap gap-2">
                {ACCOUNT_COLORS.map((swatch) => (
                  <button
                    key={swatch}
                    type="button"
                    role="radio"
                    aria-checked={swatch === color}
                    aria-label={swatch}
                    onClick={() => setColor(swatch)}
                    style={{ backgroundColor: swatch }}
                    className={cn(
                      "size-8 cursor-pointer rounded-full ring-offset-2 transition",
                      swatch === color
                        ? "ring-2 ring-metal-700 dark:ring-metal-200 dark:ring-offset-metal-900"
                        : "hover:scale-110",
                    )}
                  />
                ))}
              </div>
            </div>
          </div>
        </Modal.Body>
        <Modal.Footer>
          <div className="flex w-full justify-end gap-2 pt-1">
            <Button type="button" variant="secondary" onClick={onClose}>
              Cancelar
            </Button>
            <Button type="submit" variant="primary" disabled={isSaving}>
              {account ? "Guardar" : "Agregar"}
            </Button>
          </div>
        </Modal.Footer>
      </form>
    </Modal>
  );
};
