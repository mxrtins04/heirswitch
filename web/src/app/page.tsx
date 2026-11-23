"use client";

import React from "react";
import { computeHeirPayout, HeirVaultView, secondsUntilUnlock } from "../lib/heirswitch";

const SAMPLE_VAULT: HeirVaultView = {
  owner: "GA9X...OWNER",
  token: "USDC",
  guardian: "GB4Q...GUARDIAN",
  inactivitySeconds: 90 * 86400,
  lastHeartbeatUnix: Math.floor(Date.now() / 1000) - 12 * 86400,
  lockedPrincipalStroops: 2_500_0000000n,
  totalClaimedStroops: 0n,
  heirs: [
    { heir: "GC1A...SPOUSE", shareBps: 6_000 },
    { heir: "GD8B...SIBLING", shareBps: 4_000 },
  ],
};

export default function HeirSwitchDashboard() {
  const remainingDays = Math.ceil(
    secondsUntilUnlock(SAMPLE_VAULT, Math.floor(Date.now() / 1000)) / 86400
  );

  return (
    <main style={{ maxWidth: 840, margin: "2rem auto", fontFamily: "system-ui, sans-serif", padding: "0 1rem" }}>
      <h1 style={{ margin: 0 }}>HeirSwitch</h1>
      <p style={{ color: "#555" }}>
        TTL-Aware Dead-Man&apos;s Switch &amp; Multi-Heir Self-Custody Vault on Soroban
      </p>

      <section style={{ border: "1px solid #e2e8f0", borderRadius: 12, padding: "1rem", marginTop: "1.5rem" }}>
        <strong>Vault Active &middot; {remainingDays} days until unlock</strong>
        <ul style={{ marginTop: "0.75rem" }}>
          {SAMPLE_VAULT.heirs.map((h) => (
            <li key={h.heir}>
              {h.heir}: {(h.shareBps / 100).toFixed(1)}% ({Number(computeHeirPayout(SAMPLE_VAULT.lockedPrincipalStroops, h.shareBps)) / 1e7} USDC)
            </li>
          ))}
        </ul>
      </section>
    </main>
  );
}
