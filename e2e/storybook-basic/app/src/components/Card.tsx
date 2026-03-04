import React from "react";

interface CardProps {
  title: string;
  description: string;
  actionLabel?: string;
}

const cardStyle: React.CSSProperties = {
  fontFamily: "'Liberation Sans', Arial, sans-serif",
  width: "320px",
  padding: "24px",
  borderRadius: "8px",
  border: "1px solid #e5e7eb",
  backgroundColor: "#ffffff",
  boxShadow: "0 1px 3px rgba(0, 0, 0, 0.1)",
};

const titleStyle: React.CSSProperties = {
  fontSize: "18px",
  fontWeight: 600,
  color: "#111827",
  margin: "0 0 8px 0",
  lineHeight: "1.4",
};

const descriptionStyle: React.CSSProperties = {
  fontSize: "14px",
  color: "#6b7280",
  margin: "0 0 16px 0",
  lineHeight: "1.5",
};

const buttonStyle: React.CSSProperties = {
  fontFamily: "'Liberation Sans', Arial, sans-serif",
  fontSize: "14px",
  fontWeight: 600,
  padding: "8px 16px",
  borderRadius: "6px",
  border: "none",
  backgroundColor: "#2563eb",
  color: "#ffffff",
  cursor: "pointer",
};

export const Card: React.FC<CardProps> = ({
  title,
  description,
  actionLabel,
}) => {
  return (
    <div style={cardStyle}>
      <h3 style={titleStyle}>{title}</h3>
      <p style={descriptionStyle}>{description}</p>
      {actionLabel && <button style={buttonStyle}>{actionLabel}</button>}
    </div>
  );
};
