const r = require('alan-js-runtime')
const _5c6ae27c_5c1a_4d62_a3d3_1bacddd9b4a7 = 1n
const _4f714a16_5a80_46fd_87b9_50c000aef256 = 2n
const _6e0270e5_f143_4957_a60a_349f453e9fa3 = 3n
const _fdbe80bf_540b_4962_b5df_4f87c4584b98 = 4n
const _2e95fc9d_cc29_4c4f_9e76_d95922d77af0 = 5n
const _3cfd284b_9c33_4cc7_9b3c_43362041e97e = 8n
const _2e064130_2a0c_432a_9a61_2bb3a577bf24 = ", "
const _06fccd4d_9e2a_40cb_9779_a16860a61f81 = "\n"
const _88f36611_edab_4a30_a85b_99980aec2fe1 = true
const _e97587d4_7900_489a_9443_c4d040a62e5a = false
const _a05c8745_308f_42c0_ac64_f5d6d54679a2 = 0n
r.on('_start', async () => {
    const _309f4c91_4afa_4f0e_a185_5330e94b99a9 = r.newarr(_2e95fc9d_cc29_4c4f_9e76_d95922d77af0)
    r.pusharr(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _5c6ae27c_5c1a_4d62_a3d3_1bacddd9b4a7, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
    r.pusharr(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _4f714a16_5a80_46fd_87b9_50c000aef256, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
    r.pusharr(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _6e0270e5_f143_4957_a60a_349f453e9fa3, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
    r.pusharr(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _fdbe80bf_540b_4962_b5df_4f87c4584b98, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
    r.pusharr(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _2e95fc9d_cc29_4c4f_9e76_d95922d77af0, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
    const _e458ab23_513f_45ef_9b41_e53593766472 = async (n) => {
        const _5759f090_336e_4bfa_84ba_c8131489aeff = r.okR(n, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
        const _840300e9_496b_4326_bcb7_1a825b206764 = r.okR(_4f714a16_5a80_46fd_87b9_50c000aef256, _3cfd284b_9c33_4cc7_9b3c_43362041e97e)
        const _098265b8_1ec7_4684_ae83_f1f5cdb2e7a2 = r.muli64(_5759f090_336e_4bfa_84ba_c8131489aeff, _840300e9_496b_4326_bcb7_1a825b206764)
        return _098265b8_1ec7_4684_ae83_f1f5cdb2e7a2
      }
    const _c6c3d126_6f62_4919_9262_7eb932ee498e = await r.map(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _e458ab23_513f_45ef_9b41_e53593766472)
    const _3330fca8_7731_4c6a_8421_b74f0540c684 = async (n) => {
        const _50678c00_ffbf_415a_b832_010ec8b07037 = r.i64str(n)
        return _50678c00_ffbf_415a_b832_010ec8b07037
      }
    const _33af3ee9_3f6a_427d_acff_80a1bba934a3 = await r.map(_309f4c91_4afa_4f0e_a185_5330e94b99a9, _3330fca8_7731_4c6a_8421_b74f0540c684)
    const _b0af798d_4cc1_4af3_bc71_b6a6ad79835d = r.join(_33af3ee9_3f6a_427d_acff_80a1bba934a3, _2e064130_2a0c_432a_9a61_2bb3a577bf24)
    const _fd41847e_761a_4ed9_b724_615407a6d195 = r.catstr(_b0af798d_4cc1_4af3_bc71_b6a6ad79835d, _06fccd4d_9e2a_40cb_9779_a16860a61f81)
    const _aaada60c_2492_48bf_95be_271765093f72 = r.stdoutp(_fd41847e_761a_4ed9_b724_615407a6d195)
    const _3e909609_66d1_4025_899a_d600169fd870 = async (n) => {
        let _dd734123_af82_40ce_a989_3f29fa90e7a7 = r.zeroed()
        let _846da82e_b729_4874_98fb_932ab4f382ef = r.copybool(_88f36611_edab_4a30_a85b_99980aec2fe1)
        const _10f137af_6981_4e5f_88e6_0371134b3e96 = r.isOk(n)
        const _2f54621a_61e1_4cc3_aabf_3bfee8074ae6 = async () => {
            const _c7b7c0a7_d59d_49a1_bedf_434b366280c8 = r.getR(n)
            const _d2e6497a_9961_40a4_a0a1_4bd6d6aa73d1 = r.i64str(_c7b7c0a7_d59d_49a1_bedf_434b366280c8)
            _dd734123_af82_40ce_a989_3f29fa90e7a7 = r.refv(_d2e6497a_9961_40a4_a0a1_4bd6d6aa73d1)
            _846da82e_b729_4874_98fb_932ab4f382ef = r.copybool(_e97587d4_7900_489a_9443_c4d040a62e5a)
          }
        const _4b573ffe_0c4e_431a_9284_f7c2286998f7 = await r.condfn(_10f137af_6981_4e5f_88e6_0371134b3e96, _2f54621a_61e1_4cc3_aabf_3bfee8074ae6)
        const _e2e3484f_79df_4a0e_a834_b17766e5959c = async () => {
            const _c2475257_03c9_4043_b9df_eb4cdfd11242 = r.notbool(_10f137af_6981_4e5f_88e6_0371134b3e96)
            const _9a698120_ea1b_423e_a8d3_d758269330f4 = async () => {
                const _e81cab9e_6bee_4451_993c_6f60f7b55bdd = r.noerr()
                const _853c4708_ff9a_4713_8265_c83d9fb8124c = r.getErr(n, _e81cab9e_6bee_4451_993c_6f60f7b55bdd)
                const _e245edc8_f16d_44b3_b825_b4020ad93021 = r.errorstr(_853c4708_ff9a_4713_8265_c83d9fb8124c)
                _dd734123_af82_40ce_a989_3f29fa90e7a7 = r.refv(_e245edc8_f16d_44b3_b825_b4020ad93021)
                _846da82e_b729_4874_98fb_932ab4f382ef = r.copybool(_e97587d4_7900_489a_9443_c4d040a62e5a)
              }
            const _525bc86f_7f9b_4b90_94cb_8b25730b002f = await r.condfn(_c2475257_03c9_4043_b9df_eb4cdfd11242, _9a698120_ea1b_423e_a8d3_d758269330f4)
          }
        const _27ed910a_00e7_416c_8b74_3cecb4046d9f = await r.condfn(_846da82e_b729_4874_98fb_932ab4f382ef, _e2e3484f_79df_4a0e_a834_b17766e5959c)
        return _dd734123_af82_40ce_a989_3f29fa90e7a7
      }
    const _adfec53d_fc65_4386_9bbf_a27fc3ea49cb = await r.map(_c6c3d126_6f62_4919_9262_7eb932ee498e, _3e909609_66d1_4025_899a_d600169fd870)
    const _87bfe943_aa8d_40ae_9442_bf97bcade8e0 = r.join(_adfec53d_fc65_4386_9bbf_a27fc3ea49cb, _2e064130_2a0c_432a_9a61_2bb3a577bf24)
    const _bc3cb28d_5f90_4cd7_a981_acf0e13b995b = r.catstr(_87bfe943_aa8d_40ae_9442_bf97bcade8e0, _06fccd4d_9e2a_40cb_9779_a16860a61f81)
    const _a3206d88_6f9b_442d_a240_f94d9f491adc = r.stdoutp(_bc3cb28d_5f90_4cd7_a981_acf0e13b995b)
    r.emit('exit', _a05c8745_308f_42c0_ac64_f5d6d54679a2)
  })
r.on('stdout', async (out) => {
    const _2d768846_4981_4abf_b782_d0e95dd924ab = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _2dec4996_1c6c_4d7f_a580_fdb5b891573f = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _18c0be10_6766_4dce_8f2d_a24e3548c473 = r.stderrp(err)
  })
r.emit('_start', undefined)
