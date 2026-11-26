import React from "react";

export const metadata = {
  title: "HeirSwitch — TTL-Aware Soroban Recovery Vault",
  description: "Self-custody dead-man's switch and multi-heir recovery vault on Stellar Soroban",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body style={{ margin: 0, background: "#f8fafc", color: "#0f172a" }}>{children}</body>
    </html>
  );
}
