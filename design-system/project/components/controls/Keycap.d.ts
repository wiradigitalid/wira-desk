import * as React from "react";

export interface KeycapProps {
  /** Single key label (ignored when `combo` is set). */
  children?: React.ReactNode;
  /** Render a full shortcut, e.g. ["Ctrl", "Win", "Enter"]. */
  combo?: string[];
  size?: "sm" | "md" | "lg";
  tone?: "default" | "brand" | "ink" | "native";
  style?: React.CSSProperties;
}

/** A keyboard key / shortcut combo set in mono — WinTick's core visual motif. */
export function Keycap(props: KeycapProps): JSX.Element;
