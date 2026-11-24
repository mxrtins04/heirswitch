"use client";

import React, { useState } from "react";
import { connectFreighterWallet, WalletSession } from "../lib/wallet";

export function HeartbeatPanel({ remainingDays }: { remainingDays: number }) {
  const [session, setSession] = useState<WalletSession | null>(null);
  const [status, setStatus] = useState<string>("");

  async function handleConnect() {
    try {
      const s = await connectFreighterWallet("TESTNET");
      setSession(s);
      setStatus("Wallet connected — ready to submit Heartbeat + TTL extension");
    } catch (err) {
      setStatus(err instanceof Error ? err.message : "Wallet error");
    }
  }

  return (
    <div style={{ marginTop: "1rem", padding: "0.75rem", background: "#f8fafc", borderRadius: 8 }}>
      <strong>Heartbeat &amp; Soroban TTL Health ({remainingDays}d left)</strong>
      <div style={{ marginTop: "0.5rem", display: "flex", gap: "0.5rem", alignItems: "center" }}>
        <button onClick={handleConnect} style={{ padding: "0.4rem 0.8rem", borderRadius: 6, cursor: "pointer" }}>
          {session ? `Owner: ${session.publicKey.slice(0, 6)}...` : "Connect Owner Wallet"}
        </button>
        {status && <small style={{ color: "#475569" }}>{status}</small>}
      </div>
    </div>
  );
}
