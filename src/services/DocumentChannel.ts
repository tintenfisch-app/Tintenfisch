export const DOCUMENT_CHANNEL = 'tintenfisch-documents'

type TabMessage = { kind: 'opened' | 'present' | 'closed'; file: string; tab: string } | { kind: 'saved'; file: string; tab: string; modified: number }

export interface DocumentChannelListener {
  others(count: number): void
  saved(modified: number): void
}

export class DocumentChannel {
  readonly #channel: BroadcastChannel | null
  readonly #tab = crypto.randomUUID()
  readonly #others = new Set<string>()
  #file: string | null = null
  #listener: DocumentChannelListener | null = null

  constructor(channel: BroadcastChannel | null) {
    this.#channel = channel
    if (channel) channel.onmessage = (event: MessageEvent<TabMessage>) => this.#receive(event.data)
  }

  listen(listener: DocumentChannelListener) {
    this.#listener = listener
  }

  follow(file: string | null) {
    if (file === this.#file) return
    if (this.#file) this.#post({ kind: 'closed', file: this.#file, tab: this.#tab })
    this.#file = file
    this.#others.clear()
    this.#listener?.others(0)
    if (file) this.#post({ kind: 'opened', file, tab: this.#tab })
  }

  announceSave(modified: number) {
    if (this.#file) this.#post({ kind: 'saved', file: this.#file, tab: this.#tab, modified })
  }

  leavePage() {
    if (this.#file) this.#post({ kind: 'closed', file: this.#file, tab: this.#tab })
  }

  returnToPage() {
    if (this.#file) this.#post({ kind: 'opened', file: this.#file, tab: this.#tab })
  }

  close() {
    this.follow(null)
    this.#channel?.close()
  }

  #post(message: TabMessage) {
    this.#channel?.postMessage(message)
  }

  #receive(message: TabMessage) {
    if (message.file !== this.#file || message.tab === this.#tab) return
    if (message.kind === 'saved') {
      this.#listener?.saved(message.modified)
      return
    }
    if (message.kind === 'closed') this.#others.delete(message.tab)
    else this.#others.add(message.tab)
    if (message.kind === 'opened') this.#post({ kind: 'present', file: message.file, tab: this.#tab })
    this.#listener?.others(this.#others.size)
  }
}

export const documentChannel = new DocumentChannel(typeof BroadcastChannel === 'undefined' ? null : new BroadcastChannel(DOCUMENT_CHANNEL))

globalThis.addEventListener?.('pagehide', () => documentChannel.leavePage())
globalThis.addEventListener?.('pageshow', (event) => {
  if ((event as PageTransitionEvent).persisted) documentChannel.returnToPage()
})
