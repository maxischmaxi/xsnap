import React from "react";

type BadgeVariant = "success" | "warning" | "error" | "info";

interface BadgeProps {
  label: string;
  variant?: BadgeVariant;
}

const baseStyle: React.CSSProperties = {
  fontFamily: "'Liberation Sans', Arial, sans-serif",
  fontSize: "12px",
  fontWeight: 600,
  padding: "4px 12px",
  borderRadius: "9999px",
  display: "inline-flex",
  alignItems: "center",
  lineHeight: "1.4",
};

const variants: Record<BadgeVariant, React.CSSProperties> = {
  success: {
    backgroundColor: "#dcfce7",
    color: "#166534",
  },
  warning: {
    backgroundColor: "#fef9c3",
    color: "#854d0e",
  },
  error: {
    backgroundColor: "#fee2e2",
    color: "#991b1b",
  },
  info: {
    backgroundColor: "#dbeafe",
    color: "#1e40af",
  },
};

export const Badge: React.FC<BadgeProps> = ({
  label,
  variant = "info",
}) => {
  return (
    <span style={{ ...baseStyle, ...variants[variant] }}>
      {label}
    </span>
  );
};
