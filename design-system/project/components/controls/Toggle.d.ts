import * as React from "react";

export interface ToggleProps {
  checked?: boolean;
  onChange?: (next: boolean) => void;
  disabled?: boolean;
  /** Render the Windows-11 Fluent switch (for WinTick app surfaces). */
  native?: boolean;
  /** Optional inline label; when present, renders a full settings row. */
  label?: React.ReactNode;
  description?: React.ReactNode;
  id?: string;
  style?: React.CSSProperties;
}

/** On/off switch — brand and Windows-11 native variants. */
export function Toggle(props: ToggleProps): JSX.Element;
