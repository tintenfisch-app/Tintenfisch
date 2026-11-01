import { create } from 'zustand'
import { pdfDate } from '@/lib/pdfDate.ts'
import { CancelledError, engine } from '@/services/EngineClient.ts'
import { onDocumentChanged, onDocumentReplaced, useDocumentStore } from '@/stores/documentStore.ts'
import { useProtectionStore } from '@/stores/protectionStore.ts'
import { useWorkspaceStore } from '@/stores/workspaceStore.ts'
import type { Comment, MarkupRequest } from '@/types/protocol.ts'

const STROKE_WIDTH = 2

interface CommentsStore {
  comments: Comment[]
  selected: string | null
  typing: string | null
  placing: boolean
  notice: string | null
  reload(): Promise<void>
  add(pageId: string, markup: MarkupRequest): Promise<void>
  remove(id: string): Promise<void>
  removeSelected(): Promise<void>
  setContents(id: string, contents: string): Promise<void>
  select(id: string | null): void
  keep(key: string): void
  typed(): string
  reset(): void
}

export const useCommentsStore = create<CommentsStore>()((set, get) => {
  let sequence = 0
  let early = ''

  const attempt = async (work: () => Promise<void>) => {
    try {
      await work()
    } catch (error) {
      if (!(error instanceof CancelledError)) set({ notice: (error as Error).message })
    }
  }

  const commentOf = (id: string) => get().comments.find((comment) => comment.id === id)

  const annotating = (markup?: MarkupRequest) =>
    useProtectionStore.getState().permits(markup?.kind === 'redactArea' || markup?.kind === 'redactText' ? 'modifyContent' : 'annotate')

  return {
    comments: [],
    selected: null,
    typing: null,
    placing: false,
    notice: null,

    async reload() {
      const loading = ++sequence
      await attempt(async () => {
        const comments = await engine.request({ kind: 'comments' }, { group: 'comments' })
        if (loading !== sequence) return
        const { selected } = get()
        set({ comments, selected: comments.some((comment) => comment.id === selected) ? selected : null })
      })
    },

    async add(pageId, markup) {
      if (!annotating(markup)) return
      const style = { color: useWorkspaceStore.getState().activeColor(), width: STROKE_WIDTH, author: '', date: pdfDate(new Date()) }
      const takesText = markup.kind === 'note' || markup.kind === 'textBox'
      if (takesText) {
        early = ''
        set({ placing: true })
      }
      await attempt(async () => {
        const added = await engine.request({ kind: 'addMarkup', pageId, markup, style })
        useDocumentStore.getState().touchPages(added.dirtyPages)
        await get().reload()
        set({ selected: added.id, typing: takesText ? added.id : null })
        if (takesText) useWorkspaceStore.getState().showRight('comments')
      })
      if (get().placing) set({ placing: false })
    },

    async remove(id) {
      const comment = commentOf(id)
      if (!comment || !annotating()) return
      await attempt(async () => {
        const change = await engine.request({ kind: 'deleteComment', pageId: comment.pageId, commentId: id })
        useDocumentStore.getState().touchPages(change.dirtyPages)
        await get().reload()
      })
    },

    async removeSelected() {
      const { selected } = get()
      if (selected) await get().remove(selected)
    },

    async setContents(id, contents) {
      const comment = commentOf(id)
      if (!comment || comment.contents === contents || !annotating()) return
      await attempt(async () => {
        const change = await engine.request({ kind: 'setCommentContents', commentId: id, contents, date: pdfDate(new Date()) })
        useDocumentStore.getState().touchPages(change.dirtyPages)
        await get().reload()
      })
    },

    select(id) {
      if (id !== get().selected) set({ selected: id })
    },

    keep(key) {
      early = key === 'Backspace' ? early.slice(0, -1) : early + key
    },

    typed() {
      const kept = early
      early = ''
      if (get().typing) set({ typing: null })
      return kept
    },

    reset() {
      sequence++
      early = ''
      set({ comments: [], selected: null, typing: null, placing: false, notice: null })
    },
  }
})

onDocumentReplaced((ready) => {
  useCommentsStore.getState().reset()
  if (ready) void useCommentsStore.getState().reload()
})

onDocumentChanged(() => void useCommentsStore.getState().reload())
