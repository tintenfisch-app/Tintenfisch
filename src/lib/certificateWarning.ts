import type { CertificateSummary } from '@/types/protocol.ts'

function day(milliseconds: number): string {
  return new Date(milliseconds).toISOString().slice(0, 10)
}

export function certificateWarning(certificate: CertificateSummary): string | null {
  switch (certificate.validity) {
    case 'current':
      return null
    case 'expired':
      return `The certificate of ${certificate.signer} expired on ${day(certificate.validUntil)}. Signatures made with it show as expired.`
    case 'notYetValid':
      return `The certificate of ${certificate.signer} is only valid from ${day(certificate.validFrom)}. Signatures made with it show as not yet valid.`
  }
}
