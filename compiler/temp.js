const r = require('alan-js-runtime')
const _dd75c9e6_cd66_4a65_94f7_0f581f7fa71d = "hello world"
const _2e848184_f574_4d4c_a920_f60636bb6d24 = 1n
const _0d64239f_5578_42ad_ab66_c25be80c701a = 0
r.on('stderr', async (err) => {
    let _858f3442_aebc_43d7_a386_a2b19b4cac94 = r.stderrp(err)
  })
r.on('exit', async (status) => {
    let _621c1636_0d94_4902_83fe_6ed04edf6225 = r.exitop(status)
  })
r.on('stdout', async (out) => {
    let _347f279a_ceaf_4ee7_8da2_a1d8b0a1e026 = r.stdoutp(out)
  })
r.on('_start', async () => {
    let _3334772b_9c59_4365_b4e4_a41cffc00a43 = r.copystr(_dd75c9e6_cd66_4a65_94f7_0f581f7fa71d)
    let _73cde736_ce15_46b6_8237_e010879b9109 = r.refv(_3334772b_9c59_4365_b4e4_a41cffc00a43)
    r.emit('stdout', _73cde736_ce15_46b6_8237_e010879b9109)
    let _3bb4a3a6_1b27_425b_93a9_3441a7d57e9b = r.copyi64(_2e848184_f574_4d4c_a920_f60636bb6d24)
    let _3bfb0fd7_7d54_4aba_80b2_05abfff66b59 = r.i64str(_3bb4a3a6_1b27_425b_93a9_3441a7d57e9b)
    let _45e31a9c_ce76_40fe_986a_201ceac25de7 = r.refv(_3bfb0fd7_7d54_4aba_80b2_05abfff66b59)
    r.emit('stdout', _45e31a9c_ce76_40fe_986a_201ceac25de7)
    let _a862fe7a_c362_406f_a5d0_d8ad87d981ec = r.copyi8(_0d64239f_5578_42ad_ab66_c25be80c701a)
    r.emit('exit', _a862fe7a_c362_406f_a5d0_d8ad87d981ec)
  })
r.emit('_start', undefined)
