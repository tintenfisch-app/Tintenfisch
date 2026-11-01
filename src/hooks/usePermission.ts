import { useProtectionStore, withholds } from '@/stores/protectionStore.ts'
import { toolWithheld, type Tool } from '@/stores/workspaceStore.ts'
import type { EditRight } from '@/types/protocol.ts'

export function usePermission(right: EditRight): boolean {
  return useProtectionStore((store) => !withholds(store.current, right))
}

export function useToolPermission(): (tool: Tool) => boolean {
  const current = useProtectionStore((store) => store.current)
  return (tool) => !toolWithheld(current, tool)
}
