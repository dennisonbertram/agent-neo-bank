// Formatting utilities for currency, dates, addresses

const assetDecimals: Record<string, number> = {
  USDC: 2,
  ETH: 6,
  WETH: 6,
};

export function formatCurrency(amount: string, asset = "USDC"): string {
  const num = parseFloat(amount);
  if (isNaN(num)) return `-- ${asset}`;
  const decimals = assetDecimals[asset] ?? 6;
  const formatted = num.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: decimals,
  });
  return `${formatted} ${asset}`;
}

export function truncateAddress(address: string, chars = 4): string {
  if (address.length <= chars * 2 + 2) return address;
  return `${address.slice(0, chars + 2)}...${address.slice(-chars)}`;
}
