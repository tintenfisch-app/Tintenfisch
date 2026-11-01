import type { SignatureInfo } from '@/types/protocol.ts'
import { Icon } from '@/components/ui/Icon.tsx'
import { sentence } from '@/lib/sentence.ts'
import { signatureVerdict, type SignatureTone } from '@/lib/signatureVerdict.ts'
import { useDigitalSignatureStore } from '@/stores/digitalSignatureStore.ts'

const TONE_COLORS: Record<SignatureTone, string> = {
  caution: 'text-caution',
  negative: 'text-negative',
  muted: 'text-ink-disabled',
}

function tooltip(report: SignatureInfo, detail: string | null): string | undefined {
  const lines = [detail && sentence(detail), report.reason && `Reason: ${report.reason}`].filter(Boolean)
  return lines.length > 0 ? lines.join('\n') : undefined
}

export function SignatureStatusList() {
  const { reports } = useDigitalSignatureStore()
  if (reports.length === 0) return null
  return (
    <ul className="m-0 flex list-none flex-wrap gap-x-6 gap-y-1 border-b border-divider bg-surface px-6 py-2.5" aria-label="Signatures">
      {reports.map((report) => {
        const verdict = signatureVerdict(report)
        return (
          <li key={report.fieldId} className="flex items-center gap-2 text-ink-strong" title={tooltip(report, verdict.detail)}>
            <Icon name="badge-verified" className={`size-5 ${TONE_COLORS[verdict.tone]}`} />
            Signed by {report.signer ?? report.fieldName}: {verdict.label}
          </li>
        )
      })}
    </ul>
  )
}
