interface CurrencyDisplayProps {
  amount: string;
  asset?: string;
}

export function CurrencyDisplay({ amount, asset = "USDC" }: CurrencyDisplayProps) {
  return (
    <span className="font-mono">
      {amount} {asset}
    </span>
  );
}
