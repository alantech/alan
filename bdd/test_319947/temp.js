const r = require('alan-js-runtime')
const _0e8628dd_8afb_4c49_a222_0ae11b710e7c = 3n
const _099540fb_f172_4ce3_90be_a7797027d889 = 4n
const _2d90b5d4_b3fd_4dba_920a_3d8ce219c893 = "\n"
const _bce6f2b2_cfaa_4997_af3f_7ea9e31c993b = 1n
const _93df8dc5_ad2e_4786_b367_0bc32587a0c7 = 2n
const _e6114e8d_d646_4ce9_af37_18facaf364f8 = 8n
const _116ee46c_cd0b_47da_bf28_2a59415da16d = 0n
const _f6ab86e5_b683_4027_b3b5_8d8f83dd682f = true
const _25e725dd_18dc_4646_af7b_40881e1ef280 = "array out-of-bounds access"
const _78fd2e48_2962_4c46_a477_a9038e753ead = false
const _186e5ae8_5083_42df_a7b4_46ca83c9b9c7 = ", "
r.on('_start', async () => {
    let _69664bc7_f29f_4c8a_b2d9_511b138ebc4e = r.reff(_0e8628dd_8afb_4c49_a222_0ae11b710e7c)
    const _273e9a24_535d_4f4b_8f1f_4f4b357ca4cb = r.copyi64(_69664bc7_f29f_4c8a_b2d9_511b138ebc4e)
    let _3f794d56_63ef_4216_99f0_3ca339321d68 = r.reff(_273e9a24_535d_4f4b_8f1f_4f4b357ca4cb)
    _69664bc7_f29f_4c8a_b2d9_511b138ebc4e = r.reff(_099540fb_f172_4ce3_90be_a7797027d889)
    const _4659f79b_8239_4dcc_a7e8_495a08eb230f = r.i64str(_69664bc7_f29f_4c8a_b2d9_511b138ebc4e)
    const _e0bce7df_f3b4_44b7_af7a_877ed557d371 = r.catstr(_4659f79b_8239_4dcc_a7e8_495a08eb230f, _2d90b5d4_b3fd_4dba_920a_3d8ce219c893)
    const _70678260_673b_4354_8fc6_443abeddd1a4 = r.stdoutp(_e0bce7df_f3b4_44b7_af7a_877ed557d371)
    const _c02efe6f_49b4_4b27_ad94_1e7161a54b64 = r.i64str(_3f794d56_63ef_4216_99f0_3ca339321d68)
    const _34dd8066_c7ff_424f_87b2_5d831dc19004 = r.catstr(_c02efe6f_49b4_4b27_ad94_1e7161a54b64, _2d90b5d4_b3fd_4dba_920a_3d8ce219c893)
    const _dbd8b67a_a0d8_4990_b64f_8eb9c8bb2e61 = r.stdoutp(_34dd8066_c7ff_424f_87b2_5d831dc19004)
    const _8f987ed8_dacd_4cc0_8b84_c3721b0b83c1 = r.newarr(_0e8628dd_8afb_4c49_a222_0ae11b710e7c)
    r.pusharr(_8f987ed8_dacd_4cc0_8b84_c3721b0b83c1, _bce6f2b2_cfaa_4997_af3f_7ea9e31c993b, _e6114e8d_d646_4ce9_af37_18facaf364f8)
    r.pusharr(_8f987ed8_dacd_4cc0_8b84_c3721b0b83c1, _93df8dc5_ad2e_4786_b367_0bc32587a0c7, _e6114e8d_d646_4ce9_af37_18facaf364f8)
    r.pusharr(_8f987ed8_dacd_4cc0_8b84_c3721b0b83c1, _0e8628dd_8afb_4c49_a222_0ae11b710e7c, _e6114e8d_d646_4ce9_af37_18facaf364f8)
    let _3aad79cd_bd14_4c14_9dc4_417936a1562c = r.refv(_8f987ed8_dacd_4cc0_8b84_c3721b0b83c1)
    const _bf0a53a9_c1bb_43b1_9a07_39d707eef914 = r.copyarr(_3aad79cd_bd14_4c14_9dc4_417936a1562c)
    let _5908e8de_d7d1_41f6_9739_5299ae9d4528 = r.refv(_bf0a53a9_c1bb_43b1_9a07_39d707eef914)
    let _ff8d636b_7bf7_4b82_8758_6c44232b9f7a = r.zeroed()
    let _5d2b7f1a_9b6f_4475_a512_752b7fb58769 = r.copybool(_f6ab86e5_b683_4027_b3b5_8d8f83dd682f)
    const _283cedb0_16e1_4a03_8da9_12460513db64 = r.lti64(_116ee46c_cd0b_47da_bf28_2a59415da16d, _116ee46c_cd0b_47da_bf28_2a59415da16d)
    const _5609562b_33a4_4ecb_8b4d_2bc4ad4a18dc = r.lenarr(_5908e8de_d7d1_41f6_9739_5299ae9d4528)
    const _752642cf_41d6_48b5_8861_4556e712f84a = r.gti64(_116ee46c_cd0b_47da_bf28_2a59415da16d, _5609562b_33a4_4ecb_8b4d_2bc4ad4a18dc)
    const _85f60dec_2b79_48f0_8adb_d98f6764e133 = r.orbool(_283cedb0_16e1_4a03_8da9_12460513db64, _752642cf_41d6_48b5_8861_4556e712f84a)
    const _bc1c9720_228b_4522_9974_24ba988d917b = async () => {
        const _3753515a_9d9b_4fce_b579_39753f3165d6 = r.err(_25e725dd_18dc_4646_af7b_40881e1ef280)
        _ff8d636b_7bf7_4b82_8758_6c44232b9f7a = r.refv(_3753515a_9d9b_4fce_b579_39753f3165d6)
        _5d2b7f1a_9b6f_4475_a512_752b7fb58769 = r.copybool(_78fd2e48_2962_4c46_a477_a9038e753ead)
      }
    const _47d42e7d_de9a_4ac7_b9b8_39cb921ecc67 = await r.condfn(_85f60dec_2b79_48f0_8adb_d98f6764e133, _bc1c9720_228b_4522_9974_24ba988d917b)
    const _b5928652_1853_49e6_a778_40aece35ab73 = async () => {
        const _05531d3f_0b72_42a9_bedc_b29fb77c094f = r.notbool(_85f60dec_2b79_48f0_8adb_d98f6764e133)
        const _1bead41e_c76f_4c5c_8f0c_5b26a426509e = async () => {
            r.copytof(_5908e8de_d7d1_41f6_9739_5299ae9d4528, _116ee46c_cd0b_47da_bf28_2a59415da16d, _93df8dc5_ad2e_4786_b367_0bc32587a0c7)
            const _cbd19f92_35fe_46e8_a0f4_aedfe594ade9 = r.someM(_5908e8de_d7d1_41f6_9739_5299ae9d4528, _116ee46c_cd0b_47da_bf28_2a59415da16d)
            _ff8d636b_7bf7_4b82_8758_6c44232b9f7a = r.refv(_cbd19f92_35fe_46e8_a0f4_aedfe594ade9)
            _5d2b7f1a_9b6f_4475_a512_752b7fb58769 = r.copybool(_78fd2e48_2962_4c46_a477_a9038e753ead)
          }
        const _0f73811a_a362_45ac_933e_29ab0abb69d1 = await r.condfn(_05531d3f_0b72_42a9_bedc_b29fb77c094f, _1bead41e_c76f_4c5c_8f0c_5b26a426509e)
      }
    const _621bca5e_c27c_430d_9801_684e4fb3a624 = await r.condfn(_5d2b7f1a_9b6f_4475_a512_752b7fb58769, _b5928652_1853_49e6_a778_40aece35ab73)
    const _0f98fcc8_8a84_4da9_8b7c_173092747fe5 = async (val) => {
        const _0c2a1f41_a569_4eb6_8fdb_1b551ea166f4 = r.i64str(val)
        return _0c2a1f41_a569_4eb6_8fdb_1b551ea166f4
      }
    const _4b2a1f0a_b19b_47de_9676_7f47015a1b6c = await r.map(_3aad79cd_bd14_4c14_9dc4_417936a1562c, _0f98fcc8_8a84_4da9_8b7c_173092747fe5)
    const _36612565_9aa1_4d38_b303_be82cfef9003 = r.join(_4b2a1f0a_b19b_47de_9676_7f47015a1b6c, _186e5ae8_5083_42df_a7b4_46ca83c9b9c7)
    const _c73ab567_1780_4afc_899e_829e8fcc875e = r.catstr(_36612565_9aa1_4d38_b303_be82cfef9003, _2d90b5d4_b3fd_4dba_920a_3d8ce219c893)
    const _26f482d0_f79d_4c31_8a1e_cb0b39f076b8 = r.stdoutp(_c73ab567_1780_4afc_899e_829e8fcc875e)
    const _1f51979e_b692_4df2_89d3_905025f5d03e = async (val) => {
        const _9caecba7_58cc_479c_8209_4d6fa7834f9d = r.i64str(val)
        return _9caecba7_58cc_479c_8209_4d6fa7834f9d
      }
    const _0ecc5ba1_241d_45bc_96f2_4d82cbae6123 = await r.map(_5908e8de_d7d1_41f6_9739_5299ae9d4528, _1f51979e_b692_4df2_89d3_905025f5d03e)
    const _4dc4a949_b189_4dd8_897d_1005fe630626 = r.join(_0ecc5ba1_241d_45bc_96f2_4d82cbae6123, _186e5ae8_5083_42df_a7b4_46ca83c9b9c7)
    const _c0ae6e40_6e02_4e5f_a774_8b6b41a5758f = r.catstr(_4dc4a949_b189_4dd8_897d_1005fe630626, _2d90b5d4_b3fd_4dba_920a_3d8ce219c893)
    const _18fdb2c8_dc03_48cd_b0c3_65161709de89 = r.stdoutp(_c0ae6e40_6e02_4e5f_a774_8b6b41a5758f)
    r.emit('exit', _116ee46c_cd0b_47da_bf28_2a59415da16d)
  })
r.on('stdout', async (out) => {
    const _a02d6714_3aa8_4e15_b2b6_daf02296aca8 = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _92a00c94_1313_4c6b_8a88_a2d6041b791c = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _fa725dfc_1881_4563_ae6b_ca46511df02e = r.stderrp(err)
  })
r.emit('_start', undefined)
