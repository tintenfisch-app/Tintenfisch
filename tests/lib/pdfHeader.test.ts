import { describe, expect, it } from 'vitest'
import { hasPdfHeader } from '@/lib/pdfHeader.ts'

const text = (value: string) => new TextEncoder().encode(value)

describe('hasPdfHeader', () => {
  it('finds the signature at the start or behind leading junk within the first kilobyte', () => {
    expect(hasPdfHeader(text('%PDF-1.7\n'))).toBe(true)
    expect(hasPdfHeader(text(`${' '.repeat(1000)}%PDF-1.4`))).toBe(true)
  })

  it('refuses files without the signature or with it too far in', () => {
    expect(hasPdfHeader(new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))).toBe(false)
    expect(hasPdfHeader(text('plain text renamed to pdf'))).toBe(false)
    expect(hasPdfHeader(text(`${' '.repeat(1020)}%PDF-1.4`))).toBe(false)
    expect(hasPdfHeader(new Uint8Array())).toBe(false)
  })
})
