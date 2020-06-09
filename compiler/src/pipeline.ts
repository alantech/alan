interface Converter {
  fromFile(filename: string): string | Buffer
  fromString(str: string): string | Buffer
}
interface ConverterIntermediate {
  prev: string
  fromFile(filename: string): string | Buffer
  fromString(str: string): string | Buffer
}

type convert = [string, string, Converter]

interface OutMap {
  [out: string]: Converter
}

interface ConverterMap {
  [input: string]: OutMap
}

const buildPipeline = (converters: convert[]): ConverterMap => {
  // Get a unique set of inputs and outputs, and index the converters by their input and output
  const inputs = new Set()
  const outputs = new Set()
  const both = new Set()
  const byInput = new Map()
  const byOutput = new Map()
  const byBoth = new Map()
  converters.forEach(converter => {
    inputs.add(converter[0])
    outputs.add(converter[1])
    both.add(converter[0])
    both.add(converter[1])
    if (!byInput.has(converter[0])) {
      byInput.set(converter[0], [])
    }
    byInput.get(converter[0]).push(converter)
    if (!byOutput.has(converter[1])) {
      byOutput.set(converter[1], [])
    }
    byOutput.get(converter[1]).push(converter)
    byBoth.set(converter[0] + converter[1], converter[2])
  })
  // Compute the shortest path from every input to every output, or drop it if not possible
  const paths = {}
  inputs.forEach((input: string) => {
    outputs.forEach((output: string) => {
      // Skip identical inputs and outputs
      if (input === output) return
      // Short-circuit if a direct conversion is possible
      if (byBoth.has(input + output)) {
        paths[input] = {
          [output]: [input, output]
        }
        return
      }
      // Otherwise, scan through the graph using Djikstra's Algorithm
      const nodes = new Set()
      const dist = new Map()
      const prev = new Map()
      both.forEach(n => {
        nodes.add(n)
        dist.set(n, Infinity)
        prev.set(n, undefined)
      })
      dist.set(input, 0)
      let minDist = 0
      let minNode = input
      while (nodes.size > 0) {
        const n = minNode
        if (n === output) break
        nodes.delete(n)
        minNode = undefined
        minDist = Infinity
        // Find the smallest remaining distance node to continue the search
        nodes.forEach((node: string) => {
          if (dist.get(node) < minDist || minDist === Infinity) {
            minDist = dist.get(node)
            minNode = node
          }
        })
        if (byInput.has(n)) {
          byInput.get(n).map((r: convert) => r[1]).forEach((neighbor: string) => {
            const newDist = dist.get(n) + 1
            if (newDist < dist.get(neighbor)) {
              dist.set(neighbor, newDist)
              prev.set(neighbor, n)
            }
            if (newDist < minDist) {
              minDist = newDist
              minNode = neighbor
            }
          })
        }
      }
      const path = []
      let node = output
      while (node) {
        path.unshift(node)
        node = prev.get(node)
      }
      if (path.length < 2) return // Invalid/impossible path, skip it
      if (!paths[input]) paths[input] = {}
      paths[input][output] = path
    })
  })
  const lookup: ConverterMap = {}
  Object.keys(paths).forEach(i => {
    Object.keys(paths[i]).forEach(o => {
      if (!lookup[i]) lookup[i] = {}
      const c = paths[i][o].reduce((cumu: ConverterIntermediate, curr: string) => {
        if (!cumu.prev) return {
          prev: curr,
          fromFile: undefined,
          fromString: undefined,
        }
        const converter = byBoth.get(cumu.prev + curr)
        if (!cumu.fromFile) {
          return {
            prev: curr,
            fromFile: converter.fromFile,
            fromString: converter.fromString,
          }
        }
        return {
          prev: curr,
          fromFile: (filename: string) => converter.fromString(cumu.fromFile(filename)),
          fromString: (str: string) => converter.fromString(cumu.fromString(str)),
        }
      }, { prev: undefined, fromFile: undefined, fromString: undefined, })
      lookup[i][o] = {
        fromFile: c.fromFile,
        fromString: c.fromString,
      }
    })
  })
  return lookup
}

export default buildPipeline
