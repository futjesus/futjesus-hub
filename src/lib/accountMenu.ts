import { Menu } from "@tauri-apps/api/menu";

type Actions = {
  canHibernate: boolean;
  onHibernate: () => void;
  onReload: () => void;
  onEdit: () => void;
  onRemove: () => void;
};

/** Native context menu, so it draws above the Gmail webviews. */
export const showAccountMenu = async ({
  canHibernate,
  onHibernate,
  onReload,
  onEdit,
  onRemove,
}: Actions) => {
  // No ids: the app menu already uses ids like "reload" and they are global.
  const menu = await Menu.new({
    items: [
      { text: "Recargar", action: onReload },
      { text: "Hibernar ahora", enabled: canHibernate, action: onHibernate },
      { text: "Editar…", action: onEdit },
      { item: "Separator" },
      { text: "Eliminar…", action: onRemove },
    ],
  });
  await menu.popup();
};
