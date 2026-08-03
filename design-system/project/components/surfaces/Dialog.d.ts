import * as React from "react";

export interface DialogProps {
  title?: React.ReactNode;
  icon?: React.ReactNode;
  /** Windows-11 window chrome (default). Set false for a warm brand modal. */
  native?: boolean;
  width?: number;
  onClose?: () => void;
  onMin?: (() => void) | null;
  onMax?: (() => void) | null;
  /** Footer node, usually Buttons. */
  footer?: React.ReactNode;
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

/** Window / dialog chrome — native Windows-11 caption bar or brand modal. */
export function Dialog(props: DialogProps): JSX.Element;
