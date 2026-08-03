import * as React from "react";

export interface BadgeProps {
  children?: React.ReactNode;
  tone?: "brand" | "neutral" | "gold" | "danger" | "warning" | "success";
  appearance?: "solid" | "soft" | "outline";
  size?: "sm" | "md" | "lg";
  /** Uppercase mono eyebrow style (e.g. GRATIS • RINGAN). */
  caps?: boolean;
  /** Leading status dot. */
  dot?: boolean;
  style?: React.CSSProperties;
}

/** Small status / eyebrow label. */
export function Badge(props: BadgeProps): JSX.Element;
