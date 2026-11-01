const SIGNATURE = [0x25, 0x50, 0x44, 0x46, 0x2d]
const SEARCHED = 1024

export function hasPdfHeader(bytes: Uint8Array): boolean {
  const end = Math.min(bytes.byteLength, SEARCHED) - SIGNATURE.length
  for (let start = 0; start <= end; start++) {
    if (SIGNATURE.every((byte, offset) => bytes[start + offset] === byte)) return true
  }
  return false
}
