const r = require('alan-js-runtime')
const _b3929a91_ef31_4633_9e31_c537e62cd169 = "rm -rf dependencies"
const _857416a7_ca1d_485a_81fd_19a9fb94e0a3 = "mkdir dependencies"
const _61920894_388b_409e_8279_b5c53e639591 = "https://github.com/alantech/hellodep"
const _23b75683_4e58_4dd8_9c11_e99545c7152c = "/"
const _bb67a139_f2a8_4d1e_814d_e0f37b4cecc0 = 1n
const _fc52cc58_cb00_4a94_83f9_d3718abe003c = 8n
const _6d98bf87_ccc4_453a_a105_4333954ebf1a = ""
const _26f95ebb_4903_472c_9c8d_e7351c073455 = 2n
const _d302525a_cea8_47f9_8015_fecfd3614984 = "/dependencies/"
const _fe364758_d69d_4088_ae9f_e7dc860aa9eb = "rm -rf ."
const _ae9e277b_cae8_4143_980e_42002f2e11dd = "git clone "
const _01f13794_c66a_4233_94f7_6bef7a6b4fb9 = " ."
const _942a517a_a2ec_4d81_90e6_a83b63ec69c6 = "\n"
const _b6604bc5_bf38_4696_915d_3f492b154983 = "/.git"
const _87f237b9_b8da_4e9c_8c96_e1f9d1b01ea8 = 0n
r.on('_start', async () => {
    const _1261d078_708d_477e_b8d3_2ac133ecfb29 = async (n) => {
        const _199e1aed_72c9_425b_a073_1ba3a18a1ad4 = await r.execop(n)
        return _199e1aed_72c9_425b_a073_1ba3a18a1ad4
      }
    const _88c8c046_0284_4f78_9971_c4694d6ccdbf = await r.syncop(_1261d078_708d_477e_b8d3_2ac133ecfb29, _b3929a91_ef31_4633_9e31_c537e62cd169)
    const _7e85193d_34db_4aa6_b7d8_be0a506ce5cc = async (n) => {
        const _94c82198_7064_425e_bb2d_729979300bae = await r.execop(n)
        return _94c82198_7064_425e_bb2d_729979300bae
      }
    const _58590b4c_85fd_441a_81c1_b053a10165f6 = await r.syncop(_7e85193d_34db_4aa6_b7d8_be0a506ce5cc, _857416a7_ca1d_485a_81fd_19a9fb94e0a3)
    r.emit('install', undefined)
  })
r.on('stdout', async (out) => {
    const _0315e118_4e53_4082_a1aa_bb6f3b9c9022 = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _9928fae3_5e5e_4906_add7_d345373ea67a = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _0cd5fb03_5a7c_4803_a498_186f6d2d2dc3 = r.stderrp(err)
  })
