import { gzipSync, } from 'zlib'
import { readFileSync, } from 'fs'

export const fromFile = (filename: string): Buffer => gzipSync(readFileSync(filename), { level: 9 })
export const fromString = (buf: string | Buffer): Buffer => gzipSync(buf, { level: 9 })
