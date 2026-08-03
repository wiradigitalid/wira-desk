import * as React from "react";

export interface TrayMenuItem {
  kind: "item" | "toggle" | "separator";
  label?: string;
  checked?: boolean;
}

export interface TrayMenuProps {
  /** Defaults to WinTick's full context menu (Settings, View Logs, Auto-Start, …). */
  items?: TrayMenuItem[];
  width?: number;
  onSelect?: (item: TrayMenuItem, index: number) => void;
  style?: React.CSSProperties;
}

/**
 * Windows-11 tray context-menu flyout.
 * @startingPoint section="Navigation" subtitle="WinTick tray right-click menu" viewport="700x300"
 */
export function TrayMenu(props: TrayMenuProps): JSX.Element;
