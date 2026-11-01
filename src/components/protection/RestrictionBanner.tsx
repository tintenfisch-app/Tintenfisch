import { useState, type FormEvent } from 'react'
import { Banner } from '@/components/ui/Banner.tsx'
import { Button } from '@/components/ui/Button.tsx'
import { Input } from '@/components/ui/Field.tsx'
import { Notice } from '@/components/ui/Text.tsx'
import { useProtectionStore } from '@/stores/protectionStore.ts'
import type { EditRight } from '@/types/protocol.ts'

const LIMITS: Record<EditRight, string> = {
  fillForms: 'fill in forms',
  annotate: 'add or change comments and links',
  designForms: 'prepare form fields or place signatures',
  modifyContent: 'edit text and images or apply redactions',
  assemble: 'organize pages or bookmarks',
}

export function RestrictionBanner() {
  const { current, notice, dialogOpen, grantOwnerRights } = useProtectionStore()
  const [password, setPassword] = useState('')
  if (!current || current.withheld.length === 0) return null

  function unlock(event: FormEvent) {
    event.preventDefault()
    void grantOwnerRights(password)
    setPassword('')
  }

  return (
    <Banner icon="lock" label="Restricted document">
      <p className="min-w-0 flex-1 basis-64">
        This document is restricted. Without its permissions password you cannot {current.withheld.map((right) => LIMITS[right]).join(', ')}.
      </p>
      <form className="flex items-center gap-2 max-md:w-full" onSubmit={unlock}>
        <Input compact className="max-md:min-w-0 max-md:flex-1" type="password" aria-label="Permissions password" placeholder="Permissions password" autoComplete="off" value={password} onChange={(event) => setPassword(event.target.value)} />
        <Button type="submit" look="primary">
          Unlock
        </Button>
      </form>
      {notice && !dialogOpen && (
        <Notice role="alert" className="w-full">
          {notice}
        </Notice>
      )}
    </Banner>
  )
}
