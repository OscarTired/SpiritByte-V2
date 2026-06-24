import { forwardRef, type ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

type Variant = "default" | "primary" | "danger" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "btn",
          variant === "primary" && "btn-primary glow-text",
          variant === "danger" && "btn-danger",
          variant === "ghost" && "border-transparent shadow-none bg-transparent",
          className,
        )}
        {...props}
      />
    );
  },
);
Button.displayName = "Button";
