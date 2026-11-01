import { useState } from 'react'
import { Banner } from '@/components/ui/Banner.tsx'
import { IconButton } from '@/components/ui/IconButton.tsx'
import { useDocumentStore } from '@/stores/documentStore.ts'

const MESSAGES = {
  hybrid: 'This form contains XFA content, which Tintenfisch does not support. The XFA part may be lost when you save.',
  dynamic: 'This is a dynamic XFA form, which Tintenfisch cannot display. The pages may show only a placeholder, and the XFA part may be lost when you save.',
}

export function XfaNotice() {
  const document = useDocumentStore((store) => store.document)
  const [dismissed, setDismissed] = useState<number | null>(null)
  if (document.phase !== 'ready' || document.xfa === 'absent' || dismissed === document.session) return null
  return (
    <Banner icon="alert-triangle" label="XFA form">
      <p className="min-w-0 flex-1 basis-64">{MESSAGES[document.xfa]}</p>
      <IconButton icon="close" label="Dismiss" tooltip={null} onClick={() => setDismissed(document.session)} />
    </Banner>
  )
}
