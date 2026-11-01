import { useCommentsStore } from '@/stores/commentsStore.ts'
import { useRedactionStore } from '@/stores/redactionStore.ts'
import { useWorkspaceStore, type Tool } from '@/stores/workspaceStore.ts'
import { Button } from '@/components/ui/Button.tsx'
import { usePermission } from '@/hooks/usePermission.ts'

const SHOWN_WITH: readonly Tool[] = ['select', 'redactText', 'redactArea']

export function RedactionControl() {
  const redaction = useRedactionStore()
  const { comments } = useCommentsStore()
  const { tool } = useWorkspaceStore()
  const { confirming, applying, notice } = redaction
  const editing = usePermission('modifyContent')
  const marks = comments.filter((comment) => comment.subtype === 'Redact').length
  if ((marks === 0 && !notice) || !(SHOWN_WITH.includes(tool) || confirming || applying)) return null
  return (
    <div className="flex min-h-10 items-center gap-2" role="group" aria-label="Redaction">
      {confirming ? (
        <>
          <span className="text-ink-strong">
            Remove everything under {marks === 1 ? 'the mark' : `${marks} marks`} for good?
          </span>
          <Button look="danger" onClick={() => void redaction.apply()}>
            Apply
          </Button>
          <Button onClick={() => redaction.cancel()}>
            Cancel
          </Button>
        </>
      ) : (
        marks > 0 && (
          <Button disabled={applying || !editing} onClick={() => redaction.requestApply()}>
            {applying ? 'Applying redactions' : `Apply redactions (${marks})`}
          </Button>
        )
      )}
      {notice && (
        <span className="text-[13px] text-negative" role="status">
          {notice}
        </span>
      )}
    </div>
  )
}
