import { clsx } from "clsx";

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {}

export function Card({ className, children, ...props }: CardProps) {
  return (
    <div
      className={clsx(
        "glass rounded-[16px] shadow-[0_4px_24px_rgba(0,0,0,0.4)]",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}
