import { describe, expect, it } from 'vitest'
import { MARKUP_COLORS, useWorkspaceStore } from '@/stores/workspaceStore.ts'

const workspace = () => useWorkspaceStore.getState()

describe('workspaceStore', () => {
  it('keeps a color for highlights apart from the one every other tool draws in', () => {
    expect(workspace().tool).toBe('select')
    workspace().choose('ink')
    workspace().setColor(MARKUP_COLORS[1])
    workspace().choose('highlight')
    expect(workspace().activeColor()).toBe(MARKUP_COLORS[3])
    workspace().setColor(MARKUP_COLORS[2])
    workspace().choose('rectangle')
    expect(workspace().activeColor()).toBe(MARKUP_COLORS[1])
    expect(workspace()).toMatchObject({ color: MARKUP_COLORS[1], highlightColor: MARKUP_COLORS[2] })
  })

  it('keeps one panel open on each side and always shows a left panel', () => {
    expect(workspace().left).toBe('tools')
    workspace().toggleRight('comments')
    workspace().toggleRight('pages')
    expect(workspace()).toMatchObject({ left: 'tools', right: 'pages', sheet: false })
    workspace().toggleRight('pages')
    workspace().showLeft('edit')
    expect(workspace()).toMatchObject({ left: 'edit', right: null, sheet: false })
  })

  it('stays on the edit panel when a redaction tool is picked from it', () => {
    workspace().showLeft('edit')
    workspace().choose('redactText')
    expect(workspace()).toMatchObject({ tool: 'redactText', left: 'edit' })
    workspace().choose('redactArea')
    expect(workspace()).toMatchObject({ tool: 'redactArea', left: 'edit' })
    workspace().choose('ink')
    expect(workspace()).toMatchObject({ tool: 'ink', left: 'tools' })
  })

  it('leaves a tool for select, or for edit when the edit panel is open', () => {
    workspace().choose('rectangle')
    workspace().leaveTool()
    expect(workspace().tool).toBe('select')
    workspace().choose('addText')
    workspace().leaveTool()
    expect(workspace()).toMatchObject({ tool: 'edit', left: 'edit' })
    workspace().choose('redactArea')
    workspace().leaveTool()
    expect(workspace()).toMatchObject({ tool: 'edit', left: 'edit' })
    workspace().leaveTool()
    expect(workspace().tool).toBe('edit')
  })

  it('tells listeners only about real changes', () => {
    let calls = 0
    const stop = useWorkspaceStore.subscribe(() => calls++)
    workspace().showRight('bookmarks')
    workspace().showRight('bookmarks')
    stop()
    expect(calls).toBe(1)
  })

  it('closes the sheet when a right panel takes over', () => {
    workspace().openSheet()
    expect(workspace().sheet).toBe(true)
    workspace().showRight('comments')
    expect(workspace().sheet).toBe(false)
  })

  it('keeps the edit panel and the editing tools in step', () => {
    workspace().choose('addText')
    expect(workspace().left).toBe('edit')
    workspace().showLeft('tools')
    expect(workspace().tool).toBe('select')
    workspace().showLeft('edit')
    expect(workspace().tool).toBe('edit')
    workspace().choose('ink')
    expect(workspace().left).toBe('tools')
  })
})
