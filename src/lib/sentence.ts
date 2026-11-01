export function sentence(text: string): string {
  const trimmed = text.trim()
  if (trimmed === '') return trimmed
  const capitalized = `${trimmed[0].toUpperCase()}${trimmed.slice(1)}`
  return /[.!?]$/.test(capitalized) ? capitalized : `${capitalized}.`
}
