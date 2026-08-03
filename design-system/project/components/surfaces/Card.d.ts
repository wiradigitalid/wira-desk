import * as React from "react";

export interface CardProps {
  variant?: "elevated" | "outline" | "sunken" | "ink";
  padding?: "none" | "sm" | "md" | "lg" | "xl";
  /** Lift + deepen shadow on hover. */
  interactive?: boolean;
  as?: any;
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

/**
 * Warm content surface for brand pages.
 * @startingPoint section="Surfaces" subtitle="Elevated / outline / ink cards" viewport="700x220"
 */
export function Card(props: CardProps): JSX.Element;
