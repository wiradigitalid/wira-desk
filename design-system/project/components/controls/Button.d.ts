import * as React from "react";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual style. `native` = Windows-11 accent button for WinTick app surfaces. */
  variant?: "primary" | "secondary" | "ghost" | "danger" | "native";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  fullWidth?: boolean;
  iconLeft?: React.ReactNode;
  iconRight?: React.ReactNode;
  /** Render as another element/tag, e.g. "a". */
  as?: any;
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

/**
 * Primary call-to-action for Wira brand surfaces.
 * @startingPoint section="Controls" subtitle="Brand + native button variants" viewport="700x160"
 */
export function Button(props: ButtonProps): JSX.Element;