r.on('install', async () => {
    const _9738fec0_d51b_4231_9b35_73e709b4558a = r.split(_61920894_388b_409e_8279_b5c53e639591, _23b75683_4e58_4dd8_9c11_e99545c7152c)
    const _4fe9f880_aca2_4a26_a65c_766f071557b6 = r.lenarr(_9738fec0_d51b_4231_9b35_73e709b4558a)
    const _f2b36c1b_b40b_42b2_afbc_d378d59a7cf8 = r.okR(_4fe9f880_aca2_4a26_a65c_766f071557b6, _fc52cc58_cb00_4a94_83f9_d3718abe003c)
    const _da49d7d4_e397_45f1_b612_21b2c63dd52f = r.okR(_bb67a139_f2a8_4d1e_814d_e0f37b4cecc0, _fc52cc58_cb00_4a94_83f9_d3718abe003c)
    const _8c3bf8a9_f139_42d8_a2b7_9e4844c46adf = r.subi64(_f2b36c1b_b40b_42b2_afbc_d378d59a7cf8, _da49d7d4_e397_45f1_b612_21b2c63dd52f)
    const _5e7e3dba_b6b8_4425_8f57_3f5aed11d9c2 = r.resfrom(_9738fec0_d51b_4231_9b35_73e709b4558a, _8c3bf8a9_f139_42d8_a2b7_9e4844c46adf)
    const _03f6d776_18f6_4a3f_807a_c58e34dd3070 = r.getOrRS(_5e7e3dba_b6b8_4425_8f57_3f5aed11d9c2, _6d98bf87_ccc4_453a_a105_4333954ebf1a)
    const _3d25e78c_8b2f_4930_8d27_b2c9f60ed55b = r.lenarr(_9738fec0_d51b_4231_9b35_73e709b4558a)
    const _23041aa6_3e0d_4e09_af86_7698c9a2292b = r.okR(_3d25e78c_8b2f_4930_8d27_b2c9f60ed55b, _fc52cc58_cb00_4a94_83f9_d3718abe003c)
    const _755a480a_cb28_4b85_82ff_ea76d35bfe64 = r.okR(_26f95ebb_4903_472c_9c8d_e7351c073455, _fc52cc58_cb00_4a94_83f9_d3718abe003c)
    const _45117fce_cc43_47e6_a7e8_db318c13b449 = r.subi64(_23041aa6_3e0d_4e09_af86_7698c9a2292b, _755a480a_cb28_4b85_82ff_ea76d35bfe64)
    const _1f9caac8_b3e4_4eb0_ad82_d91972a9b9ff = r.resfrom(_9738fec0_d51b_4231_9b35_73e709b4558a, _45117fce_cc43_47e6_a7e8_db318c13b449)
    const _b503293b_a862_4e6d_96ea_a712e08bc51b = r.getOrRS(_1f9caac8_b3e4_4eb0_ad82_d91972a9b9ff, _6d98bf87_ccc4_453a_a105_4333954ebf1a)
    const _5e239e24_0242_4163_ab3c_83be1146b011 = r.catstr(_d302525a_cea8_47f9_8015_fecfd3614984, _b503293b_a862_4e6d_96ea_a712e08bc51b)
    const _8291391e_977c_44e4_863b_0c9a5a13c417 = r.catstr(_5e239e24_0242_4163_ab3c_83be1146b011, _23b75683_4e58_4dd8_9c11_e99545c7152c)
    const _eadb738e_a1af_458f_8775_4002653f7962 = r.catstr(_8291391e_977c_44e4_863b_0c9a5a13c417, _03f6d776_18f6_4a3f_807a_c58e34dd3070)
    const _2e6f295c_dafe_4540_8296_a5de91f3383d = r.catstr(_fe364758_d69d_4088_ae9f_e7dc860aa9eb, _eadb738e_a1af_458f_8775_4002653f7962)
    const _6865c013_8524_4881_bc3f_bc9c98caeb48 = async (n) => {
        const _a9480eff_1f4c_41b1_855b_3b08ea4f55b2 = await r.execop(n)
        return _a9480eff_1f4c_41b1_855b_3b08ea4f55b2
      }
    const _257a0f27_04cf_435c_9eee_58b51fa59c38 = await r.syncop(_6865c013_8524_4881_bc3f_bc9c98caeb48, _2e6f295c_dafe_4540_8296_a5de91f3383d)
    const _c8bd42dc_1759_4d18_85ae_f2b4e6ae8f55 = r.catstr(_ae9e277b_cae8_4143_980e_42002f2e11dd, _61920894_388b_409e_8279_b5c53e639591)
    const _450b791c_ff6e_45f2_a73f_073097459a0e = r.catstr(_c8bd42dc_1759_4d18_85ae_f2b4e6ae8f55, _01f13794_c66a_4233_94f7_6bef7a6b4fb9)
    const _a84dcaeb_33be_46e7_a544_6f3c88aa36da = r.catstr(_450b791c_ff6e_45f2_a73f_073097459a0e, _eadb738e_a1af_458f_8775_4002653f7962)
    const _028bae8d_8496_4f89_8ad3_57b856a04b40 = async (n) => {
        const _d9a98d1f_d915_4e1f_ad36_e5688a6b631c = await r.execop(n)
        return _d9a98d1f_d915_4e1f_ad36_e5688a6b631c
      }
    const _cdfcd5f6_39db_4604_a6ca_5c53958dee35 = await r.syncop(_028bae8d_8496_4f89_8ad3_57b856a04b40, _a84dcaeb_33be_46e7_a544_6f3c88aa36da)
    const _b656a149_81a3_4649_a2c2_9aa7af5ab459 = r.register(_cdfcd5f6_39db_4604_a6ca_5c53958dee35, _26f95ebb_4903_472c_9c8d_e7351c073455)
    const _0b3687a6_39be_4727_9a5c_ec0c5ba667dd = r.catstr(_b656a149_81a3_4649_a2c2_9aa7af5ab459, _942a517a_a2ec_4d81_90e6_a83b63ec69c6)
    const _4f3ab70f_6210_4f14_84b7_6f4dcf8cf5f0 = r.stdoutp(_0b3687a6_39be_4727_9a5c_ec0c5ba667dd)
    const _827369bd_b0fb_4066_b1fd_2fd3caacd24a = r.catstr(_fe364758_d69d_4088_ae9f_e7dc860aa9eb, _eadb738e_a1af_458f_8775_4002653f7962)
    const _4b86733d_c232_4bb3_9c3b_c7abdd44c501 = r.catstr(_827369bd_b0fb_4066_b1fd_2fd3caacd24a, _b6604bc5_bf38_4696_915d_3f492b154983)
    const _d52a3cbf_3781_4c3d_94e4_7205b714fe30 = async (n) => {
        const _c9dbb985_822d_48a7_ac78_7e2ccfc4c9d0 = await r.execop(n)
        return _c9dbb985_822d_48a7_ac78_7e2ccfc4c9d0
      }
    const _8e98fae9_5f7e_4e49_bb43_f46202a35c4b = await r.syncop(_d52a3cbf_3781_4c3d_94e4_7205b714fe30, _4b86733d_c232_4bb3_9c3b_c7abdd44c501)
    r.emit('exit', _87f237b9_b8da_4e9c_8c96_e1f9d1b01ea8)
  })
r.emit('_start', undefined)
