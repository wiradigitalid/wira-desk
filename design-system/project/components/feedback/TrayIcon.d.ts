import * as React from "react";

export interface TrayIconProps {
  /** Health state: normal · warning (silent log, red dot) · error (hook dead, red cross). */
  state?: "normal" | "warning" | "error";
  size?: number;
  /** True when sitting on a dark taskbar (glyph drawn white). */
  onDark?: boolean;
  style?: React.CSSProperties;
}

/** WinTick system-tray icon with 3-tier health overlays. */
export function TrayIcon(props: TrayIconProps): JSX.Element;
