import React from "react";

type ButtonVariant = "primary" | "secondary" | "outline";

interface ButtonProps {
  label: string;
  variant?: ButtonVariant;
  disabled?: boolean;
}

const baseStyle: React.CSSProperties = {
  fontFamily: "'Liberation Sans', Arial, sans-serif",
  fontSize: "14px",
  fontWeight: 600,
  padding: "10px 24px",
  borderRadius: "6px",
  cursor: "pointer",
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  border: "2px solid transparent",
  lineHeight: "1.4",
};

const variants: Record<ButtonVariant, React.CSSProperties> = {
  primary: {
    backgroundColor: "#2563eb",
    color: "#ffffff",
    borderColor: "#2563eb",
  },
  secondary: {
    backgroundColor: "#6b7280",
    color: "#ffffff",
    borderColor: "#6b7280",
  },
  outline: {
    backgroundColor: "transparent",
    color: "#2563eb",
    borderColor: "#2563eb",
  },
};

const disabledStyle: React.CSSProperties = {
  opacity: 0.5,
  cursor: "not-allowed",
};

export const Button: React.FC<ButtonProps> = ({
  label,
  variant = "primary",
  disabled = false,
}) => {
  return (
    <button
      style={{
        ...baseStyle,
        ...variants[variant],
        ...(disabled ? disabledStyle : {}),
      }}
      disabled={disabled}
    >
      {label}
    </button>
  );
};
