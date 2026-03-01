import { clsx } from "clsx";

interface BadgeProps {
  variant?: "default" | "success" | "warning" | "error";
  children: React.ReactNode;
  className?: string;
}

export function Badge({ variant = "default", children, className }: BadgeProps) {
  return (
    <span
      className={clsx(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
        {
          "bg-bg-tertiary text-text-secondary": variant === "default",
          "bg-accent-success/20 text-accent-success": variant === "success",
          "bg-accent-warning/20 text-accent-warning": variant === "warning",
          "bg-accent-error/20 text-accent-error": variant === "error",
        },
        className,
      )}
    >
      {children}
    </span>
  );
}
