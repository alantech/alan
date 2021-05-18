const r = require('alan-js-runtime')
const _80f87d5b_987b_45dc_97ad_36c8b20aea05 = "hello, world!"
const _1fc7e135_e673_443b_98d9_614082bc4a3c = "\n"
const _920d6630_3ac2_4b41_bc68_52845b6e12ef = true
const _8694de7d_017e_4b2e_98b3_6381280eead8 = 100n
const _8ce68d0f_1e5c_428d_a379_aee1d0d25454 = 0
r.on('stderr', async (err) => {
    let _6e2c823a_0d10_4b37_afc6_589860d4f165 = r.stderrp(err)
  })
r.on('exit', async (status) => {
    let _313923a1_eb1c_4df7_9590_746c8996cdfe = r.exitop(status)
  })
r.on('stdout', async (out) => {
    let _ac65084c_77e9_4b70_a5ff_3dbb9e6c6880 = r.stdoutp(out)
  })
r.on('_start', async () => {
    let _e11d3e42_f996_43b1_88b1_e95e2ecd281b = r.copystr(_80f87d5b_987b_45dc_97ad_36c8b20aea05)
    let _a1e7d56f_5416_40f1_8106_d8ab276bf412 = r.copystr(_1fc7e135_e673_443b_98d9_614082bc4a3c)
    let _d940dd3d_859a_41d8_a117_8f5804beaf5b = r.catstr(_e11d3e42_f996_43b1_88b1_e95e2ecd281b, _a1e7d56f_5416_40f1_8106_d8ab276bf412)
    let _240b2f67_027a_42c2_8c1a_776e7bd8ed14 = r.refv(_d940dd3d_859a_41d8_a117_8f5804beaf5b)
    r.emit('stdout', _240b2f67_027a_42c2_8c1a_776e7bd8ed14)
    let _c7da7db2_d684_4b52_bc75_b7b46248d933 = r.copybool(_920d6630_3ac2_4b41_bc68_52845b6e12ef)
    let _4dbef199_61f8_419f_9c78_edb026b6f9e9 = r.notbool(_c7da7db2_d684_4b52_bc75_b7b46248d933)
    let _b3423481_46c3_4bc1_936d_16eb15a9e410 = r.reff(_4dbef199_61f8_419f_9c78_edb026b6f9e9)
    let _6618cd4a_99cd_495a_95ed_8f428f89a5eb = r.boolstr(_b3423481_46c3_4bc1_936d_16eb15a9e410)
    let _559c836c_080e_4829_b031_61aed000857d = r.refv(_6618cd4a_99cd_495a_95ed_8f428f89a5eb)
    let _e0f3d3ab_e5d6_407b_b1a3_51245b862b9c = r.copystr(_1fc7e135_e673_443b_98d9_614082bc4a3c)
    let _e7ffc1ff_8b13_4842_99c1_1990ec0cff36 = r.catstr(_559c836c_080e_4829_b031_61aed000857d, _e0f3d3ab_e5d6_407b_b1a3_51245b862b9c)
    let _6aa435c5_16a1_46b4_98e1_c6b7e8937b3a = r.refv(_e7ffc1ff_8b13_4842_99c1_1990ec0cff36)
    r.emit('stdout', _6aa435c5_16a1_46b4_98e1_c6b7e8937b3a)
    let _ae38a92c_9969_477d_a674_13437846077b = r.copyi64(_8694de7d_017e_4b2e_98b3_6381280eead8)
    let _e8edf9fc_f1ae_4ee2_bc01_009f1b9dc36c = await r.waitop(_ae38a92c_9969_477d_a674_13437846077b)
    let _95991f2b_a021_4b03_bc2e_10b89e55cd1c = r.copyi8(_8ce68d0f_1e5c_428d_a379_aee1d0d25454)
    r.emit('exit', _95991f2b_a021_4b03_bc2e_10b89e55cd1c)
  })
r.emit('_start', undefined)
