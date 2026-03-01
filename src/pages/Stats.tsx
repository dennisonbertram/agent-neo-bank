import { ScreenHeader } from '../components/layout/ScreenHeader'

export default function Stats() {
  return (
    <div className="flex flex-col h-full">
      <ScreenHeader title="Stats" />
      <div className="flex-1 overflow-y-auto px-6 pt-4 scrollbar-hide">
        <p className="text-[15px] text-[var(--text-secondary)]">
          Coming soon
        </p>
      </div>
    </div>
  )
}
