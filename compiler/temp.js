const r = require('alan-js-runtime')
const _7690c180_3d26_4b27_a329_cff5233473dc = "hello, world!"
const _3e57da3a_d389_40b1_87a1_8eb117aaa969 = 0n
r.on('stderr', async (err) => {
    let _673c6244_7e87_4da9_93b2_ef2045c9a151 = r.stderrp(err)
  })
r.on('exit', async (status) => {
    let _1b205a68_b0f6_415b_ac27_397e7a26adf0 = r.exitop(status)
  })
r.on('stdout', async (out) => {
    let _354b8dd2_0ea2_42f8_ba77_365bf5daccd9 = r.stdoutp(out)
  })
r.on('_start', async () => {
    r.emit('stdout', _7690c180_3d26_4b27_a329_cff5233473dc)
    r.emit('exit', _3e57da3a_d389_40b1_87a1_8eb117aaa969)
  })
r.emit('_start', undefined)
