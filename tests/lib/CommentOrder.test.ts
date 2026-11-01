import { describe, expect, it } from 'vitest'
import { CommentOrder } from '@/lib/CommentOrder.ts'
import type { Comment } from '@/types/protocol.ts'

const page = (id: string) => ({ id, width: 595, height: 842, rotate: 0 })
const comment = (id: string, pageId: string): Comment => ({ id, pageId, subtype: 'Text', contents: '', author: '', date: '', rect: { x: 0, y: 0, width: 20, height: 20 }, color: null })

describe('CommentOrder', () => {
  it('numbers comments by where their page stands now and lists them in that order', () => {
    const comments = [comment('a', '2-0'), comment('b', '9-0'), comment('c', '5-0')]
    const numbered = CommentOrder.byPage(comments, [page('9-0'), page('5-0'), page('2-0')])
    expect(numbered.map(({ comment, pageNumber }) => [comment.id, pageNumber])).toEqual([['b', 1], ['c', 2], ['a', 3]])
  })

  it('takes the first place of a page listed twice and leaves out comments on pages that are gone', () => {
    const numbered = CommentOrder.byPage([comment('a', '2-0'), comment('b', '7-0')], [page('2-0'), page('3-0'), page('2-0')])
    expect(numbered.map(({ comment, pageNumber }) => [comment.id, pageNumber])).toEqual([['a', 1]])
  })
})
