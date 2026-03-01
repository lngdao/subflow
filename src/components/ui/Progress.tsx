import { clsx } from "clsx";

interface ProgressProps {
  value: number;
  className?: string;
  showLabel?: boolean;
}

export function Progress({ value, className, showLabel = false }: ProgressProps) {
  const clamped = Math.min(100, Math.max(0, value * 100));

  return (
    <div className={clsx("flex items-center gap-3", className)}>
      <div className="flex-1 h-2 bg-bg-secondary rounded-full overflow-hidden">
        <div
          className="h-full rounded-full progress-shimmer transition-all duration-300 ease-out"
          style={{ width: `${clamped}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-xs text-text-tertiary tabular-nums w-10 text-right">
          {Math.round(clamped)}%
        </span>
      )}
    </div>
  );
}
