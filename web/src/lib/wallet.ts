export interface WalletSession {
  publicKey: string;
  network: "TESTNET" | "PUBLIC";
  connected: boolean;
}

export const SOROBAN_RPC_URLS = {
  TESTNET: "https://soroban-testnet.stellar.org",
  PUBLIC: "https://mainnet.sorobanrpc.com",
} as const;

export async function connectFreighterWallet(
  network: "TESTNET" | "PUBLIC" = "TESTNET"
): Promise<WalletSession> {
  if (typeof window === "undefined") {
    throw new Error("Wallet connection requires a browser context");
  }
  const freighter = (window as unknown as { freighterApi?: { getPublicKey: () => Promise<string> } })
    .freighterApi;
  if (!freighter) {
    throw new Error("Freighter wallet extension not found");
  }
  const publicKey = await freighter.getPublicKey();
  return { publicKey, network, connected: true };
}
