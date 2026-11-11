import { ReactNode } from "react";
import { Button, cn } from "@konstructio/ui";

/** Borderless circle button; also applied to the modals' close button. */
export const ICON_BUTTON_CLASSES =
  "justify-center border-0 bg-transparent shadow-none hover:bg-metal-200 focus-visible:ring-0 dark:bg-transparent dark:hover:bg-metal-800";

type Props = {
  label: string;
  size?: "small" | "large";
  disabled?: boolean;
  className?: string;
  onClick: () => void;
  children: ReactNode;
};

export const IconButton = ({
  label,
  size = "small",
  disabled,
  className,
  onClick,
  children,
}: Props) => (
  <Button
    variant="tertiary"
    shape="circle"
    title={label}
    aria-label={label}
    disabled={disabled}
    onClick={onClick}
    className={cn(ICON_BUTTON_CLASSES, size === "large" ? "size-10" : "size-8", className)}
  >
    {children}
  </Button>
);
