const r = require('alan-js-runtime')
const _cd3bfaac_50ec_4c2a_9662_8cd4fd8c22a4 = "hello, world!"
const _7207e7d1_aa7b_4280_884c_001ac4113758 = "length:"
const _b0722764_e10c_4694_a3b1_581f14bd582b = "\n"
const _bf484e6c_493a_48b8_8dc0_d9b453895ad3 = "not:"
const _ee39df42_88f6_4f72_9157_e272d5e8f31f = "  trimmed  "
const _c1215440_5ef3_412f_88ab_01e0e92269fa = "r"
const _8b349d7c_a080_4532_81c7_b2404c1ae675 = "lt:"
const _9c682f0c_795a_4729_ad1a_62a59ff35291 = "gt:"
const _f1df32b2_1153_40a5_9434_d04b7d5945c3 = "lte:"
const _563acc9a_240b_420e_bc06_c149b6aadf2d = "gte:"
const _2947e2d2_e20e_4152_8965_1c5ca9851c99 = "eq:"
const _41e07975_68bd_4cd0_9793_1aa510c24684 = "neq:"
const _031f0566_b7b3_4645_9fbd_0a7b1e273656 = "hi there"
const _e4262f83_89f0_4c3b_bd89_3960674712f8 = "i t"
const _9a1dab46_0ca6_4b2f_9a86_e6f7f1f5ea0a = "matches:"
const _10eaee0e_5dd2_4f49_8423_5de492edd735 = "and:"
const _84179fb6_c319_4a62_a351_997ea6b7dd56 = "booland:"
const _aa495d6d_f53d_4026_a399_e134851a9a95 = "nand:"
const _3d6baa22_5c6c_4434_82b5_cf4a8e0c83c2 = "xnor:"
const _cf5fcedc_42e0_4742_ae05_4be1f4bbea5e = "xor:"
const _836107dd_5914_4609_96c7_3ade3ea5ae25 = "or:"
const _b42afa46_b8f6_46f0_8901_85ff466bc94c = "boolor:"
const _85a79e83_7dff_4308_b342_8f0ebb7eb23a = "nor:"
const _242425c5_4430_4911_b579_5b8249ac9d5c = true
const _c3d3d79c_e6e0_4d10_b840_a7a7255c75cd = 1n
const _33ec4880_7514_4fb8_b16f_8cfcfa17ae7f = 2n
const _77e75f15_884c_47e0_b317_37fc016b855d = 3n
const _cb3c2d2d_29a8_49b4_bcb4_28b19aff8290 = 100n
const _35d45f77_d05c_4433_b913_acd109c80237 = 0
r.on('stderr', async (err) => {
    let _9681c566_62d9_44fd_99e5_095d799473b1 = r.stderrp(err)
  })
r.on('exit', async (status) => {
    let _90db09ea_c046_40c4_bb28_0f17bc983673 = r.exitop(status)
  })
r.on('stdout', async (out) => {
    let _d32e8bc2_caae_45b3_9d02_e11e7606ab84 = r.stdoutp(out)
  })
