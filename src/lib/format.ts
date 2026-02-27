// Formatting utilities for currency, dates, addresses

export function formatCurrency(amount: string, asset = "USDC"): string {
  return `${amount} ${asset}`;
}

export function truncateAddress(address: string, chars = 4): string {
  if (address.length <= chars * 2 + 2) return address;
  return `${address.slice(0, chars + 2)}...${address.slice(-chars)}`;
}
