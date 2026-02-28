import { BottomNav } from '../components/layout/BottomNav'

export default function Stats() {
  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-y-auto px-6 pt-[60px] pb-[100px] scrollbar-hide">
        <h1 className="text-title mb-2">Stats</h1>
        <p className="text-[15px] text-[var(--text-secondary)]">
          Coming soon
        </p>
      </div>

      <BottomNav activeTab="stats" />
    </div>
  )
}
