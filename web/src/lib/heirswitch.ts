export interface HeirShareView {
  heir: string;
  shareBps: number;
}

export interface HeirVaultView {
  owner: string;
  token: string;
  guardian: string | null;
  inactivitySeconds: number;
  lastHeartbeatUnix: number;
  lockedPrincipalStroops: bigint;
  totalClaimedStroops: bigint;
  heirs: HeirShareView[];
}

export const TOTAL_BPS = 10_000;

export function validateHeirSplits(heirs: HeirShareView[]): boolean {
  if (heirs.length === 0) return false;
  let sum = 0;
  for (const h of heirs) {
    if (h.shareBps <= 0) return false;
    sum += h.shareBps;
  }
  return sum === TOTAL_BPS;
}

export function secondsUntilUnlock(vault: HeirVaultView, nowUnix: number): number {
  const unlockAt = vault.lastHeartbeatUnix + vault.inactivitySeconds;
  return Math.max(0, unlockAt - nowUnix);
}

export function computeHeirPayout(principalStroops: bigint, shareBps: number): bigint {
  return (principalStroops * BigInt(shareBps)) / BigInt(TOTAL_BPS);
}
