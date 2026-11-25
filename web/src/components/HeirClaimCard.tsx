"use client";

import React from "react";
import { computeHeirPayout, HeirShareView } from "../lib/heirswitch";

export function HeirClaimCard({
  heirs,
  principalStroops,
  claimable,
}: {
  heirs: HeirShareView[];
  principalStroops: bigint;
  claimable: boolean;
}) {
  return (
    <div style={{ border: "1px solid #e2e8f0", borderRadius: 10, padding: "1rem", marginTop: "1rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between" }}>
        <strong>Beneficiary Claim Matrix</strong>
        <span>{claimable ? "Unlocked for Claim" : "Locked (Owner Active)"}</span>
      </div>
      <ul style={{ marginTop: "0.75rem" }}>
        {heirs.map((h) => (
          <li key={h.heir}>
            {h.heir} &mdash; {(h.shareBps / 100).toFixed(1)}% (
            {(Number(computeHeirPayout(principalStroops, h.shareBps)) / 1e7).toFixed(2)} USDC)
          </li>
        ))}
      </ul>
    </div>
  );
}
