// Adapted from Untitled UI React (MIT), revision 1d410cb8facde8d50869e7bbb5f3d048e2267466.
// Button-only API and semantic CSS; see ../../public/licenses/untitled-ui.txt.
import type { ReactNode } from "react";
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from "react-aria-components";
export interface ButtonProps extends Omit<AriaButtonProps, "children" | "className" | "isPending"> {
  children: ReactNode;
  color?: "primary" | "secondary" | "quiet" | "danger";
  isLoading?: boolean;
  className?: string;
}
export function Button({
  children,
  color = "primary",
  isLoading = false,
  className = "",
  ...props
}: ButtonProps) {
  return (
    <>
      <AriaButton
        {...props}
        type={props.type || "button"}
        isDisabled={props.isDisabled || isLoading}
        data-loading={isLoading || undefined}
        className={`button button-${color} ${className}`}
      >
        {isLoading && (
          <svg aria-hidden="true" viewBox="0 0 20 20" className="spinner" fill="none">
            <circle cx="10" cy="10" r="8" stroke="currentColor" strokeWidth="2" opacity=".3" />
            <circle
              cx="10"
              cy="10"
              r="8"
              stroke="currentColor"
              strokeWidth="2"
              strokeDasharray="12.5 50"
              strokeLinecap="round"
            />
          </svg>
        )}
        <span>{children}</span>
      </AriaButton>
      {isLoading && (
        <span className="visually-hidden" role="status">
          Working…
        </span>
      )}
    </>
  );
}
