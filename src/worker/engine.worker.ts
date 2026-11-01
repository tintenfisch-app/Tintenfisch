import initializeWasm, { Engine } from '@wasm/tintenfisch_wasm.js'
import wasmUrl from '@wasm/tintenfisch_wasm_bg.wasm?url'
import { EngineHost } from '@/worker/EngineHost.ts'
import { WorkerRuntime } from '@/worker/WorkerRuntime.ts'
import type { WorkerMessage } from '@/types/protocol.ts'

const scope = self as unknown as DedicatedWorkerGlobalScope

async function fetchFont(fileName: string): Promise<Uint8Array | undefined> {
  const response = await fetch(`${import.meta.env.BASE_URL}fonts/${fileName}`)
  return response.ok ? new Uint8Array(await response.arrayBuffer()) : undefined
}

async function startEngine(): Promise<EngineHost> {
  await initializeWasm({ module_or_path: wasmUrl })
  const engine = new Engine()
  await Promise.all(
    Engine.fallbackFontFiles().map(async (fileName) => {
      const bytes = await fetchFont(fileName)
      if (bytes) engine.registerFont(fileName, bytes)
    }),
  )
  return new EngineHost(engine, fetchFont)
}

const channel = new MessageChannel()
const pending: Array<() => void> = []
channel.port1.onmessage = () => pending.shift()?.()

const runtime = new WorkerRuntime(
  startEngine(),
  (response, transfer) => scope.postMessage(response, transfer),
  (resume) => {
    pending.push(resume)
    channel.port2.postMessage(null)
  },
)

scope.onmessage = (event: MessageEvent<WorkerMessage>) => runtime.receive(event.data)
