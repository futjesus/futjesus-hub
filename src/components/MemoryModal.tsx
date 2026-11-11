import { useEffect, useState } from "react";
import { Loading, Modal, ProgressBar, Typography } from "@konstructio/ui";
import { RefreshIcon } from "@konstructio/ui/icons";

import { MemoryReport, api, formatBytes } from "../lib/api";
import { ICON_BUTTON_CLASSES, IconButton } from "./IconButton";

const REFRESH_MS = 2000;

type Props = {
  onClose: () => void;
  onReload: (id: string) => void;
};

export const MemoryModal = ({ onClose, onReload }: Props) => {
  const [report, setReport] = useState<MemoryReport>();

  useEffect(() => {
    let cancelled = false;
    const refresh = () =>
      api.getMemoryUsage().then((next) => {
        if (!cancelled) setReport(next);
      });
    refresh();
    const timer = setInterval(refresh, REFRESH_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  const largest = Math.max(1, ...(report?.entries.map((entry) => entry.bytes) ?? []));

  return (
    <Modal
      isOpen
      showCloseButton
      autoFocus={false}
      onClose={onClose}
      buttonCloseClassName={ICON_BUTTON_CLASSES}
      className="flex w-[520px] flex-col gap-5 p-6"
    >
      <Modal.Header>
        <Typography variant="h5">Consumo de memoria</Typography>
      </Modal.Header>
      <Modal.Body>
        {report ? (
          <div className="flex flex-col gap-4">
            <ul className="flex flex-col gap-3">
              {report.entries.map((entry) => (
                <li key={entry.id ?? entry.label} className="flex items-center gap-3">
                  <ProgressBar
                    className="flex-1"
                    percent={(entry.bytes / largest) * 100}
                    ariaLabel={entry.label}
                    leftContent={
                      <Typography variant="body2" className="truncate font-semibold">
                        {entry.label}
                      </Typography>
                    }
                    rightContent={
                      <Typography variant="body2" className="tabular-nums">
                        {formatBytes(entry.bytes)}
                      </Typography>
                    }
                  />
                  {entry.id ? (
                    <IconButton
                      label="Recargar para liberar memoria"
                      onClick={() => entry.id && onReload(entry.id)}
                    >
                      <RefreshIcon className="size-4" />
                    </IconButton>
                  ) : (
                    <span className="w-8 shrink-0" />
                  )}
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-between border-t border-metal-200 pt-3 dark:border-metal-700">
              <Typography variant="subtitle2">Total</Typography>
              <Typography variant="subtitle2" className="tabular-nums">
                {formatBytes(report.total)}
              </Typography>
            </div>
            <Typography variant="body3" className="text-metal-500">
              Se actualiza cada 2 segundos. Igual que en el Monitor de Actividad, no incluye
              los procesos de red y GPU que WebKit comparte entre cuentas.
            </Typography>
          </div>
        ) : (
          <div className="flex justify-center py-8">
            <Loading />
          </div>
        )}
      </Modal.Body>
    </Modal>
  );
};
