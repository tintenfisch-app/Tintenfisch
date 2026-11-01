import 'fake-indexeddb/auto'
import { afterEach, vi } from 'vitest'
import type * as Zustand from 'zustand'

const resets = vi.hoisted(() => new Set<() => void>())

vi.mock('zustand', async (importOriginal) => {
  const zustand = await importOriginal<typeof Zustand>()
  const remembered = (creator: Zustand.StateCreator<unknown>) => {
    const store = zustand.create<unknown>((set, get, api) => {
      api.getInitialState = get
      return creator(set, get, api)
    })
    const initial = store.getState()
    resets.add(() => store.setState(initial, true))
    return store
  }
  const create = ((creator?: Zustand.StateCreator<unknown>) => (creator ? remembered(creator) : remembered)) as typeof zustand.create
  return { ...zustand, create }
})

afterEach(() => {
  vi.useRealTimers()
  resets.forEach((reset) => reset())
})
