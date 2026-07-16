import type { ReactNode } from "react";

interface CardProps {
  title: string;
  children: ReactNode;
}

export default function Card({ title, children }: CardProps) {
  return (
    <div style={{ border: "1px solid #ccc", borderRadius: 8, padding: 16 }}>
      <h3 style={{ margin: "0 0 8px" }}>{title}</h3>
      {children}
    </div>
  );
}
