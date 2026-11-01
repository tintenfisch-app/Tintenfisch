import type { Comment, PageSummary } from '@/types/protocol.ts'

export interface NumberedComment {
  comment: Comment
  pageNumber: number
}

export class CommentOrder {
  static byPage(comments: readonly Comment[], pages: readonly PageSummary[]): NumberedComment[] {
    const numbers = new Map<string, number>()
    pages.forEach((page, index) => {
      if (!numbers.has(page.id)) numbers.set(page.id, index + 1)
    })
    return comments
      .flatMap((comment) => {
        const pageNumber = numbers.get(comment.pageId)
        return pageNumber === undefined ? [] : [{ comment, pageNumber }]
      })
      .sort((first, second) => first.pageNumber - second.pageNumber)
  }
}
