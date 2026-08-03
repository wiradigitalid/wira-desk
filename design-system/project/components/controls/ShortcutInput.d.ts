import * as React from "react";

export interface ShortcutInputProps {
  /** Current combo, as key labels, e.g. ["Win", "`"]. */
  value?: string[];
  /** Called with the captured combo when a key press is recorded. */
  onChange?: (combo: string[]) => void;
  /** Windows-11 native styling for WinTick Settings. */
  native?: boolean;
  disabled?: boolean;
  style?: React.CSSProperties;
}

/** "Listening mode" keyboard-shortcut capturer — captures physical keys, never typed text. */
export function ShortcutInput(props: ShortcutInputProps): JSX.Element;
