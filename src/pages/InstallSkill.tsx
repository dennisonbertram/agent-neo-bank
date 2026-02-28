import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Package, ChevronDown, FileText } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { SuccessCheck } from '../components/ui/SuccessCheck'
import { cn } from '../lib/cn'

type ScreenState = 'install' | 'success'

export default function InstallSkill() {
  const [state, setState] = useState<ScreenState>('install')
  const [expanded, setExpanded] = useState(false)
  const navigate = useNavigate()

  if (state === 'success') {
    return (
      <div className="flex flex-col h-full relative">
        <div className="flex-1 flex flex-col items-center justify-center px-10 text-center">
          <SuccessCheck className="mb-6" />
          <h1 className="text-title mb-3">Skill installed</h1>
          <p className="text-body max-w-[280px]">
            The Research Skill has been configured and is ready to use. Connect your wallet to get started.
          </p>
        </div>
        <div className="absolute bottom-[50px] left-[40px] right-[40px]">
          <Button variant="primary" onClick={() => navigate('/setup/connect')}>
            Continue
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-y-auto px-6 pt-[72px] pb-[160px]">
        {/* Icon badge */}
        <div className="w-[48px] h-[48px] rounded-[16px] bg-[var(--accent-green-dim)] flex items-center justify-center mb-5">
          <Package size={24} color="var(--status-active-text)" />
        </div>

        {/* Title + description */}
        <h1 className="text-title mb-3">Install Research Skill</h1>
        <p className="text-body mb-8">
          This skill allows AI agents to interact with your wallet through controlled spending policies and approval workflows.
        </p>

        {/* Skill card */}
        <div className="bg-[var(--bg-secondary)] rounded-[20px] p-5">
          {/* Expand header */}
          <button
            type="button"
            data-testid="expand-changes"
            onClick={() => setExpanded(!expanded)}
            className="flex items-center justify-between w-full bg-transparent border-none cursor-pointer p-0 appearance-none outline-none text-left"
          >
            <span className="text-[15px] font-medium text-[var(--text-primary)]">
              What changes?
            </span>
            <ChevronDown
              size={18}
              className={cn(
                'text-[var(--text-secondary)] transition-transform duration-200',
                expanded ? 'rotate-0' : '-rotate-90'
              )}
            />
          </button>

          {/* Expandable panel */}
          {expanded && (
            <div className="border-t border-black/5 mt-4 pt-4 flex flex-col gap-3">
              {/* File item: claude.md */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <FileText size={16} className="text-[var(--text-secondary)]" />
                  <div>
                    <span className="text-mono text-[13px] text-[var(--text-primary)]">claude.md</span>
                    <span className="text-[12px] text-[var(--text-secondary)] ml-2">Config update</span>
                  </div>
                </div>
                <span className="text-[10px] font-bold uppercase px-2 py-1 rounded-[4px] bg-[var(--accent-green-dim)] text-[var(--status-active-text)]">
                  UPDATED
                </span>
              </div>

              {/* File item: agents.md */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <FileText size={16} className="text-[var(--text-secondary)]" />
                  <div>
                    <span className="text-mono text-[13px] text-[var(--text-primary)]">agents.md</span>
                    <span className="text-[12px] text-[var(--text-secondary)] ml-2">Permissions</span>
                  </div>
                </div>
                <span className="text-[10px] font-bold uppercase px-2 py-1 rounded-[4px] bg-[var(--accent-green-dim)] text-[var(--status-active-text)]">
                  UPDATED
                </span>
              </div>
            </div>
          )}
        </div>

        {/* Footer note */}
        <p className="text-[12px] text-[var(--text-secondary)] mt-4">
          All changes are local to your machine. No data is sent externally.
        </p>
      </div>

      {/* Buttons pinned to bottom */}
      <div className="absolute bottom-[50px] left-[40px] right-[40px] flex flex-col gap-3">
        <Button variant="primary" onClick={() => setState('success')}>
          Confirm Installation
        </Button>
        <Button variant="outline" onClick={() => navigate(-1)}>
          Cancel
        </Button>
      </div>
    </div>
  )
}
