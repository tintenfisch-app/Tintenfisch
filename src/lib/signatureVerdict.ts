import type { SignatureInfo, SignerIdentity } from '@/types/protocol.ts'

export type SignatureTone = 'caution' | 'negative' | 'muted'

export interface SignatureVerdict {
  label: string
  tone: SignatureTone
  detail: string | null
}

const IDENTITY_LABELS: Record<SignerIdentity, string> = {
  expired: 'Signature intact, certificate expired',
  notYetValid: 'Signature intact, certificate not yet valid',
  unverified: 'Signature intact, signer identity not verified',
}

export function signatureVerdict(report: SignatureInfo): SignatureVerdict {
  switch (report.status) {
    case 'intact': {
      const label = IDENTITY_LABELS[report.identity ?? 'unverified']
      return {
        label: report.changedAfter ? `${label}, document changed since signing` : label,
        tone: 'caution',
        detail: report.identityProblem,
      }
    }
    case 'broken':
      return { label: `Signature broken: ${report.problem ?? 'unknown reason'}`, tone: 'negative', detail: null }
    case 'unsupported':
      return { label: `Signature not checked: ${report.problem ?? 'unknown reason'}`, tone: 'muted', detail: null }
  }
}
