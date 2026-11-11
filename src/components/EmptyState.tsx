import { Button, Typography } from "@konstructio/ui";
import { PlusIcon } from "@konstructio/ui/icons";

export const EmptyState = ({ onAdd }: { onAdd: () => void }) => (
  <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
    <img src="/logo.svg" alt="" className="size-24 drop-shadow-lg" />
    <Typography variant="h3">Bienvenido a Futjesus Hub</Typography>
    <Typography variant="body1" className="max-w-md text-metal-500">
      Todas tus cuentas de Gmail en una sola ventana, cada una con su propia sesión.
    </Typography>
    <Button variant="primary" onClick={onAdd}>
      <PlusIcon className="size-4" />
      Agregar cuenta de Gmail
    </Button>
  </div>
);
