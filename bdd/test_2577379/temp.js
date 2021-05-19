const r = require('alan-js-runtime')
const _6b377f51_4394_41e8_b3f8_40e9a830f83d = "foo"
const _89a48e8f_2462_4f28_b88b_f0682714fa33 = "bar"
const _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c = "\n"
const _095f9d07_d127_4f6b_b245_27970e0199ec = "baz"
const _7b6c5040_7224_4761_81a3_1f22d4849e5b = ""
const _38000408_b21c_42d8_aa5f_1bfabb3c1910 = "inc"
const _e25e37bc_3f3c_49ab_8cb9_cb918f6f03f8 = 0n
const _f4fc2399_c4e4_4de6_9e44_dd360f7910b0 = 100n
const _b63d5710_e6fc_491e_951c_2e2f685e386d = 200n
const _15ac0877_94fb_441a_8ec4_6a10fe2865f5 = 300n
const _3d7864cc_a8e9_448e_b637_423aaa68fb4a = 1n
const _a83ed39d_7263_4201_a2ce_1a6bc310fa31 = 8n
const _d2858d8a_e1d1_4f26_9d07_78493ae8d08e = 3n
r.on('_start', async () => {
    const _c25810c4_2d6e_444c_af98_49ba8a2d939f = r.dshas(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _dda4741f_72a9_4fc0_a8fd_d974e3158d76 = r.boolstr(_c25810c4_2d6e_444c_af98_49ba8a2d939f)
    const _553837fd_5a75_4ac2_97cd_24a71d55fd03 = r.catstr(_dda4741f_72a9_4fc0_a8fd_d974e3158d76, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _a4c5483f_7851_4dd1_a2c4_d33f3f29de85 = r.stdoutp(_553837fd_5a75_4ac2_97cd_24a71d55fd03)
    r.dssetv(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33, _095f9d07_d127_4f6b_b245_27970e0199ec)
    const _25934409_cb0a_47dd_b619_70650972f1df = r.dshas(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _d946aa99_7b8b_4de5_a85d_e85021a3c77f = r.boolstr(_25934409_cb0a_47dd_b619_70650972f1df)
    const _80bc2c40_4ac2_48f2_b908_3f1c308f7d96 = r.catstr(_d946aa99_7b8b_4de5_a85d_e85021a3c77f, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _1c4bc1f5_f3cf_48c3_a445_757308f7a1e5 = r.stdoutp(_80bc2c40_4ac2_48f2_b908_3f1c308f7d96)
    const _aa8d53a6_2f29_4850_aa8c_bbc17bd6321c = r.dsgetv(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _31dea767_d745_42aa_8ac6_4df3867b80d4 = r.getOrRS(_aa8d53a6_2f29_4850_aa8c_bbc17bd6321c, _7b6c5040_7224_4761_81a3_1f22d4849e5b)
    const _7b39306e_d2c4_46e0_a1db_41dc5a175e74 = r.catstr(_31dea767_d745_42aa_8ac6_4df3867b80d4, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _c00697fa_34a7_4add_adf7_dc199136b008 = r.stdoutp(_7b39306e_d2c4_46e0_a1db_41dc5a175e74)
    const _3af77391_8bc6_46b7_9d75_76c50b4f1c86 = r.dsdel(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _e95a49a3_5809_43b5_a1d8_ccf72243d1ca = r.dshas(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _41375c4a_cc3b_448f_91ca_43f4c6f87a14 = r.boolstr(_e95a49a3_5809_43b5_a1d8_ccf72243d1ca)
    const _52db6fdb_5fb7_4c00_90f6_485f81a7d7d5 = r.catstr(_41375c4a_cc3b_448f_91ca_43f4c6f87a14, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _7720cb82_f296_4f4c_beeb_39a28e1cb18a = r.stdoutp(_52db6fdb_5fb7_4c00_90f6_485f81a7d7d5)
    const _8abd0937_44cf_4cfe_937c_07509a60ff36 = r.dsgetv(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _89a48e8f_2462_4f28_b88b_f0682714fa33)
    const _2c4ed4c0_5e4e_413e_a2d9_a41befe39cc8 = r.getOrRS(_8abd0937_44cf_4cfe_937c_07509a60ff36, _7b6c5040_7224_4761_81a3_1f22d4849e5b)
    const _dc671167_a81f_4878_8e42_6b19be2a0337 = r.catstr(_2c4ed4c0_5e4e_413e_a2d9_a41befe39cc8, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _3628532c_ba1e_4369_8126_c110990bd99f = r.stdoutp(_dc671167_a81f_4878_8e42_6b19be2a0337)
    r.dssetf(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _38000408_b21c_42d8_aa5f_1bfabb3c1910, _e25e37bc_3f3c_49ab_8cb9_cb918f6f03f8)
    r.emit('waitAndInc', _f4fc2399_c4e4_4de6_9e44_dd360f7910b0)
    r.emit('waitAndInc', _b63d5710_e6fc_491e_951c_2e2f685e386d)
    r.emit('waitAndInc', _15ac0877_94fb_441a_8ec4_6a10fe2865f5)
  })
r.on('stdout', async (out) => {
    const _ea65e748_fa78_4b0a_9973_79cf8f9f2d6d = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _ca4324a2_6eb5_4ffa_9760_8de95a24ae86 = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _d6998820_0e00_4478_916d_4fafe6bd5f09 = r.stderrp(err)
  })
r.on('waitAndInc', async (ms) => {
    const _e148d6c1_0ebe_457a_a45c_97695e20090b = await r.waitop(ms)
    const _1df58369_a9fa_43f0_bea4_600daee0b643 = r.dsgetf(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _38000408_b21c_42d8_aa5f_1bfabb3c1910)
    const _69ae8773_5ecc_4a9c_bb29_d4d1b7a9dc51 = r.getOrR(_1df58369_a9fa_43f0_bea4_600daee0b643, _e25e37bc_3f3c_49ab_8cb9_cb918f6f03f8)
    let _cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c = r.reff(_69ae8773_5ecc_4a9c_bb29_d4d1b7a9dc51)
    const _10d23782_45c3_427b_8eb0_8fa43051df94 = r.okR(_cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c, _a83ed39d_7263_4201_a2ce_1a6bc310fa31)
    const _0db1aed8_f88c_4d54_b58f_f9b6226d6582 = r.okR(_3d7864cc_a8e9_448e_b637_423aaa68fb4a, _a83ed39d_7263_4201_a2ce_1a6bc310fa31)
    const _6bb0be80_39e4_4b5a_b32d_94dcaa8f912a = r.addi64(_10d23782_45c3_427b_8eb0_8fa43051df94, _0db1aed8_f88c_4d54_b58f_f9b6226d6582)
    const _c87d1372_319b_4ae6_a637_9a13396119e6 = r.getOrR(_6bb0be80_39e4_4b5a_b32d_94dcaa8f912a, _e25e37bc_3f3c_49ab_8cb9_cb918f6f03f8)
    _cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c = r.reff(_c87d1372_319b_4ae6_a637_9a13396119e6)
    const _bcc7f5b4_0c39_4c65_a43a_08e97aa5813d = r.i64str(_cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c)
    const _7ac8af11_9a86_48cb_acba_a6ed7fdafa2f = r.catstr(_bcc7f5b4_0c39_4c65_a43a_08e97aa5813d, _1423b02e_46aa_4c1b_9d6c_ee078b3cfe1c)
    const _b0ca097b_4705_4475_9b21_da9138bcbf25 = r.stdoutp(_7ac8af11_9a86_48cb_acba_a6ed7fdafa2f)
    r.dssetf(_6b377f51_4394_41e8_b3f8_40e9a830f83d, _38000408_b21c_42d8_aa5f_1bfabb3c1910, _cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c)
    const _99fdee7d_8e1d_43c2_bdd2_fdce34dd3d2a = r.eqi64(_cd6bb24d_4bd3_43ca_b40f_6d5cda5cd97c, _d2858d8a_e1d1_4f26_9d07_78493ae8d08e)
    const _55e8b45f_5f2b_48fe_9171_aa4bcc1f2a68 = async () => {
        r.emit('exit', _e25e37bc_3f3c_49ab_8cb9_cb918f6f03f8)
      }
    const _29c72402_a7e2_4e80_bfb4_9be305e75547 = await r.condfn(_99fdee7d_8e1d_43c2_bdd2_fdce34dd3d2a, _55e8b45f_5f2b_48fe_9171_aa4bcc1f2a68)
  })
r.emit('_start', undefined)
