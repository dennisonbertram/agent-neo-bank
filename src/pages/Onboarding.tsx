import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { AppLogo } from '../components/icons/AppLogo'
import { Button } from '../components/ui/Button'

const slides = [
  {
    title: "Your Agents,\nYour Rules",
    description: "Set spending limits, approve transactions, and keep your AI agents accountable — all from one dashboard.",
  },
  {
    title: "Smart Spending\nPolicies",
    description: "Define daily, weekly, and per-transaction limits. Agents operate within your boundaries automatically.",
  },
  {
    title: "Real-Time\nTransparency",
    description: "Every transaction includes metadata — what was purchased, why, and for which service. Full audit trail.",
  },
  {
    title: "Get Started\nin Minutes",
    description: "Connect your Coinbase wallet, set up your first agent, and start managing AI spending today.",
  },
]

export default function Onboarding() {
  const [currentSlide, setCurrentSlide] = useState(0)
  const navigate = useNavigate()
  const isLast = currentSlide === slides.length - 1

  const handleNext = () => {
    if (isLast) {
      navigate('/setup/install')
    } else {
      setCurrentSlide((prev) => prev + 1)
    }
  }

  const slide = slides[currentSlide]!

  return (
    <div className="flex flex-col h-full relative">
      {/* Content area — centered vertically */}
      <div className="flex-1 flex flex-col items-center justify-center px-10">
        <AppLogo size={60} className="mb-8" />

        {/* Slide content with animation key to trigger re-render */}
        <div key={currentSlide} className="animate-slide-up text-center">
          <h1
            className="text-[28px] font-semibold leading-tight tracking-[-0.5px] text-[var(--text-primary)] whitespace-pre-line"
          >
            {slide.title}
          </h1>
          <p className="text-body mt-4 max-w-[280px] mx-auto">
            {slide.description}
          </p>
        </div>

        {/* Indicator dots */}
        <div className="flex items-center gap-[6px] mt-10">
          {slides.map((_, i) => (
            <div
              key={i}
              className={
                i === currentSlide
                  ? 'w-[24px] h-[6px] rounded-[4px] bg-[var(--text-primary)] transition-all duration-300'
                  : 'w-[6px] h-[6px] rounded-full bg-[var(--text-tertiary)] transition-all duration-300'
              }
            />
          ))}
        </div>
      </div>

      {/* CTA button pinned to bottom */}
      <div className="absolute bottom-[50px] left-[40px] right-[40px]">
        <Button variant="primary" onClick={handleNext}>
          {isLast ? 'Get set up' : 'Next'}
        </Button>
      </div>
    </div>
  )
}
