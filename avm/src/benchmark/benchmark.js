const formatTime = (ms) => {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${ms / 1000.0}s`
  const minutes = Math.floor(ms / 60000)
  const remaining = ms - (minutes * 60000)
  return `${minutes}min ${remaining / 1000.0}s`
}

const square = (a) => a * a

const mxPlusB = (m, x, b) => m * x + b

const eField = (i, arr) => {
  const len = arr.length
  let out = 0.0
  for (let n = 0; n < len; n++) {
    const distance = i - n
    if (distance === 0) continue
    const sqdistance = distance * distance
    const invsqdistance = 1.0 / sqdistance
    const scaled = invsqdistance * arr[n]
    out = out + scaled
  }
  return out
}

const genRandArray = (size) => {
  const out = []
  for (let i = 0; i < size; i++) {
    out.push(Math.floor(Math.random() * 100000.0))
  }
  return out
}

const linSquare = (size) => {
  const data = genRandArray(size)
  const start = Date.now()
  const output = data.map(square)
  const end = Date.now()
  return end - start
}

const linMxPlusB = (size) => {
  const m = 2
  const b = 3
  const data = genRandArray(size)
  const start = Date.now()
  const output = data.map((x) => mxPlusB(m, x, b))
  const end = Date.now()
  return end - start
}

const linEField = (size) => {
  const data = genRandArray(size)
  const start = Date.now()
  const output = data.map((_, i) => eField(i, data))
  const end = Date.now()
  return end - start
}

const benchmark = () => {
  console.log("JS Benchmark!")
  console.log(`Squares 100-element array: ${formatTime(linSquare(100))}`)
  console.log(`Squares 10,000-element array: ${formatTime(linSquare(10000))}`)
  console.log(`Squares 1,000,000-element array: ${formatTime(linSquare(1000000))}`)
  console.log(`mx+b 100-element array: ${formatTime(linMxPlusB(100))}`)
  console.log(`mx+b 10,000-element array: ${formatTime(linMxPlusB(10000))}`)
  console.log(`mx+b 1,000,000-element array: ${formatTime(linMxPlusB(1000000))}`)
  console.log(`e-field 100-element array: ${formatTime(linEField(100))}`)
  console.log(`e-field 10,000-element array: ${formatTime(linEField(10000))}`)
  //console.log(`e-field 1,000,000-element array: ${formatTime(linEField(1000000))}`)
}

benchmark()