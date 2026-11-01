import { useEffect } from 'react'
import { useCommentsStore } from '@/stores/commentsStore.ts'
import { useEditingStore } from '@/stores/editingStore.ts'
import { useFormsStore } from '@/stores/formsStore.ts'
import { useImagesStore } from '@/stores/imagesStore.ts'
import { useLinksStore } from '@/stores/linksStore.ts'
import { usePrintStore } from '@/stores/printStore.ts'
import { useSelectionStore } from '@/stores/selectionStore.ts'
import { useWorkspaceStore } from '@/stores/workspaceStore.ts'

const NOT_TYPED = ['checkbox', 'radio', 'button', 'submit', 'file', 'range', 'color']

function isEditable(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement
}

function typesText(target: EventTarget | null): boolean {
  if (target instanceof HTMLTextAreaElement) return !('editInput' in target.dataset)
  return target instanceof HTMLInputElement && !NOT_TYPED.includes(target.type)
}

function finishesEdit(target: EventTarget | null): boolean {
  const editing = useEditingStore.getState()
  if (!editing.session() || (target instanceof HTMLElement && 'editInput' in target.dataset)) return false
  if (isEditable(target) && !(target instanceof Element && target.closest('[data-keeps-edit]'))) return false
  void editing.commit()
  return true
}

function removeOrDeselect(event: KeyboardEvent, remove: () => Promise<void>, deselect: () => void) {
  if (event.key === 'Delete' || event.key === 'Backspace') {
    event.preventDefault()
    void remove()
  } else if (event.key === 'Escape') {
    deselect()
  }
}

function press(event: KeyboardEvent) {
  const command = event.metaKey || event.ctrlKey
  const key = event.key.toLowerCase()
  const reverting = command && (key === 'z' || (key === 'y' && !event.shiftKey))
  if (reverting && !typesText(event.target)) {
    event.preventDefault()
    const editing = useEditingStore.getState()
    void (key === 'y' || event.shiftKey ? editing.redo() : editing.undo())
  }
  if (command && key === 'p') {
    event.preventDefault()
    usePrintStore.getState().openDialog()
  }
  if (event.key === 'Escape' && finishesEdit(event.target)) return
  if (isEditable(event.target)) return
  const workspace = useWorkspaceStore.getState()
  const { tool } = workspace
  const selection = useSelectionStore.getState()
  const comments = useCommentsStore.getState()
  const forms = useFormsStore.getState()
  const images = useImagesStore.getState()
  const links = useLinksStore.getState()
  const holding = selection.selection || comments.selected || (tool === 'form' && forms.selected) || images.selected || (tool === 'link' && (links.selected || links.draft))
  if (event.key === 'Escape') {
    selection.clear()
    if (!holding) workspace.leaveTool()
  }
  if (comments.selected) removeOrDeselect(event, comments.removeSelected, () => comments.select(null))
  if (tool === 'form' && forms.selected) removeOrDeselect(event, forms.deleteSelected, () => forms.select(null))
  if (images.selected) removeOrDeselect(event, images.removeSelected, () => images.select(null))
  if (tool === 'link' && links.selected && !links.draft) removeOrDeselect(event, links.removeSelected, () => links.select(null))
}

function copy(event: ClipboardEvent) {
  const text = useSelectionStore.getState().selection?.text ?? ''
  if (!text || isEditable(event.target)) return
  event.clipboardData?.setData('text/plain', text)
  event.preventDefault()
}

export function useKeyboardShortcuts() {
  useEffect(() => {
    document.addEventListener('keydown', press)
    document.addEventListener('copy', copy)
    return () => {
      document.removeEventListener('keydown', press)
      document.removeEventListener('copy', copy)
    }
  }, [])
}
