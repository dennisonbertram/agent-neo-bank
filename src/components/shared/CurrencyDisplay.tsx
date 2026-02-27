interface CurrencyDisplayProps {
  amount: string;
  asset?: string;
}

export function CurrencyDisplay({ amount }: CurrencyDisplayProps) {
  const num = parseFloat(amount);
  const formatted = isNaN(num)
    ? amount
    : `$${num.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;

  return <span className="font-mono">{formatted}</span>;
}
