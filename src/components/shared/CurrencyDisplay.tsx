interface CurrencyDisplayProps {
  amount: string;
  asset?: string;
}

const assetDecimals: Record<string, number> = {
  USDC: 2,
  ETH: 6,
  WETH: 6,
};

export function CurrencyDisplay({ amount, asset }: CurrencyDisplayProps) {
  const num = parseFloat(amount);
  if (isNaN(num)) {
    return <span className="font-mono">{amount}</span>;
  }
  const decimals = asset ? (assetDecimals[asset] ?? 6) : 2;
  const prefix = asset && asset !== "USDC" ? "" : "$";
  const suffix = asset && asset !== "USDC" ? ` ${asset}` : "";
  const formatted = num.toLocaleString("en-US", {
    minimumFractionDigits: Math.min(decimals, 2),
    maximumFractionDigits: decimals,
  });

  return <span className="font-mono">{prefix}{formatted}{suffix}</span>;
}
