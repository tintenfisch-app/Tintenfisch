import { create } from 'zustand'
import { useProtectionStore, withholds } from '@/stores/protectionStore.ts'
import type { EditRight, NewFieldKind, ProtectionState, Rgb } from '@/types/protocol.ts'

export type Tool = 'select' | 'edit' | 'addText' | 'addImage' | 'highlight' | 'note' | 'textBox' | 'ink' | 'rectangle' | 'ellipse' | 'arrow' | 'stamp' | 'redactText' | 'redactArea' | 'form' | 'link' | 'sign'

export type LeftPanel = 'tools' | 'edit'
export type RightPanel = 'comments' | 'pages' | 'bookmarks'

export const EDITING_TOOLS: readonly Tool[] = ['edit', 'addText', 'addImage', 'link']

const EDIT_PANEL_TOOLS: readonly Tool[] = [...EDITING_TOOLS, 'redactText', 'redactArea']

export const DRAWING_TOOLS: readonly Tool[] = ['note', 'textBox', 'ink', 'rectangle', 'ellipse', 'arrow', 'stamp', 'redactArea']

const TOOL_RIGHTS: Readonly<Partial<Record<Tool, EditRight>>> = {
  edit: 'modifyContent',
  addText: 'modifyContent',
  addImage: 'modifyContent',
  redactText: 'modifyContent',
  redactArea: 'modifyContent',
  link: 'annotate',
  highlight: 'annotate',
  note: 'annotate',
  textBox: 'annotate',
  ink: 'annotate',
  rectangle: 'annotate',
  ellipse: 'annotate',
  arrow: 'annotate',
  stamp: 'annotate',
  form: 'designForms',
  sign: 'designForms',
}

export function toolWithheld(current: ProtectionState | null, tool: Tool): boolean {
  const right = TOOL_RIGHTS[tool]
  return right !== undefined && withholds(current, right)
}

const usable = (tool: Tool) => !toolWithheld(useProtectionStore.getState().current, tool)

export const MARKUP_COLORS: readonly Rgb[] = [
  [0.86, 0.15, 0.15],
  [0.15, 0.39, 0.92],
  [0.09, 0.64, 0.29],
  [1, 0.83, 0.1],
]

export const STAMP_LABELS: readonly string[] = ['Approved', 'Draft', 'Confidential', 'Rejected']

interface WorkspaceStore {
  tool: Tool
  color: Rgb
  highlightColor: Rgb
  stampLabel: string
  fieldKind: NewFieldKind
  left: LeftPanel
  right: RightPanel | null
  sheet: boolean
  activeColor(): Rgb
  choose(tool: Tool): void
  leaveTool(): void
  setColor(color: Rgb): void
  setStampLabel(stampLabel: string): void
  setFieldKind(fieldKind: NewFieldKind): void
  showLeft(left: LeftPanel): void
  showRight(right: RightPanel | null): void
  toggleRight(right: RightPanel): void
  openSheet(): void
  closeSheet(): void
}

export const useWorkspaceStore = create<WorkspaceStore>()((set, get) => ({
  tool: 'select',
  color: MARKUP_COLORS[0],
  highlightColor: MARKUP_COLORS[3],
  stampLabel: STAMP_LABELS[0],
  fieldKind: 'text',
  left: 'tools',
  right: null,
  sheet: false,

  activeColor() {
    const { tool, color, highlightColor } = get()
    return tool === 'highlight' ? highlightColor : color
  },

  choose(tool) {
    if (!usable(tool)) return
    const { left } = get()
    const leavesEdit = left === 'edit' && !EDIT_PANEL_TOOLS.includes(tool)
    set({ tool, left: EDITING_TOOLS.includes(tool) ? 'edit' : leavesEdit ? 'tools' : left })
  },

  leaveTool() {
    const { tool, left } = get()
    if (tool !== 'select' && tool !== 'edit') get().choose(left === 'edit' && usable('edit') ? 'edit' : 'select')
  },

  setColor(color) {
    set(get().tool === 'highlight' ? { highlightColor: color } : { color })
  },

  setStampLabel(stampLabel) {
    set({ stampLabel })
  },

  setFieldKind(fieldKind) {
    set({ fieldKind })
  },

  showLeft(left) {
    const { tool } = get()
    const editing = EDITING_TOOLS.includes(tool)
    if (left === 'edit' && !EDIT_PANEL_TOOLS.includes(tool)) {
      if (usable('edit')) set({ left, tool: 'edit' })
    } else if (left !== 'edit' && editing) {
      set({ left, tool: 'select' })
    } else if (left !== get().left) {
      set({ left })
    }
  },

  showRight(right) {
    const { sheet } = get()
    if (right !== get().right) set({ right, sheet: right === null && sheet })
  },

  toggleRight(right) {
    get().showRight(get().right === right ? null : right)
  },

  openSheet() {
    if (!get().sheet) set({ sheet: true })
  },

  closeSheet() {
    if (get().sheet) set({ sheet: false })
  },
}))

export function onToolChosen(listener: (tool: Tool, previous: Tool) => void): () => void {
  return useWorkspaceStore.subscribe(({ tool }, previous) => {
    if (tool !== previous.tool) listener(tool, previous.tool)
  })
}

useProtectionStore.subscribe(({ current }, previous) => {
  if (current === previous.current) return
  const { tool, left } = useWorkspaceStore.getState()
  if (toolWithheld(current, tool)) useWorkspaceStore.setState({ tool: 'select', left: left === 'edit' ? 'tools' : left })
})
