import type { ReactNode } from 'react'
import { Icon, type IconName } from '@/components/ui/Icon.tsx'

export function Banner({ icon, label, children }: { icon: IconName; label: string; children: ReactNode }) {
  return (
    <section className="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-divider bg-surface px-6 py-2 text-ink-strong max-md:px-4" aria-label={label} role="status">
      <Icon name={icon} className="size-5 text-ink-muted" />
      {children}
    </section>
  )
}
