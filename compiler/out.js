const r = require('alan-js-runtime')
const _6e93f97d_2ca8_4739_91fa_b352b6638aba = "https://raw.githubusercontent.com/ledwards/advent-2020/main/data/expenses.txt"
const _a4c4d844_5bf3_4b2d_bbae_a031a7587c6f = ""
const _b4ad8cdc_6956_43fa_8432_be19d6e1ba70 = "\n"
const _c6b87a56_c688_4068_acb4_2181b7030f8d = 2020n
const _321eadd3_6654_47bd_9d6f_b63467f332a1 = 8n
const _7c8033d2_d2ce_434b_85a5_514f856e988d = true
const _e8a891d1_b130_4a2e_bfd8_f075cb377bc5 = false
const _6e30a89b_48fb_4e4e_be4a_49da90240c76 = 0n
const _c8a86d92_61e0_42fc_af22_b79126fb87fd = 1n
const _5f2c1ef5_c75b_468e_a7b1_f9c28bf2266c = 128n
const _8020acc8_fefc_4e4b_84a9_c3c5619d7a55 = 2n
const _4da0a50d_5540_4b4a_83bc_870394aa1af3 = 3n
const _786b0b86_bbfb_4a93_8333_148a8d3e3f9e = 200n
const _b884828e_37e8_4f04_9f22_907b4c59464d = "Content-Length"
const _77545b46_fe93_49e0_a5ad_b39b58aeb193 = "0"
const _65122beb_adcd_45eb_b688_5467e81d256d = "array out-of-bounds access"
const _bb50ad84_e1c4_4b58_93cc_4389b6d14900 = 4n
r.on('_start', async () => {
    const _019dc413_eb6c_4f08_9888_bb8780a8bd19 = await r.httpget(_6e93f97d_2ca8_4739_91fa_b352b6638aba)
    const _d0e64198_2708_4c16_88e7_183763cf07f6 = r.getOrRS(_019dc413_eb6c_4f08_9888_bb8780a8bd19, _a4c4d844_5bf3_4b2d_bbae_a031a7587c6f)
    const _7bf06c73_18fd_4bca_86bc_160fab4de89b = r.trim(_d0e64198_2708_4c16_88e7_183763cf07f6)
    const _3cb20afb_bb8a_452b_9396_9f5d5c0ac339 = r.split(_7bf06c73_18fd_4bca_86bc_160fab4de89b, _b4ad8cdc_6956_43fa_8432_be19d6e1ba70)
    const _683a1c01_9334_42d9_8d53_5eb475c29b58 = async (n) => {
        const _69350512_d68b_411d_9029_9e4a5527110c = r.stri64(n)
        return _69350512_d68b_411d_9029_9e4a5527110c
      }
    const _a7707284_0470_4ba0_8b35_ebcfa05bf521 = await r.map(_3cb20afb_bb8a_452b_9396_9f5d5c0ac339, _683a1c01_9334_42d9_8d53_5eb475c29b58)
    const _19debd25_6d50_479a_b5e6_d263071af1f7 = async (a) => {
        const _1299d122_b8c9_41a9_9290_ab6ca5484277 = async (b) => {
            const _b81a7e55_fddd_4597_ae7d_961a0abc4d6b = async (cc) => {
                const _97ddb859_680f_44fb_85ee_430bf175db62 = r.okR(a, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _3944ad4c_72f2_45cf_810b_29b136ab56de = r.okR(b, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _c76e23fd_a7d1_437c_93f3_31cc4754ab3b = r.addi64(_97ddb859_680f_44fb_85ee_430bf175db62, _3944ad4c_72f2_45cf_810b_29b136ab56de)
                const _0595edca_dc6c_401c_bc6f_1f71f126858a = r.okR(cc, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _7c7550d2_8124_443d_b2a6_fd2cc0b8b828 = r.addi64(_c76e23fd_a7d1_437c_93f3_31cc4754ab3b, _0595edca_dc6c_401c_bc6f_1f71f126858a)
                let _5f9894ab_3798_4555_a070_07285614e7f9 = r.zeroed()
                let _1f47a3fe_fbe5_4785_9fc8_8f4afb321eb2 = r.copybool(_7c8033d2_d2ce_434b_85a5_514f856e988d)
                const _f9122fd4_d6f8_402c_b097_be6dbcee9bae = r.isErr(_7c7550d2_8124_443d_b2a6_fd2cc0b8b828)
                const _70fbb810_682a_4c13_8ba6_9449148e2556 = async () => {
                    _5f9894ab_3798_4555_a070_07285614e7f9 = r.reff(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                    _1f47a3fe_fbe5_4785_9fc8_8f4afb321eb2 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                  }
                const _2af73822_1121_4001_8196_0a9ac61523f5 = await r.condfn(_f9122fd4_d6f8_402c_b097_be6dbcee9bae, _70fbb810_682a_4c13_8ba6_9449148e2556)
                const _c351c132_2c96_42f4_abe6_77eaffc90439 = async () => {
                    const _9fb3af55_2741_4677_bfd8_dd5eda1f7bc8 = r.getR(_7c7550d2_8124_443d_b2a6_fd2cc0b8b828)
                    const _3381a9c3_5d3f_4f9c_933b_2d155a706009 = r.eqi64(_9fb3af55_2741_4677_bfd8_dd5eda1f7bc8, _c6b87a56_c688_4068_acb4_2181b7030f8d)
                    _5f9894ab_3798_4555_a070_07285614e7f9 = r.reff(_3381a9c3_5d3f_4f9c_933b_2d155a706009)
                    _1f47a3fe_fbe5_4785_9fc8_8f4afb321eb2 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                  }
                const _4d685387_1716_404a_856d_3e6c8e29d71c = await r.condfn(_1f47a3fe_fbe5_4785_9fc8_8f4afb321eb2, _c351c132_2c96_42f4_abe6_77eaffc90439)
                return _5f9894ab_3798_4555_a070_07285614e7f9
              }
            const _7ddbc6b1_af71_4c43_bd04_85712bf95c43 = await r.findl(_a7707284_0470_4ba0_8b35_ebcfa05bf521, _b81a7e55_fddd_4597_ae7d_961a0abc4d6b)
            const _f677b864_ee09_48ec_9525_628c1aaf3ea3 = r.isOk(_7ddbc6b1_af71_4c43_bd04_85712bf95c43)
            const _3edd7e1b_94e5_49a5_af6a_3dd85bedea91 = async () => {
                const _19fcd40f_c2ba_4fa5_a7f1_945b327e68c1 = r.i64str(a)
                const _6e2deee5_4ec8_4229_810a_f757b3f36a52 = r.catstr(_19fcd40f_c2ba_4fa5_a7f1_945b327e68c1, _b4ad8cdc_6956_43fa_8432_be19d6e1ba70)
                r.emit('stdout', _6e2deee5_4ec8_4229_810a_f757b3f36a52)
                const _ba0d2904_b810_4aa4_82eb_b2c00ecd952e = r.i64str(b)
                const _56e54902_d1f3_409a_8bba_96f8fa366fb2 = r.catstr(_ba0d2904_b810_4aa4_82eb_b2c00ecd952e, _b4ad8cdc_6956_43fa_8432_be19d6e1ba70)
                r.emit('stdout', _56e54902_d1f3_409a_8bba_96f8fa366fb2)
                let _5caaee15_84c6_45cc_9b6c_4f9e6c8753c5 = r.zeroed()
                let _53270e7b_30cb_49a0_b3a8_77aa3b7cd799 = r.copybool(_7c8033d2_d2ce_434b_85a5_514f856e988d)
                const _582f2ce7_8d11_458c_8860_524eba0c23b5 = r.isOk(_7ddbc6b1_af71_4c43_bd04_85712bf95c43)
                const _3ffa60ce_4709_45da_b2eb_e26350b144e0 = async () => {
                    const _1c2fd0b9_02ce_4446_8567_37ae17cfc3e0 = r.getR(_7ddbc6b1_af71_4c43_bd04_85712bf95c43)
                    const _2475b2a5_11fe_410d_80bf_a2ad91e6b6e6 = r.i64str(_1c2fd0b9_02ce_4446_8567_37ae17cfc3e0)
                    _5caaee15_84c6_45cc_9b6c_4f9e6c8753c5 = r.refv(_2475b2a5_11fe_410d_80bf_a2ad91e6b6e6)
                    _53270e7b_30cb_49a0_b3a8_77aa3b7cd799 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                  }
                const _e5467e36_aa61_4146_a9d6_003b9651c6ab = await r.condfn(_582f2ce7_8d11_458c_8860_524eba0c23b5, _3ffa60ce_4709_45da_b2eb_e26350b144e0)
                const _8d597592_0e03_4889_9ff7_79a00f90f27d = async () => {
                    const _45e980b5_7d41_46b2_b6c8_d7770adfdf12 = r.notbool(_582f2ce7_8d11_458c_8860_524eba0c23b5)
                    const _b087f5f8_e04b_485c_950d_9b577d6a2bf2 = async () => {
                        const _f71683ba_6f69_45d4_8dc9_355dffe081b5 = r.noerr()
                        const _74d3d7d0_d9be_4a01_a478_c30abe625ddf = r.getErr(_7ddbc6b1_af71_4c43_bd04_85712bf95c43, _f71683ba_6f69_45d4_8dc9_355dffe081b5)
                        const _afbb9ecd_4bdb_4f1e_9ef0_113e43ab18ea = r.errorstr(_74d3d7d0_d9be_4a01_a478_c30abe625ddf)
                        _5caaee15_84c6_45cc_9b6c_4f9e6c8753c5 = r.refv(_afbb9ecd_4bdb_4f1e_9ef0_113e43ab18ea)
                        _53270e7b_30cb_49a0_b3a8_77aa3b7cd799 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                      }
                    const _e67ddc28_7fd0_4396_9b3a_951b58f24647 = await r.condfn(_45e980b5_7d41_46b2_b6c8_d7770adfdf12, _b087f5f8_e04b_485c_950d_9b577d6a2bf2)
                  }
                const _da8550f2_22ec_4e10_b5e2_700d6ec41c15 = await r.condfn(_53270e7b_30cb_49a0_b3a8_77aa3b7cd799, _8d597592_0e03_4889_9ff7_79a00f90f27d)
                const _0cfc7c62_92bb_4619_87e6_4221b753000a = r.catstr(_5caaee15_84c6_45cc_9b6c_4f9e6c8753c5, _b4ad8cdc_6956_43fa_8432_be19d6e1ba70)
                r.emit('stdout', _0cfc7c62_92bb_4619_87e6_4221b753000a)
                const _b108050c_ff5a_43c2_a0ce_97dd4b347c5a = r.getOrR(_7ddbc6b1_af71_4c43_bd04_85712bf95c43, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
                const _846ae7bd_23f9_4070_a33e_5efa5e56305d = r.okR(a, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _9b7023e4_455d_48e5_b07a_149904c54fa1 = r.okR(b, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _1ea73336_d821_4aa0_b31b_77af7a0203f7 = r.muli64(_846ae7bd_23f9_4070_a33e_5efa5e56305d, _9b7023e4_455d_48e5_b07a_149904c54fa1)
                const _5cebad51_b4d0_43d2_b8f4_5fe48ce6f657 = r.okR(_b108050c_ff5a_43c2_a0ce_97dd4b347c5a, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _81480727_7dbb_41de_ace1_502fc13aed38 = r.muli64(_1ea73336_d821_4aa0_b31b_77af7a0203f7, _5cebad51_b4d0_43d2_b8f4_5fe48ce6f657)
                let _76d9099f_0474_4330_ad1f_53d42471dac5 = r.zeroed()
                let _1c2ea796_5acd_45d0_b75e_0be79bc4bc0d = r.copybool(_7c8033d2_d2ce_434b_85a5_514f856e988d)
                const _e943e54e_46f1_4538_ab1d_f83510cc5a17 = r.isOk(_81480727_7dbb_41de_ace1_502fc13aed38)
                const _05d10867_9746_4587_896d_d9e7441fa7bb = async () => {
                    const _6f883185_a25d_4082_bd39_244c6999464c = r.getR(_81480727_7dbb_41de_ace1_502fc13aed38)
                    const _53c41c8f_79f0_4e69_9dbc_347fc8f10a06 = r.i64str(_6f883185_a25d_4082_bd39_244c6999464c)
                    _76d9099f_0474_4330_ad1f_53d42471dac5 = r.refv(_53c41c8f_79f0_4e69_9dbc_347fc8f10a06)
                    _1c2ea796_5acd_45d0_b75e_0be79bc4bc0d = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                  }
                const _8a20fa67_ab7c_4f61_8c49_ab0e528f9362 = await r.condfn(_e943e54e_46f1_4538_ab1d_f83510cc5a17, _05d10867_9746_4587_896d_d9e7441fa7bb)
                const _c38e1f03_1f81_4a3a_87ae_3433acf70bfa = async () => {
                    const _e3d52986_e618_483e_84d2_f33a6e0a2083 = r.notbool(_e943e54e_46f1_4538_ab1d_f83510cc5a17)
                    const _fce9dfbd_a640_4f3f_906d_3447fa3799d0 = async () => {
                        const _7f98211a_9241_4b00_bb48_d49361588053 = r.noerr()
                        const _952ddb90_36ce_4ac0_95f8_419740e50216 = r.getErr(_81480727_7dbb_41de_ace1_502fc13aed38, _7f98211a_9241_4b00_bb48_d49361588053)
                        const _1b96d6f2_2456_4030_8a56_c05f2ba313fc = r.errorstr(_952ddb90_36ce_4ac0_95f8_419740e50216)
                        _76d9099f_0474_4330_ad1f_53d42471dac5 = r.refv(_1b96d6f2_2456_4030_8a56_c05f2ba313fc)
                        _1c2ea796_5acd_45d0_b75e_0be79bc4bc0d = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                      }
                    const _da172b54_4b15_4df4_8bfb_5675b1636a37 = await r.condfn(_e3d52986_e618_483e_84d2_f33a6e0a2083, _fce9dfbd_a640_4f3f_906d_3447fa3799d0)
                  }
                const _48f0efc7_1357_45bb_9769_7578cb65b0c6 = await r.condfn(_1c2ea796_5acd_45d0_b75e_0be79bc4bc0d, _c38e1f03_1f81_4a3a_87ae_3433acf70bfa)
                const _361b3423_9f41_4cff_aac4_6381a156c8e4 = r.catstr(_76d9099f_0474_4330_ad1f_53d42471dac5, _b4ad8cdc_6956_43fa_8432_be19d6e1ba70)
                r.emit('stdout', _361b3423_9f41_4cff_aac4_6381a156c8e4)
                r.emit('exit', _6e30a89b_48fb_4e4e_be4a_49da90240c76)
              }
            const _f7fca9a2_6c44_4b2d_b043_ac830055c3e4 = await r.condfn(_f677b864_ee09_48ec_9525_628c1aaf3ea3, _3edd7e1b_94e5_49a5_af6a_3dd85bedea91)
          }
        const _6f8e8885_ff5f_4645_b935_b90f6e59b494 = await r.eachl(_a7707284_0470_4ba0_8b35_ebcfa05bf521, _1299d122_b8c9_41a9_9290_ab6ca5484277)
      }
    const _212df940_fbf3_4544_9735_3fc7040f0767 = await r.eachl(_a7707284_0470_4ba0_8b35_ebcfa05bf521, _19debd25_6d50_479a_b5e6_d263071af1f7)
    r.emit('exit', _6e30a89b_48fb_4e4e_be4a_49da90240c76)
  })
r.on('__conn', async (conn) => {
    const _736040ad_c168_42d4_8628_f427ea3bca64 = r.register(conn, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _07335fcc_ce93_4a8c_880d_75b659004886 = r.register(conn, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
    const _326cc0f7_fc8c_48a0_8a2b_85a8ea112655 = r.newarr(_6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _8ff747b2_92fa_499f_905c_28c2d04f7e24 = r.newarr(_c8a86d92_61e0_42fc_af22_b79126fb87fd)
    r.pusharr(_8ff747b2_92fa_499f_905c_28c2d04f7e24, _326cc0f7_fc8c_48a0_8a2b_85a8ea112655, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _2b13af7b_f8ce_4633_ac9e_d016dc21c8f1 = r.reparr(_8ff747b2_92fa_499f_905c_28c2d04f7e24, _5f2c1ef5_c75b_468e_a7b1_f9c28bf2266c)
    const _2382c409_513a_4739_bb12_beca5a77fd39 = r.newarr(_8020acc8_fefc_4e4b_84a9_c3c5619d7a55)
    r.pusharr(_2382c409_513a_4739_bb12_beca5a77fd39, _07335fcc_ce93_4a8c_880d_75b659004886, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_2382c409_513a_4739_bb12_beca5a77fd39, _2b13af7b_f8ce_4633_ac9e_d016dc21c8f1, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    let _d24e2f24_7441_44b4_9258_f68c905aef1d = r.refv(_2382c409_513a_4739_bb12_beca5a77fd39)
    const _46ddc8ee_f981_4566_9167_51fc16a2e0da = async (kv, i) => {
        const _9bd66bf0_c79c_43c4_912c_556e408c4096 = r.register(kv, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
        const _61d32389_0fdd_4648_a2c1_67b619c70a22 = r.hashv(_9bd66bf0_c79c_43c4_912c_556e408c4096)
        const _cc608e2d_78c2_46fe_8260_78661c850ac3 = r.absi64(_61d32389_0fdd_4648_a2c1_67b619c70a22)
        const _c71eb811_a212_41d6_ba2d_f89f48de0c33 = r.register(_d24e2f24_7441_44b4_9258_f68c905aef1d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
        const _cd97179e_f3a3_4f6a_a3a2_9235fc4d15ad = r.lenarr(_c71eb811_a212_41d6_ba2d_f89f48de0c33)
        const _1f2f8b31_461b_45ae_8f4f_e8310922a6a6 = r.modi64(_cc608e2d_78c2_46fe_8260_78661c850ac3, _cd97179e_f3a3_4f6a_a3a2_9235fc4d15ad)
        const _c37bb580_709f_40d8_b4f9_711b6245c2ff = r.register(_d24e2f24_7441_44b4_9258_f68c905aef1d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
        const _2c4a155d_d271_4900_bc45_5480fcd230b7 = r.okR(_1f2f8b31_461b_45ae_8f4f_e8310922a6a6, _321eadd3_6654_47bd_9d6f_b63467f332a1)
        const _df5af33c_93bc_4900_89ec_37a970209d59 = r.resfrom(_c37bb580_709f_40d8_b4f9_711b6245c2ff, _2c4a155d_d271_4900_bc45_5480fcd230b7)
        const _2b74661d_11b7_43b8_9efb_02844b26f17e = r.getR(_df5af33c_93bc_4900_89ec_37a970209d59)
        r.pusharr(_2b74661d_11b7_43b8_9efb_02844b26f17e, i, _321eadd3_6654_47bd_9d6f_b63467f332a1)
      }
    const _75edb76a_9f92_4426_bfa8_99159c5d0cec = await r.eachl(_07335fcc_ce93_4a8c_880d_75b659004886, _46ddc8ee_f981_4566_9167_51fc16a2e0da)
    const _4c5003c1_73c4_45ee_a4bf_43f2cbc8c100 = r.register(conn, _8020acc8_fefc_4e4b_84a9_c3c5619d7a55)
    const _a4ef876f_2336_42b4_ad76_37d45583ce3d = r.newarr(_4da0a50d_5540_4b4a_83bc_870394aa1af3)
    r.pusharr(_a4ef876f_2336_42b4_ad76_37d45583ce3d, _736040ad_c168_42d4_8628_f427ea3bca64, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_a4ef876f_2336_42b4_ad76_37d45583ce3d, _d24e2f24_7441_44b4_9258_f68c905aef1d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_a4ef876f_2336_42b4_ad76_37d45583ce3d, _4c5003c1_73c4_45ee_a4bf_43f2cbc8c100, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _3027b597_ee74_46c7_9cf2_0eb04a0d3af1 = r.newarr(_6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _69792a19_73c9_4dd8_9af6_9eb54b8c7c20 = r.newarr(_6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _e3209c11_6c6e_4563_9f0f_5c8afd8324b5 = r.newarr(_c8a86d92_61e0_42fc_af22_b79126fb87fd)
    r.pusharr(_e3209c11_6c6e_4563_9f0f_5c8afd8324b5, _69792a19_73c9_4dd8_9af6_9eb54b8c7c20, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _8af39889_9c5f_4893_8d40_a2bafc8db193 = r.reparr(_e3209c11_6c6e_4563_9f0f_5c8afd8324b5, _5f2c1ef5_c75b_468e_a7b1_f9c28bf2266c)
    const _01f6a592_41d6_46ef_a5b6_6656da86f780 = r.newarr(_8020acc8_fefc_4e4b_84a9_c3c5619d7a55)
    r.pusharr(_01f6a592_41d6_46ef_a5b6_6656da86f780, _3027b597_ee74_46c7_9cf2_0eb04a0d3af1, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_01f6a592_41d6_46ef_a5b6_6656da86f780, _8af39889_9c5f_4893_8d40_a2bafc8db193, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    let _e29249f1_39f5_41ba_a067_ddcd39d8556d = r.refv(_01f6a592_41d6_46ef_a5b6_6656da86f780)
    const _a54ef901_d6d3_45bf_8a9d_b1eec708e4f4 = r.newarr(_8020acc8_fefc_4e4b_84a9_c3c5619d7a55)
    r.pusharr(_a54ef901_d6d3_45bf_8a9d_b1eec708e4f4, _b884828e_37e8_4f04_9f22_907b4c59464d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_a54ef901_d6d3_45bf_8a9d_b1eec708e4f4, _77545b46_fe93_49e0_a5ad_b39b58aeb193, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _99e00fed_df14_4c60_8e2c_71181eb4768f = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _452a8fb3_ae01_4bdc_9339_bab4607122e4 = r.lenarr(_99e00fed_df14_4c60_8e2c_71181eb4768f)
    const _bf3dce70_7e6d_4fa3_b7b9_ba915e5fcd50 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_bf3dce70_7e6d_4fa3_b7b9_ba915e5fcd50, _a54ef901_d6d3_45bf_8a9d_b1eec708e4f4, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    const _cf0f4c9c_7d9d_4fe6_881a_b6978139f55e = r.hashv(_b884828e_37e8_4f04_9f22_907b4c59464d)
    const _d171cf1a_1835_4375_8bbc_295a994ed8f5 = r.absi64(_cf0f4c9c_7d9d_4fe6_881a_b6978139f55e)
    const _29cdda2e_b3f4_497f_a6aa_94e3b56a191b = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
    const _1b521eed_092b_48a7_b9e3_d335ae4261f5 = r.lenarr(_29cdda2e_b3f4_497f_a6aa_94e3b56a191b)
    const _16bb1c61_0ad6_4c8f_a43f_33b39d3aab8e = r.modi64(_d171cf1a_1835_4375_8bbc_295a994ed8f5, _1b521eed_092b_48a7_b9e3_d335ae4261f5)
    const _935b7f05_0484_477f_96a6_404fe97fe1f1 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
    const _56aec7ca_0be0_49b6_ac6e_326d30a81f49 = r.okR(_16bb1c61_0ad6_4c8f_a43f_33b39d3aab8e, _321eadd3_6654_47bd_9d6f_b63467f332a1)
    const _021fbe37_5f21_4f2f_9177_e27141de6b93 = r.resfrom(_935b7f05_0484_477f_96a6_404fe97fe1f1, _56aec7ca_0be0_49b6_ac6e_326d30a81f49)
    const _5abe05c7_5625_4ed6_bb7a_937bce12f6ae = r.getR(_021fbe37_5f21_4f2f_9177_e27141de6b93)
    const _6107c941_0e6f_4573_a088_04e489f9f610 = r.lenarr(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae)
    const _fb446d77_bee1_44d7_bc58_8d2b9760a807 = r.eqi64(_6107c941_0e6f_4573_a088_04e489f9f610, _321eadd3_6654_47bd_9d6f_b63467f332a1)
    const _d76831ec_a14e_4666_a5ab_fe1cbb81f97f = async () => {
        const _412206d2_39d4_4e35_9e6a_d30dfd354c49 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
        const _923d7f1f_ab7a_4eaa_b9ae_2864537d519b = r.lenarr(_412206d2_39d4_4e35_9e6a_d30dfd354c49)
        const _ecbe5d6e_fb23_44b3_9af5_a3ebb3601a4e = r.okR(_923d7f1f_ab7a_4eaa_b9ae_2864537d519b, _321eadd3_6654_47bd_9d6f_b63467f332a1)
        const _91e00964_42b2_4c99_bb5b_bb73dcb6ef2e = r.okR(_8020acc8_fefc_4e4b_84a9_c3c5619d7a55, _321eadd3_6654_47bd_9d6f_b63467f332a1)
        const _0a6ae145_9023_4d5a_9669_cdffa3b43d55 = r.muli64(_ecbe5d6e_fb23_44b3_9af5_a3ebb3601a4e, _91e00964_42b2_4c99_bb5b_bb73dcb6ef2e)
        const _1fa7aa14_329d_4ba5_934e_7fd9a98bfcca = r.getOrR(_0a6ae145_9023_4d5a_9669_cdffa3b43d55, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
        const _e570d038_7710_46d4_b876_5bf20b6405ed = r.newarr(_6e30a89b_48fb_4e4e_be4a_49da90240c76)
        const _26e44f7a_7e8e_41d5_8199_327e86a31322 = r.newarr(_c8a86d92_61e0_42fc_af22_b79126fb87fd)
        r.pusharr(_26e44f7a_7e8e_41d5_8199_327e86a31322, _e570d038_7710_46d4_b876_5bf20b6405ed, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
        const _bed0bbc5_8ea6_4d5c_b2f3_a1153639dbb1 = r.reparr(_26e44f7a_7e8e_41d5_8199_327e86a31322, _1fa7aa14_329d_4ba5_934e_7fd9a98bfcca)
        const _0ca4865e_1bf5_4aef_bcc9_e9e60276d9a4 = r.refv(_bed0bbc5_8ea6_4d5c_b2f3_a1153639dbb1)
        const _949cddd9_4c7f_4d64_9b9c_9b60193cc7bd = r.refv(_0ca4865e_1bf5_4aef_bcc9_e9e60276d9a4)
        r.copytov(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _c8a86d92_61e0_42fc_af22_b79126fb87fd, _949cddd9_4c7f_4d64_9b9c_9b60193cc7bd)
        const _019725b9_f585_4bef_8de7_590a1d62d3c6 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
        const _2070210e_00a3_4edf_ae3c_ef8a0c62d428 = async (kv, i) => {
            const _40774ac2_1795_4a03_b75f_cfc05a777e82 = r.register(kv, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
            const _c082956e_ff64_4452_a9a8_5fcb1c77f827 = r.hashv(_40774ac2_1795_4a03_b75f_cfc05a777e82)
            const _1eec1865_f1fc_4028_b385_70d88fa866ef = r.absi64(_c082956e_ff64_4452_a9a8_5fcb1c77f827)
            const _79342823_d6bd_4a1b_b222_5408eb662cc2 = r.modi64(_1eec1865_f1fc_4028_b385_70d88fa866ef, _1fa7aa14_329d_4ba5_934e_7fd9a98bfcca)
            const _544c46fa_bcea_4e6d_a367_12d5f63ada27 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _c8a86d92_61e0_42fc_af22_b79126fb87fd)
            const _237c0bf1_b978_4137_9abb_8a1f8ef5a779 = r.okR(_79342823_d6bd_4a1b_b222_5408eb662cc2, _321eadd3_6654_47bd_9d6f_b63467f332a1)
            const _48c00dad_666d_47f6_bedc_f8c78d74b80b = r.resfrom(_544c46fa_bcea_4e6d_a367_12d5f63ada27, _237c0bf1_b978_4137_9abb_8a1f8ef5a779)
            const _6af1336a_dd7b_4e92_bb61_2a186c7c6f50 = r.getR(_48c00dad_666d_47f6_bedc_f8c78d74b80b)
            r.pusharr(_6af1336a_dd7b_4e92_bb61_2a186c7c6f50, i, _321eadd3_6654_47bd_9d6f_b63467f332a1)
          }
        const _4bf75268_f6d8_4e49_8f40_886c74f783c3 = await r.eachl(_019725b9_f585_4bef_8de7_590a1d62d3c6, _2070210e_00a3_4edf_ae3c_ef8a0c62d428)
      }
    const _c92c2ff7_1116_4921_a802_60bcb6544126 = await r.condfn(_fb446d77_bee1_44d7_bc58_8d2b9760a807, _d76831ec_a14e_4666_a5ab_fe1cbb81f97f)
    const _8a6aacd7_5cf3_40ba_bb4c_b989c31b6fb8 = r.notbool(_fb446d77_bee1_44d7_bc58_8d2b9760a807)
    const _5609e330_432e_4c04_9b4c_25d18778ca56 = async () => {
        const _6b136e22_96a5_4976_9397_3320633c5c26 = async (idx) => {
            const _5297996b_1ab8_41db_8759_6dfe0d7d0af8 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
            const _e29f5e4f_3f0f_4ea3_aef1_bf5294e3d260 = r.okR(idx, _321eadd3_6654_47bd_9d6f_b63467f332a1)
            const _4500a366_eefa_48a6_a5ed_634fd78336d3 = r.resfrom(_5297996b_1ab8_41db_8759_6dfe0d7d0af8, _e29f5e4f_3f0f_4ea3_aef1_bf5294e3d260)
            const _8100e486_5564_4eaf_94fb_6b7a61d4b68c = r.getR(_4500a366_eefa_48a6_a5ed_634fd78336d3)
            const _603b41f0_c907_4595_acb4_c01ad5165665 = r.register(_8100e486_5564_4eaf_94fb_6b7a61d4b68c, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
            const _e0e9bae0_1fd9_40a5_a24d_b6e5ff66c2b4 = r.eqstr(_603b41f0_c907_4595_acb4_c01ad5165665, _b884828e_37e8_4f04_9f22_907b4c59464d)
            return _e0e9bae0_1fd9_40a5_a24d_b6e5ff66c2b4
          }
        const _b8dbc977_aaa6_4fe1_be24_7bedeeac3b30 = await r.find(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae, _6b136e22_96a5_4976_9397_3320633c5c26)
        const _0a1a31bf_859d_4ba2_abe1_4bff65013794 = r.isOk(_b8dbc977_aaa6_4fe1_be24_7bedeeac3b30)
        const _6c57a9f0_7b15_47c0_b65f_d7705876376c = async () => {
            const _9b1ee870_847f_4cdb_9369_1c9ec64a2eb3 = async (idx, i) => {
                const _a30bf0b4_e135_4624_b675_866c245c1203 = r.register(_e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
                const _31f5f1e6_2926_4029_ac4a_2d4bcdb6bf40 = r.okR(idx, _321eadd3_6654_47bd_9d6f_b63467f332a1)
                const _59245bab_d30d_4188_9b2c_3e5f32c569d8 = r.resfrom(_a30bf0b4_e135_4624_b675_866c245c1203, _31f5f1e6_2926_4029_ac4a_2d4bcdb6bf40)
                const _542e80b1_f287_4c62_b689_bcc5a299e612 = r.getR(_59245bab_d30d_4188_9b2c_3e5f32c569d8)
                const _de3adf57_7405_45ee_a3c4_2cc825da96f4 = r.register(_542e80b1_f287_4c62_b689_bcc5a299e612, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
                const _17b86fbc_c17a_40ca_bc40_6679cdf5ec9e = r.eqstr(_de3adf57_7405_45ee_a3c4_2cc825da96f4, _b884828e_37e8_4f04_9f22_907b4c59464d)
                const _d69d357f_7b43_49d7_8fd4_d0481148c042 = async () => {
                    let _9d7a3e7d_b3cd_4e90_9438_dd25ff7d418f = r.zeroed()
                    let _2d4818da_24af_4da8_861d_a1e45ddd4840 = r.copybool(_7c8033d2_d2ce_434b_85a5_514f856e988d)
                    const _d2335a9e_16a5_41e9_a693_592c3f4aaa08 = r.lti64(i, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
                    const _43d64371_2c21_4ce3_9ccd_881b8392d75c = r.lenarr(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae)
                    const _1cd7a8e1_52c2_4c8a_9558_d21d0c615603 = r.gti64(i, _43d64371_2c21_4ce3_9ccd_881b8392d75c)
                    const _d4f7a06a_789b_443c_a7f5_040c4183698d = r.orbool(_d2335a9e_16a5_41e9_a693_592c3f4aaa08, _1cd7a8e1_52c2_4c8a_9558_d21d0c615603)
                    const _837bb388_3ca7_4295_b54a_225c86c6cca6 = async () => {
                        const _7d1b7deb_aaf7_4863_8908_de9db0155e05 = r.err(_65122beb_adcd_45eb_b688_5467e81d256d)
                        _9d7a3e7d_b3cd_4e90_9438_dd25ff7d418f = r.refv(_7d1b7deb_aaf7_4863_8908_de9db0155e05)
                        _2d4818da_24af_4da8_861d_a1e45ddd4840 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                      }
                    const _3d11eed9_0426_45d6_a4d0_7805ddc05c78 = await r.condfn(_d4f7a06a_789b_443c_a7f5_040c4183698d, _837bb388_3ca7_4295_b54a_225c86c6cca6)
                    const _d9a86438_2e59_4c72_89ac_6ee83e61a771 = async () => {
                        const _e7061563_b05b_4a50_9dcc_cec850935cb6 = r.notbool(_d4f7a06a_789b_443c_a7f5_040c4183698d)
                        const _f3eb2945_5a3d_4af1_bff4_18b84a94f285 = async () => {
                            r.copytof(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae, i, _452a8fb3_ae01_4bdc_9339_bab4607122e4)
                            const _7e24ae1d_309b_4b7b_af0d_4cc87b92e485 = r.someM(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
                            _9d7a3e7d_b3cd_4e90_9438_dd25ff7d418f = r.refv(_7e24ae1d_309b_4b7b_af0d_4cc87b92e485)
                            _2d4818da_24af_4da8_861d_a1e45ddd4840 = r.copybool(_e8a891d1_b130_4a2e_bfd8_f075cb377bc5)
                          }
                        const _409010fe_4dd7_4b48_a069_f631d7206a89 = await r.condfn(_e7061563_b05b_4a50_9dcc_cec850935cb6, _f3eb2945_5a3d_4af1_bff4_18b84a94f285)
                      }
                    const _ca921c2e_077d_41f5_94da_74ab10a9e5c9 = await r.condfn(_2d4818da_24af_4da8_861d_a1e45ddd4840, _d9a86438_2e59_4c72_89ac_6ee83e61a771)
                  }
                const _034b36c1_241b_42fa_a4bd_4689731fa27f = await r.condfn(_17b86fbc_c17a_40ca_bc40_6679cdf5ec9e, _d69d357f_7b43_49d7_8fd4_d0481148c042)
              }
            const _5c038cc9_af99_4c15_a721_829c781916dd = await r.eachl(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae, _9b1ee870_847f_4cdb_9369_1c9ec64a2eb3)
          }
        const _5f50c6e5_e2ee_469d_bdad_9d6a7b339187 = await r.condfn(_0a1a31bf_859d_4ba2_abe1_4bff65013794, _6c57a9f0_7b15_47c0_b65f_d7705876376c)
        const _40a18055_72e5_49f9_ab63_c15434622b7a = r.notbool(_0a1a31bf_859d_4ba2_abe1_4bff65013794)
        const _89f3db19_9f5b_4c2b_abbb_a4c0873bdade = async () => {
            r.pusharr(_5abe05c7_5625_4ed6_bb7a_937bce12f6ae, _452a8fb3_ae01_4bdc_9339_bab4607122e4, _321eadd3_6654_47bd_9d6f_b63467f332a1)
          }
        const _96d27a78_c990_40e2_8d46_473c8f33e9b2 = await r.condfn(_40a18055_72e5_49f9_ab63_c15434622b7a, _89f3db19_9f5b_4c2b_abbb_a4c0873bdade)
      }
    const _ed2b87e2_b614_47b3_adaf_dfcd89305a07 = await r.condfn(_8a6aacd7_5cf3_40ba_bb4c_b989c31b6fb8, _5609e330_432e_4c04_9b4c_25d18778ca56)
    const _ffbfe575_3216_4608_a046_32634f9e2618 = r.register(conn, _4da0a50d_5540_4b4a_83bc_870394aa1af3)
    const _dc280f8c_263a_4c0a_a153_df3a76c441b7 = r.newarr(_bb50ad84_e1c4_4b58_93cc_4389b6d14900)
    r.pusharr(_dc280f8c_263a_4c0a_a153_df3a76c441b7, _786b0b86_bbfb_4a93_8333_148a8d3e3f9e, _321eadd3_6654_47bd_9d6f_b63467f332a1)
    r.pusharr(_dc280f8c_263a_4c0a_a153_df3a76c441b7, _e29249f1_39f5_41ba_a067_ddcd39d8556d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_dc280f8c_263a_4c0a_a153_df3a76c441b7, _a4c4d844_5bf3_4b2d_bbae_a031a7587c6f, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_dc280f8c_263a_4c0a_a153_df3a76c441b7, _ffbfe575_3216_4608_a046_32634f9e2618, _321eadd3_6654_47bd_9d6f_b63467f332a1)
    const _90dfb043_e162_489b_8226_8fc0e6783214 = r.newarr(_8020acc8_fefc_4e4b_84a9_c3c5619d7a55)
    r.pusharr(_90dfb043_e162_489b_8226_8fc0e6783214, _a4ef876f_2336_42b4_ad76_37d45583ce3d, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.pusharr(_90dfb043_e162_489b_8226_8fc0e6783214, _dc280f8c_263a_4c0a_a153_df3a76c441b7, _6e30a89b_48fb_4e4e_be4a_49da90240c76)
    r.emit('connection', _90dfb043_e162_489b_8226_8fc0e6783214)
  })
r.on('stdout', async (out) => {
    const _d8bf0fcd_964b_4337_b5fe_101fa38c4e1c = r.stdoutp(out)
  })
r.on('exit', async (status) => {
    const _f4b4f581_eb05_4d57_b99d_2fb76ccaf446 = r.exitop(status)
  })
r.on('stderr', async (err) => {
    const _b67aea3a_35e5_4a34_aee2_35cb4f9c1552 = r.stderrp(err)
  })
r.emit('_start', undefined)
