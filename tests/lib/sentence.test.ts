import { describe, expect, it } from 'vitest'
import { sentence } from '@/lib/sentence.ts'

describe('sentence', () => {
  it('starts with a capital and ends with a full stop once', () => {
    expect(sentence('the password does not open the certificate file')).toBe('The password does not open the certificate file.')
    expect(sentence('Already done.')).toBe('Already done.')
    expect(sentence('  ')).toBe('')
  })
})