r.on('_start', async () => {
    let _04b6c54c_f764_4803_9a34_bc8f09675c72 = r.copystr(_cd3bfaac_50ec_4c2a_9662_8cd4fd8c22a4)
    let _0e1fdf65_9aff_48ff_abcc_93731c361d2f = r.lenstr(_04b6c54c_f764_4803_9a34_bc8f09675c72)
    let _a037dee0_1f23_409a_83a7_447294b42d60 = r.reff(_0e1fdf65_9aff_48ff_abcc_93731c361d2f)
    let _b3462e61_c8fc_4704_ba42_2c99670132e7 = r.copystr(_7207e7d1_aa7b_4280_884c_001ac4113758)
    let _e05a7ff8_38c1_4f82_9bcf_c11330fb66d2 = r.i64str(_a037dee0_1f23_409a_83a7_447294b42d60)
    let _2b05f097_7ab3_47ca_a562_fcf9758920c1 = r.refv(_e05a7ff8_38c1_4f82_9bcf_c11330fb66d2)
    let _bef405a4_a91f_45a0_8591_cc3d99131353 = r.catstr(_b3462e61_c8fc_4704_ba42_2c99670132e7, _2b05f097_7ab3_47ca_a562_fcf9758920c1)
    let _f0dfebe2_c001_45e7_a8f5_91da6d62fc62 = r.refv(_bef405a4_a91f_45a0_8591_cc3d99131353)
    let _80cfda3f_5471_43c4_97d6_f1bc08c9b18d = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _3437516d_c3f8_440c_8a26_d621afe8ba2a = r.catstr(_f0dfebe2_c001_45e7_a8f5_91da6d62fc62, _80cfda3f_5471_43c4_97d6_f1bc08c9b18d)
    let _a7052846_8dd6_4bff_a46f_50856d9c8310 = r.refv(_3437516d_c3f8_440c_8a26_d621afe8ba2a)
    r.emit('stdout', _a7052846_8dd6_4bff_a46f_50856d9c8310)
    let _7cd4534d_824c_429f_b1c8_7e95a4e43f93 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _025ab420_0554_41a4_bcb3_2a0522cd8d5c = r.notbool(_7cd4534d_824c_429f_b1c8_7e95a4e43f93)
    let _041a0052_371b_459f_9436_da06e87a587e = r.reff(_025ab420_0554_41a4_bcb3_2a0522cd8d5c)
    let _697656ae_346f_45be_a8be_d32e01bb62de = r.copystr(_bf484e6c_493a_48b8_8dc0_d9b453895ad3)
    let _000585a6_b700_4ff6_b6e8_f2c4148d2c09 = r.boolstr(_041a0052_371b_459f_9436_da06e87a587e)
    let _bdeb3034_8fe4_4652_b9a8_047b53f786d1 = r.refv(_000585a6_b700_4ff6_b6e8_f2c4148d2c09)
    let _7dd9e7e8_556b_4607_aa2b_044dc0d7fa75 = r.catstr(_697656ae_346f_45be_a8be_d32e01bb62de, _bdeb3034_8fe4_4652_b9a8_047b53f786d1)
    let _5d1d11b0_ab12_4d6e_912f_dad201f43953 = r.refv(_7dd9e7e8_556b_4607_aa2b_044dc0d7fa75)
    let _eebaa0d5_81c0_4aa2_b53f_31832955187e = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _051aff80_ac32_44c0_9c94_a28c9d138609 = r.catstr(_5d1d11b0_ab12_4d6e_912f_dad201f43953, _eebaa0d5_81c0_4aa2_b53f_31832955187e)
    let _fc31ba46_4563_4122_8025_584d945779d4 = r.refv(_051aff80_ac32_44c0_9c94_a28c9d138609)
    r.emit('stdout', _fc31ba46_4563_4122_8025_584d945779d4)
    let _b6b6ef55_a0f9_4451_a3a6_57e5094540a7 = r.copystr(_ee39df42_88f6_4f72_9157_e272d5e8f31f)
    let _f3f33cef_892d_40db_9e05_d59b0fb101dc = r.trim(_b6b6ef55_a0f9_4451_a3a6_57e5094540a7)
    let _145a7bda_39d6_486d_bb57_e44f90925d0a = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _b0ba9ad6_36eb_4f8f_96ae_26b40dba41c5 = r.catstr(_f3f33cef_892d_40db_9e05_d59b0fb101dc, _145a7bda_39d6_486d_bb57_e44f90925d0a)
    let _05f014d7_8264_4241_9c36_df86cfe4b2a0 = r.refv(_b0ba9ad6_36eb_4f8f_96ae_26b40dba41c5)
    r.emit('stdout', _05f014d7_8264_4241_9c36_df86cfe4b2a0)
    let _69bc8a15_a89e_45f9_a643_6b17f7d6a936 = r.copyi64(_cb3c2d2d_29a8_49b4_bcb4_28b19aff8290)
    let _45c2e797_2832_4e7c_94ba_794ea6367428 = await r.waitop(_69bc8a15_a89e_45f9_a643_6b17f7d6a936)
    let _fb21f385_86be_459e_bcf3_d7e84b992887 = r.copystr(_c1215440_5ef3_412f_88ab_01e0e92269fa)
    let _d23c1ed0_1040_4a00_847f_97fe07066653 = r.copyi64(_77e75f15_884c_47e0_b317_37fc016b855d)
    let _bc4bffc0_80e2_4d44_97e6_10a916073a51 = r.repstr(_fb21f385_86be_459e_bcf3_d7e84b992887, _d23c1ed0_1040_4a00_847f_97fe07066653)
    let _b1f7b38c_b769_4c47_9121_746b5eb7b465 = r.refv(_bc4bffc0_80e2_4d44_97e6_10a916073a51)
    r.emit('stdout', _b1f7b38c_b769_4c47_9121_746b5eb7b465)
    let _1c7cdaab_2da3_4033_bd35_5727f5761e65 = r.copyi64(_cb3c2d2d_29a8_49b4_bcb4_28b19aff8290)
    let _682111c5_1535_4d83_a524_6a7e7522cb62 = await r.waitop(_1c7cdaab_2da3_4033_bd35_5727f5761e65)
    let _1c054452_95a2_4f72_8e90_8d54ded0389f = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    r.emit('stdout', _1c054452_95a2_4f72_8e90_8d54ded0389f)
    let _acabf211_838a_49d8_bfc3_93736dff86a2 = r.copyi64(_cb3c2d2d_29a8_49b4_bcb4_28b19aff8290)
    let _427aa72d_44b2_490c_ae35_55e759920d65 = await r.waitop(_acabf211_838a_49d8_bfc3_93736dff86a2)
    let _cd5dd256_bf10_4175_bd56_6d0e76e4a88b = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _b3efe99d_5aa9_41c3_b6e5_a0896a2a0a56 = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _00649ca5_6a41_4fd3_9d76_c6f06705a889 = r.lti64(_cd5dd256_bf10_4175_bd56_6d0e76e4a88b, _b3efe99d_5aa9_41c3_b6e5_a0896a2a0a56)
    let _dfe7e4d1_c130_4a68_ac56_f02664912c79 = r.reff(_00649ca5_6a41_4fd3_9d76_c6f06705a889)
    let _16740495_9452_4059_b469_c4383328d157 = r.copystr(_8b349d7c_a080_4532_81c7_b2404c1ae675)
    let _734fd8b5_61a8_4c97_8a93_9e90e2840143 = r.boolstr(_dfe7e4d1_c130_4a68_ac56_f02664912c79)
    let _b46fbd79_5bb8_4e86_869a_dfabe5a543d2 = r.refv(_734fd8b5_61a8_4c97_8a93_9e90e2840143)
    let _a00f1e29_87bb_4df2_84e7_777af59340e6 = r.catstr(_16740495_9452_4059_b469_c4383328d157, _b46fbd79_5bb8_4e86_869a_dfabe5a543d2)
    let _08816796_dd92_404c_97c9_2cef3b61cab3 = r.refv(_a00f1e29_87bb_4df2_84e7_777af59340e6)
    let _71c801cc_2659_44f0_bd16_918f1c38e8e4 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _751c1e3e_0b4c_465c_a9aa_c6fe8831f0ee = r.catstr(_08816796_dd92_404c_97c9_2cef3b61cab3, _71c801cc_2659_44f0_bd16_918f1c38e8e4)
    let _63f62617_70e7_4569_b2a8_2fc72ec16dad = r.refv(_751c1e3e_0b4c_465c_a9aa_c6fe8831f0ee)
    r.emit('stdout', _63f62617_70e7_4569_b2a8_2fc72ec16dad)
    let _e64756e4_1ca0_4638_a987_72056a9a6d96 = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _12968dcb_3bd6_48df_b5a0_0b18cfe5bb82 = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _6368f55e_e276_4ce7_a1e3_1dfdb2b52597 = r.gti64(_e64756e4_1ca0_4638_a987_72056a9a6d96, _12968dcb_3bd6_48df_b5a0_0b18cfe5bb82)
    let _09b2f805_f593_4563_94f4_46fba9ab0c87 = r.reff(_6368f55e_e276_4ce7_a1e3_1dfdb2b52597)
    let _bd5e9911_3c0a_4857_816b_669453af5325 = r.copystr(_9c682f0c_795a_4729_ad1a_62a59ff35291)
    let _085ed6e7_9543_464c_97e6_ebb68df5cafb = r.boolstr(_09b2f805_f593_4563_94f4_46fba9ab0c87)
    let _8aab6b0d_be77_48fc_a67e_d31330374afb = r.refv(_085ed6e7_9543_464c_97e6_ebb68df5cafb)
    let _ca94dd8d_08cd_426d_96b1_e59da69baece = r.catstr(_bd5e9911_3c0a_4857_816b_669453af5325, _8aab6b0d_be77_48fc_a67e_d31330374afb)
    let _30565562_2a79_4338_aaad_d8fb38a69a7f = r.refv(_ca94dd8d_08cd_426d_96b1_e59da69baece)
    let _45ed8830_d422_41ec_84a1_625273fdb661 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _84da05ad_6c2c_46a3_942d_b2f969dd227d = r.catstr(_30565562_2a79_4338_aaad_d8fb38a69a7f, _45ed8830_d422_41ec_84a1_625273fdb661)
    let _ba35625f_c480_462d_af18_8bc802e57094 = r.refv(_84da05ad_6c2c_46a3_942d_b2f969dd227d)
    r.emit('stdout', _ba35625f_c480_462d_af18_8bc802e57094)
    let _9dbeb1e3_505b_42c2_98a9_a4990edeb891 = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _1087a9f9_5be7_4787_89ab_5d0631fefdcc = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _64513e54_61b3_4f4f_90aa_87a7a08a8f1d = r.ltei64(_9dbeb1e3_505b_42c2_98a9_a4990edeb891, _1087a9f9_5be7_4787_89ab_5d0631fefdcc)
    let _1bb42851_146d_4a8f_8ab2_908b46093d6b = r.reff(_64513e54_61b3_4f4f_90aa_87a7a08a8f1d)
    let _f1c736c9_4955_449f_b5f9_9382ef55e7ca = r.copystr(_f1df32b2_1153_40a5_9434_d04b7d5945c3)
    let _ec85d30f_aec3_47e3_972f_c908f36da116 = r.boolstr(_1bb42851_146d_4a8f_8ab2_908b46093d6b)
    let _cb546c5f_ed95_46a3_ae7e_6bef89510b39 = r.refv(_ec85d30f_aec3_47e3_972f_c908f36da116)
    let _9a4229d2_964a_45fe_aec5_441dc404dd7b = r.catstr(_f1c736c9_4955_449f_b5f9_9382ef55e7ca, _cb546c5f_ed95_46a3_ae7e_6bef89510b39)
    let _f91ce9d4_bb06_4fcd_8961_7984d281c06b = r.refv(_9a4229d2_964a_45fe_aec5_441dc404dd7b)
    let _1883c478_f5f2_450b_bb3a_4545507142cb = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _4f7a3e26_0641_41b6_b5f5_23a6be4739ef = r.catstr(_f91ce9d4_bb06_4fcd_8961_7984d281c06b, _1883c478_f5f2_450b_bb3a_4545507142cb)
    let _06075221_2ccf_4f69_a7a6_d996ad53b323 = r.refv(_4f7a3e26_0641_41b6_b5f5_23a6be4739ef)
    r.emit('stdout', _06075221_2ccf_4f69_a7a6_d996ad53b323)
    let _7b62a150_cedc_4ab3_8ed3_e36e48f45e19 = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _b79408c5_5ca2_42cc_8188_59706b9ceded = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _7272a9a5_1335_404c_8e29_343d68617e95 = r.gtei64(_7b62a150_cedc_4ab3_8ed3_e36e48f45e19, _b79408c5_5ca2_42cc_8188_59706b9ceded)
    let _89facad8_f4f3_445c_add2_05c178edd91f = r.reff(_7272a9a5_1335_404c_8e29_343d68617e95)
    let _4b5bd3aa_9672_478b_bf50_cd21b2e4a211 = r.copystr(_563acc9a_240b_420e_bc06_c149b6aadf2d)
    let _d1fd9b2b_55e5_4af4_86a6_f8a47834cda1 = r.boolstr(_89facad8_f4f3_445c_add2_05c178edd91f)
    let _d0868e39_fe7d_4cbc_9b8b_e588d6b5a39d = r.refv(_d1fd9b2b_55e5_4af4_86a6_f8a47834cda1)
    let _6ed3f58b_3b27_48b8_940b_16fbf04a6de8 = r.catstr(_4b5bd3aa_9672_478b_bf50_cd21b2e4a211, _d0868e39_fe7d_4cbc_9b8b_e588d6b5a39d)
    let _b7a98f5a_1440_40e3_8a38_e2d906e4b01d = r.refv(_6ed3f58b_3b27_48b8_940b_16fbf04a6de8)
    let _25bbf201_382e_4d40_8f4f_bd3303bbcd28 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _f9df8ba8_7546_4d2c_893e_dde88090895a = r.catstr(_b7a98f5a_1440_40e3_8a38_e2d906e4b01d, _25bbf201_382e_4d40_8f4f_bd3303bbcd28)
    let _68e0a2fd_7bd1_4523_b485_1868be021545 = r.refv(_f9df8ba8_7546_4d2c_893e_dde88090895a)
    r.emit('stdout', _68e0a2fd_7bd1_4523_b485_1868be021545)
    let _c0e49d76_339e_4235_b06a_5ebd4b4e83d1 = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _36ac6138_6c6c_4b04_acf1_77c4a14a9443 = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _fa245227_5ba8_42fa_892b_8f4d5980da8d = r.eqi64(_c0e49d76_339e_4235_b06a_5ebd4b4e83d1, _36ac6138_6c6c_4b04_acf1_77c4a14a9443)
    let _92cc333e_417c_4917_97c0_ace5402bb7dd = r.reff(_fa245227_5ba8_42fa_892b_8f4d5980da8d)
    let _cee07704_965b_44f4_bca1_f2bef063bc1a = r.copystr(_2947e2d2_e20e_4152_8965_1c5ca9851c99)
    let _d44e2759_f307_4862_b699_907213b7dfde = r.boolstr(_92cc333e_417c_4917_97c0_ace5402bb7dd)
    let _7e3a19e9_4489_4548_912e_4e2590890632 = r.refv(_d44e2759_f307_4862_b699_907213b7dfde)
    let _e5b8d5ab_fdc1_4d01_8774_3cf2917661d7 = r.catstr(_cee07704_965b_44f4_bca1_f2bef063bc1a, _7e3a19e9_4489_4548_912e_4e2590890632)
    let _082e1eb0_d843_450d_91ff_661a724c4434 = r.refv(_e5b8d5ab_fdc1_4d01_8774_3cf2917661d7)
    let _3ceee833_5c8d_45b9_af3c_4ad2f346c7c7 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _878cfbd8_b794_4ab0_a184_605d8917fa14 = r.catstr(_082e1eb0_d843_450d_91ff_661a724c4434, _3ceee833_5c8d_45b9_af3c_4ad2f346c7c7)
    let _0436661f_5563_4701_be84_88b8100f9707 = r.refv(_878cfbd8_b794_4ab0_a184_605d8917fa14)
    r.emit('stdout', _0436661f_5563_4701_be84_88b8100f9707)
    let _7c414333_c476_4d00_b466_32a02d16be13 = r.copyi64(_c3d3d79c_e6e0_4d10_b840_a7a7255c75cd)
    let _da9f7fe4_0e37_45e1_bed6_628df491dece = r.copyi64(_33ec4880_7514_4fb8_b16f_8cfcfa17ae7f)
    let _bc490e60_1f95_4fb6_885b_8817e5ca1f88 = r.neqi64(_7c414333_c476_4d00_b466_32a02d16be13, _da9f7fe4_0e37_45e1_bed6_628df491dece)
    let _c9dda20d_1234_4c33_892b_eb77301beb43 = r.reff(_bc490e60_1f95_4fb6_885b_8817e5ca1f88)
    let _09fcee89_7a9c_4fdb_9876_2841785602be = r.copystr(_41e07975_68bd_4cd0_9793_1aa510c24684)
    let _ad614521_5be7_4d33_80e0_478a8dfff653 = r.boolstr(_c9dda20d_1234_4c33_892b_eb77301beb43)
    let _0eb76205_cc07_4d4a_81f6_cbc544f5d353 = r.refv(_ad614521_5be7_4d33_80e0_478a8dfff653)
    let _9445724f_b2ca_48ef_9520_d66b52b87cf1 = r.catstr(_09fcee89_7a9c_4fdb_9876_2841785602be, _0eb76205_cc07_4d4a_81f6_cbc544f5d353)
    let _62b1a830_b102_412e_b9d6_ddadf51cb6f3 = r.refv(_9445724f_b2ca_48ef_9520_d66b52b87cf1)
    let _54c72669_94a3_4363_98ff_2a346c6ae00b = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _98193ca5_8041_4a75_8b5f_b3c8b1c588bf = r.catstr(_62b1a830_b102_412e_b9d6_ddadf51cb6f3, _54c72669_94a3_4363_98ff_2a346c6ae00b)
    let _5f02d48b_ff9f_4623_b88b_2dc73011d15b = r.refv(_98193ca5_8041_4a75_8b5f_b3c8b1c588bf)
    r.emit('stdout', _5f02d48b_ff9f_4623_b88b_2dc73011d15b)
    let _f6c4929d_def1_4f0d_be7b_0108cc4c4906 = r.copystr(_031f0566_b7b3_4645_9fbd_0a7b1e273656)
    let _6b3b4297_30a4_47d8_95f3_1ccca4a9b0aa = r.copystr(_e4262f83_89f0_4c3b_bd89_3960674712f8)
    let _bd24bf4a_e6ab_4f19_bea6_9f0b45437b12 = r.matches(_f6c4929d_def1_4f0d_be7b_0108cc4c4906, _6b3b4297_30a4_47d8_95f3_1ccca4a9b0aa)
    let _d974f6a0_b3f8_4e9a_9459_8710a0dd7a62 = r.copystr(_9a1dab46_0ca6_4b2f_9a86_e6f7f1f5ea0a)
    let _2261ba7e_8b16_4fd4_8f42_2a9842359c2e = r.boolstr(_bd24bf4a_e6ab_4f19_bea6_9f0b45437b12)
    let _2947fdb9_0257_4e2d_945c_4eac12f72bd5 = r.refv(_2261ba7e_8b16_4fd4_8f42_2a9842359c2e)
    let _2b423ac2_fa65_45d7_a96f_81745d11535a = r.catstr(_d974f6a0_b3f8_4e9a_9459_8710a0dd7a62, _2947fdb9_0257_4e2d_945c_4eac12f72bd5)
    let _b0446358_47fe_478f_8e02_e082494b362a = r.refv(_2b423ac2_fa65_45d7_a96f_81745d11535a)
    let _d29cd4f3_86ab_420d_946f_f421afe0bbd0 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _749aedcf_9c3c_4b26_a701_2e7fda97ddb6 = r.catstr(_b0446358_47fe_478f_8e02_e082494b362a, _d29cd4f3_86ab_420d_946f_f421afe0bbd0)
    let _f57aaa19_8910_4cb5_ae80_f368d28a1ced = r.refv(_749aedcf_9c3c_4b26_a701_2e7fda97ddb6)
    r.emit('stdout', _f57aaa19_8910_4cb5_ae80_f368d28a1ced)
    let _f2713f8f_46de_40e2_8670_ac6a4811115f = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _df9f98dc_a30a_4737_860d_202b30a67c83 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _1c645648_39c0_4e94_8ff8_453bd16b095f = r.andbool(_f2713f8f_46de_40e2_8670_ac6a4811115f, _df9f98dc_a30a_4737_860d_202b30a67c83)
    let _6da220ff_0644_423d_8fc0_f97234e85b39 = r.reff(_1c645648_39c0_4e94_8ff8_453bd16b095f)
    let _6b3d9e3d_c608_4346_b05e_3644f2c2a290 = r.copystr(_10eaee0e_5dd2_4f49_8423_5de492edd735)
    let _5e6ad3fa_8135_4b0f_b56b_0fed7b66af11 = r.boolstr(_6da220ff_0644_423d_8fc0_f97234e85b39)
    let _07d94d81_ad90_4de0_9b36_1092b27fbb62 = r.refv(_5e6ad3fa_8135_4b0f_b56b_0fed7b66af11)
    let _9639408e_f011_4fe9_bee9_0739d6fab9c5 = r.catstr(_6b3d9e3d_c608_4346_b05e_3644f2c2a290, _07d94d81_ad90_4de0_9b36_1092b27fbb62)
    let _fab5fae4_136c_4982_92fb_108db989af6d = r.refv(_9639408e_f011_4fe9_bee9_0739d6fab9c5)
    let _74d00dcb_8bcc_45c7_96d7_d1f2c7d7b5b4 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _059e1b21_ce0e_427a_b931_b8cfb8439559 = r.catstr(_fab5fae4_136c_4982_92fb_108db989af6d, _74d00dcb_8bcc_45c7_96d7_d1f2c7d7b5b4)
    let _00e4a35a_f532_4f94_ba2d_8fcb040e5482 = r.refv(_059e1b21_ce0e_427a_b931_b8cfb8439559)
    r.emit('stdout', _00e4a35a_f532_4f94_ba2d_8fcb040e5482)
    let _74d747b7_b5a9_4b70_8d00_86a6f49b24c5 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _85ac8204_b0f3_4154_b7dd_8dc239bcbec4 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _f4f6b5e1_0f79_403f_abf7_a63224d5c1a6 = r.andbool(_74d747b7_b5a9_4b70_8d00_86a6f49b24c5, _85ac8204_b0f3_4154_b7dd_8dc239bcbec4)
    let _4ed52ffd_44c3_43d3_b29b_d0508fab1aee = r.reff(_f4f6b5e1_0f79_403f_abf7_a63224d5c1a6)
    let _06d0c310_abc3_47e2_941d_61a7190123e2 = r.reff(_4ed52ffd_44c3_43d3_b29b_d0508fab1aee)
    let _319dc819_0a39_4396_a276_71898dbeeb5f = r.copystr(_84179fb6_c319_4a62_a351_997ea6b7dd56)
    let _b358388a_1dda_4a59_a3f8_645affd3ce35 = r.boolstr(_06d0c310_abc3_47e2_941d_61a7190123e2)
    let _908fd578_94b3_4dd1_a626_a2bdb76da85c = r.refv(_b358388a_1dda_4a59_a3f8_645affd3ce35)
    let _7e56a281_b9c6_46da_9a82_c11e63c7ee09 = r.catstr(_319dc819_0a39_4396_a276_71898dbeeb5f, _908fd578_94b3_4dd1_a626_a2bdb76da85c)
    let _d6680bb4_f795_4a93_9602_8461e2425e4a = r.refv(_7e56a281_b9c6_46da_9a82_c11e63c7ee09)
    let _526736f8_6464_426d_9b6b_c86d4f587577 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _8f1a6d83_ca78_4c46_814a_c227540b09ad = r.catstr(_d6680bb4_f795_4a93_9602_8461e2425e4a, _526736f8_6464_426d_9b6b_c86d4f587577)
    let _42665e24_d623_43be_842a_3936dc50b3c9 = r.refv(_8f1a6d83_ca78_4c46_814a_c227540b09ad)
    r.emit('stdout', _42665e24_d623_43be_842a_3936dc50b3c9)
    let _68434cea_f852_4100_a560_355b1aa9bba7 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _6a4a6604_53ea_40d9_a10a_7d4ae6f529f7 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _8eee53ad_40fb_4810_ac2d_e01436f57851 = r.nandboo(_68434cea_f852_4100_a560_355b1aa9bba7, _6a4a6604_53ea_40d9_a10a_7d4ae6f529f7)
    let _0d363c19_b5fa_4171_981d_6f80a70ff954 = r.reff(_8eee53ad_40fb_4810_ac2d_e01436f57851)
    let _c44f2353_64ed_4053_bce4_83ef469ce4a5 = r.copystr(_aa495d6d_f53d_4026_a399_e134851a9a95)
    let _0e319e35_e757_429d_b611_f8634eda31ff = r.boolstr(_0d363c19_b5fa_4171_981d_6f80a70ff954)
    let _a6639030_d77f_4e2a_87a3_5469c0830363 = r.refv(_0e319e35_e757_429d_b611_f8634eda31ff)
    let _64ae86e8_9fa6_42e1_b56f_160de7e3b3ae = r.catstr(_c44f2353_64ed_4053_bce4_83ef469ce4a5, _a6639030_d77f_4e2a_87a3_5469c0830363)
    let _8a5782ac_6768_482c_b19f_7586d2ca13ec = r.refv(_64ae86e8_9fa6_42e1_b56f_160de7e3b3ae)
    let _dc9a9852_cef5_4950_aed2_492191b11813 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _082255a3_f0fb_4b63_bbb7_a5e16e969ee5 = r.catstr(_8a5782ac_6768_482c_b19f_7586d2ca13ec, _dc9a9852_cef5_4950_aed2_492191b11813)
    let _8883a5a1_7d95_4a0e_94f6_a75705a1897d = r.refv(_082255a3_f0fb_4b63_bbb7_a5e16e969ee5)
    r.emit('stdout', _8883a5a1_7d95_4a0e_94f6_a75705a1897d)
    let _86046804_e75e_4546_90c0_c597d58ec15c = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _3c6f2e8e_17ae_4541_91e7_51b0bbc0ee5d = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _c1926df2_c76b_478c_b1ff_19ed9c1933dc = r.xnorboo(_86046804_e75e_4546_90c0_c597d58ec15c, _3c6f2e8e_17ae_4541_91e7_51b0bbc0ee5d)
    let _ee444283_a894_4910_ab2d_a48d3a93a2fa = r.reff(_c1926df2_c76b_478c_b1ff_19ed9c1933dc)
    let _49a9016a_bf92_4a88_9c51_d171a8864d9e = r.copystr(_3d6baa22_5c6c_4434_82b5_cf4a8e0c83c2)
    let _79e07607_5f52_4a69_8a0f_179930e320a2 = r.boolstr(_ee444283_a894_4910_ab2d_a48d3a93a2fa)
    let _861066ba_2056_412a_8eca_243b4582495a = r.refv(_79e07607_5f52_4a69_8a0f_179930e320a2)
    let _2c5a8721_e0f3_4afe_92e3_72918c21c9e7 = r.catstr(_49a9016a_bf92_4a88_9c51_d171a8864d9e, _861066ba_2056_412a_8eca_243b4582495a)
    let _d629ac57_6714_47dd_b7fa_51b852d0962a = r.refv(_2c5a8721_e0f3_4afe_92e3_72918c21c9e7)
    let _cec90108_636c_4e60_9ca3_7d4ef9e11d17 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _38535110_212f_49e4_b3f7_c341910597fc = r.catstr(_d629ac57_6714_47dd_b7fa_51b852d0962a, _cec90108_636c_4e60_9ca3_7d4ef9e11d17)
    let _7cf4c657_ecf2_4492_9bb8_f59dae80a67d = r.refv(_38535110_212f_49e4_b3f7_c341910597fc)
    r.emit('stdout', _7cf4c657_ecf2_4492_9bb8_f59dae80a67d)
    let _6f336c80_1383_41d2_8bd6_3bb9bfb07880 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _9e8fa305_6186_4401_9130_cacc641f69fb = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _4e8b0722_4efe_44f1_962e_1822d4fdfef6 = r.xorbool(_6f336c80_1383_41d2_8bd6_3bb9bfb07880, _9e8fa305_6186_4401_9130_cacc641f69fb)
    let _d247b02b_a57d_4c81_89ef_360127713bc4 = r.reff(_4e8b0722_4efe_44f1_962e_1822d4fdfef6)
    let _0e4ed59b_aedd_4a8f_894e_d11e38b52b0a = r.copystr(_cf5fcedc_42e0_4742_ae05_4be1f4bbea5e)
    let _782c1481_1f04_4f52_ac1f_15b9c1cbef94 = r.boolstr(_d247b02b_a57d_4c81_89ef_360127713bc4)
    let _985b777e_eb3d_44ff_a6e4_c2ab5952d286 = r.refv(_782c1481_1f04_4f52_ac1f_15b9c1cbef94)
    let _2d7e929d_9a3f_4f78_a311_c1609d0540ce = r.catstr(_0e4ed59b_aedd_4a8f_894e_d11e38b52b0a, _985b777e_eb3d_44ff_a6e4_c2ab5952d286)
    let _cae674b7_0d43_4c46_842c_156ad79fbd63 = r.refv(_2d7e929d_9a3f_4f78_a311_c1609d0540ce)
    let _8b6c05ee_b898_4042_a6bc_a5e5bde3d730 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _71e80fec_cb24_47cb_b9d9_03dcd7b6eea3 = r.catstr(_cae674b7_0d43_4c46_842c_156ad79fbd63, _8b6c05ee_b898_4042_a6bc_a5e5bde3d730)
    let _ba782379_8d96_450a_8047_f201d68cd2d7 = r.refv(_71e80fec_cb24_47cb_b9d9_03dcd7b6eea3)
    r.emit('stdout', _ba782379_8d96_450a_8047_f201d68cd2d7)
    let _55aca479_89b9_4d1e_b5b1_f34235ee92e4 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _06cb982f_66f2_41e6_a385_64a23b11622d = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _c8c7fa2e_8ced_4ef8_9f4f_57bfe2a18925 = r.xnorboo(_55aca479_89b9_4d1e_b5b1_f34235ee92e4, _06cb982f_66f2_41e6_a385_64a23b11622d)
    let _7e058d1d_ca39_4f4d_9601_b7dcd6962b56 = r.reff(_c8c7fa2e_8ced_4ef8_9f4f_57bfe2a18925)
    let _cf412628_cf52_4f11_94a3_d1c89b1656d4 = r.copystr(_3d6baa22_5c6c_4434_82b5_cf4a8e0c83c2)
    let _156a4a05_2e29_48ed_bb81_b60473ef6f04 = r.boolstr(_7e058d1d_ca39_4f4d_9601_b7dcd6962b56)
    let _5457ff42_7e5e_4ef1_9d4a_f6c5d3da375c = r.refv(_156a4a05_2e29_48ed_bb81_b60473ef6f04)
    let _dd2efe71_0f63_4da0_a573_855fd2513e8f = r.catstr(_cf412628_cf52_4f11_94a3_d1c89b1656d4, _5457ff42_7e5e_4ef1_9d4a_f6c5d3da375c)
    let _ec5cfa4e_482f_4342_9b9a_0587357fbc28 = r.refv(_dd2efe71_0f63_4da0_a573_855fd2513e8f)
    let _5372fa40_da2f_4fbe_90bc_c5fd8e0b9adc = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _266fc236_fbe5_45ad_b853_8748e4a273d0 = r.catstr(_ec5cfa4e_482f_4342_9b9a_0587357fbc28, _5372fa40_da2f_4fbe_90bc_c5fd8e0b9adc)
    let _feb52339_4408_4b37_a252_946ed5786afc = r.refv(_266fc236_fbe5_45ad_b853_8748e4a273d0)
    r.emit('stdout', _feb52339_4408_4b37_a252_946ed5786afc)
    let _c6e6dca9_2ec3_4641_8818_4136c20bfbe4 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _637ef87e_5268_4cc0_932f_24ba6273ce7e = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _206d45e1_160e_4b1b_8c42_b16afd7d22ee = r.orbool(_c6e6dca9_2ec3_4641_8818_4136c20bfbe4, _637ef87e_5268_4cc0_932f_24ba6273ce7e)
    let _e0c74c51_aef0_4cbc_a3f5_271673075be1 = r.reff(_206d45e1_160e_4b1b_8c42_b16afd7d22ee)
    let _15e178ff_1145_44d1_8d6a_6cdca9c52736 = r.copystr(_836107dd_5914_4609_96c7_3ade3ea5ae25)
    let _aa489dd5_3c6b_4dda_b7fa_f58527c16336 = r.boolstr(_e0c74c51_aef0_4cbc_a3f5_271673075be1)
    let _5e0501ce_42eb_459d_bda7_e331de426a75 = r.refv(_aa489dd5_3c6b_4dda_b7fa_f58527c16336)
    let _51d45cc3_b4f1_41be_bead_ff60a0c13421 = r.catstr(_15e178ff_1145_44d1_8d6a_6cdca9c52736, _5e0501ce_42eb_459d_bda7_e331de426a75)
    let _2a7cdc87_b705_4cb5_9f3b_d55078807173 = r.refv(_51d45cc3_b4f1_41be_bead_ff60a0c13421)
    let _eae97cb0_4441_4bab_8cd1_5c5e0655f746 = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _84a08e88_faea_4fed_9fca_3599b92eed2f = r.catstr(_2a7cdc87_b705_4cb5_9f3b_d55078807173, _eae97cb0_4441_4bab_8cd1_5c5e0655f746)
    let _c25080a1_d69b_4d8e_87f3_718850047548 = r.refv(_84a08e88_faea_4fed_9fca_3599b92eed2f)
    r.emit('stdout', _c25080a1_d69b_4d8e_87f3_718850047548)
    let _ace96fab_39d9_402c_a68c_1bde7baba885 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _057020fb_f8b6_457b_b344_f623b77ad667 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _f9ba412c_4fb3_4c23_b613_26a5bae5d2ff = r.orbool(_ace96fab_39d9_402c_a68c_1bde7baba885, _057020fb_f8b6_457b_b344_f623b77ad667)
    let _6ea3d6f1_73b0_408b_bf7b_2f2cf650d91c = r.reff(_f9ba412c_4fb3_4c23_b613_26a5bae5d2ff)
    let _049cba0d_0b33_4517_a90d_e2116925fbdd = r.reff(_6ea3d6f1_73b0_408b_bf7b_2f2cf650d91c)
    let _b1b41f89_7e0a_4dbb_98c1_792e8ce259cf = r.copystr(_b42afa46_b8f6_46f0_8901_85ff466bc94c)
    let _c6f6eb46_dc7d_4944_9ce3_47305f0f404a = r.boolstr(_049cba0d_0b33_4517_a90d_e2116925fbdd)
    let _f37bcb05_569b_44f5_8830_14ca993aa314 = r.refv(_c6f6eb46_dc7d_4944_9ce3_47305f0f404a)
    let _2627fa62_9802_4e11_8d9a_f959848784ee = r.catstr(_b1b41f89_7e0a_4dbb_98c1_792e8ce259cf, _f37bcb05_569b_44f5_8830_14ca993aa314)
    let _74fa1110_1db1_4e08_976c_67a478f778da = r.refv(_2627fa62_9802_4e11_8d9a_f959848784ee)
    let _87e3d997_00ff_4711_893e_1bfcc09d4a7c = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _a2f70dad_046c_4abb_95db_45525c3ca3f3 = r.catstr(_74fa1110_1db1_4e08_976c_67a478f778da, _87e3d997_00ff_4711_893e_1bfcc09d4a7c)
    let _4b6087b6_e69d_426b_a204_a29d4432ee3c = r.refv(_a2f70dad_046c_4abb_95db_45525c3ca3f3)
    r.emit('stdout', _4b6087b6_e69d_426b_a204_a29d4432ee3c)
    let _606d47dc_ae15_4a52_80f2_ae5f84d2bb04 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _f6b75ed5_11c9_416f_ac95_12054c2e49b8 = r.copybool(_242425c5_4430_4911_b579_5b8249ac9d5c)
    let _c17ff9d1_834e_4b88_8df6_1a5595c97f8c = r.norbool(_606d47dc_ae15_4a52_80f2_ae5f84d2bb04, _f6b75ed5_11c9_416f_ac95_12054c2e49b8)
    let _fb7ecf16_6ff6_4527_852d_c1a5d7f333ac = r.reff(_c17ff9d1_834e_4b88_8df6_1a5595c97f8c)
    let _259bff4d_34a4_4dd7_8ce1_79d736c40a87 = r.copystr(_85a79e83_7dff_4308_b342_8f0ebb7eb23a)
    let _17a6f96a_b531_4f5d_9f20_d91df00ef664 = r.boolstr(_fb7ecf16_6ff6_4527_852d_c1a5d7f333ac)
    let _1ff434d3_25df_4f85_b0b5_35cc19412802 = r.refv(_17a6f96a_b531_4f5d_9f20_d91df00ef664)
    let _783f8fe9_f447_447b_8d0f_700dbb7fab43 = r.catstr(_259bff4d_34a4_4dd7_8ce1_79d736c40a87, _1ff434d3_25df_4f85_b0b5_35cc19412802)
    let _92617cfe_393f_4a68_8052_66a486e29571 = r.refv(_783f8fe9_f447_447b_8d0f_700dbb7fab43)
    let _aadcf6df_9af1_4272_a41d_54808002da8e = r.copystr(_b0722764_e10c_4694_a3b1_581f14bd582b)
    let _cb114978_f8d3_453e_8b4f_dd9910bd7c2b = r.catstr(_92617cfe_393f_4a68_8052_66a486e29571, _aadcf6df_9af1_4272_a41d_54808002da8e)
    let _45fb5ba3_a4f7_4172_88ab_d9915d2decae = r.refv(_cb114978_f8d3_453e_8b4f_dd9910bd7c2b)
    r.emit('stdout', _45fb5ba3_a4f7_4172_88ab_d9915d2decae)
    let _c3f674ec_38b4_450d_bad4_b1af79e86a2b = r.copyi64(_cb3c2d2d_29a8_49b4_bcb4_28b19aff8290)
    let _42fa7474_4643_49b8_95c0_3a3308d6106f = await r.waitop(_c3f674ec_38b4_450d_bad4_b1af79e86a2b)
    let _62fee08e_2730_4ec2_98a3_23952d9a437a = r.copyi8(_35d45f77_d05c_4433_b913_acd109c80237)
    r.emit('exit', _62fee08e_2730_4ec2_98a3_23952d9a437a)
  })
r.emit('_start', undefined)
