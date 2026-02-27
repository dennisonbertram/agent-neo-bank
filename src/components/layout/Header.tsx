import { useBalance } from "@/hooks/useBalance";
import { CurrencyDisplay } from "@/components/shared/CurrencyDisplay";

export function Header() {
  const { balance, isLoading } = useBalance();

  return (
    <header role="banner" className="flex h-14 items-center justify-between border-b border-[#E8E5E0] bg-[#FAFAF9] px-6">
      <span className="text-sm font-semibold text-[#1A1A1A]">Agent Neo Bank</span>
      <div className="text-sm text-[#1A1A1A]">
        {isLoading ? (
          <span>Loading...</span>
        ) : balance ? (
          <CurrencyDisplay amount={balance} />
        ) : (
          <span>--</span>
        )}
      </div>
    </header>
  );
}
