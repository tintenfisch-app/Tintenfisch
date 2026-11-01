import { afterEach, describe, expect, it, vi } from 'vitest'
import { DocumentChannel } from '@/services/DocumentChannel.ts'

const NAME = 'document-channel-test'
const opened: DocumentChannel[] = []

function tab() {
  const channel = new DocumentChannel(new BroadcastChannel(NAME))
  const others = vi.fn()
  const saved = vi.fn()
  channel.listen({ others, saved })
  opened.push(channel)
  return { channel, others, saved }
}

afterEach(() => {
  opened.splice(0).forEach((channel) => channel.close())
})

describe('DocumentChannel', () => {
  it('lets tabs with the same document find each other, whichever opened it first', async () => {
    const first = tab()
    const second = tab()
    const unrelated = tab()
    first.channel.follow('report')
    unrelated.channel.follow('other')
    second.channel.follow('report')
    await vi.waitFor(() => expect(first.others).toHaveBeenLastCalledWith(1))
    await vi.waitFor(() => expect(second.others).toHaveBeenLastCalledWith(1))
    expect(unrelated.others).not.toHaveBeenCalledWith(1)
    second.channel.follow(null)
    await vi.waitFor(() => expect(first.others).toHaveBeenLastCalledWith(0))
  })

  it('passes saves on to the other tabs with the same document only', async () => {
    const saving = tab()
    const watching = tab()
    const unrelated = tab()
    saving.channel.follow('report')
    watching.channel.follow('report')
    unrelated.channel.follow('other')
    saving.channel.announceSave(42)
    await vi.waitFor(() => expect(watching.saved).toHaveBeenCalledWith(42))
    expect(saving.saved).not.toHaveBeenCalled()
    expect(unrelated.saved).not.toHaveBeenCalled()
  })

  it('stays quiet without a broadcast channel', () => {
    const channel = new DocumentChannel(null)
    expect(() => {
      channel.follow('report')
      channel.announceSave(1)
      channel.close()
    }).not.toThrow()
  })
})
