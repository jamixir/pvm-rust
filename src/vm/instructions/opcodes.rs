pub const TRAP: u8 = 0;
pub const FALLTHROUGH: u8 = 1;
pub const ECALLI: u8 = 10;
pub const LOAD_IMM_64: u8 = 20;
pub const STORE_IMM_U8: u8 = 30;
pub const STORE_IMM_U16: u8 = 31;
pub const STORE_IMM_U32: u8 = 32;
pub const STORE_IMM_U64: u8 = 33;
pub const JUMP: u8 = 40;
pub const JUMP_IND: u8 = 50;
pub const LOAD_IMM: u8 = 51;
pub const LOAD_U8: u8 = 52;
pub const LOAD_I8: u8 = 53;
pub const LOAD_U16: u8 = 54;
pub const LOAD_I16: u8 = 55;
pub const LOAD_U32: u8 = 56;
pub const LOAD_I32: u8 = 57;
pub const LOAD_U64: u8 = 58;
pub const STORE_U8: u8 = 59;
pub const STORE_U16: u8 = 60;
pub const STORE_U32: u8 = 61;
pub const STORE_U64: u8 = 62;
pub const STORE_IMM_IND_U8: u8 = 70;
pub const STORE_IMM_IND_U16: u8 = 71;
pub const STORE_IMM_IND_U32: u8 = 72;
pub const STORE_IMM_IND_U64: u8 = 73;
pub const LOAD_IMM_JUMP: u8 = 80;
pub const BRANCH_EQ_IMM: u8 = 81;
pub const BRANCH_NE_IMM: u8 = 82;
pub const BRANCH_LT_U_IMM: u8 = 83;
pub const BRANCH_LE_U_IMM: u8 = 84;
pub const BRANCH_GE_U_IMM: u8 = 85;
pub const BRANCH_GT_U_IMM: u8 = 86;
pub const BRANCH_LT_S_IMM: u8 = 87;
pub const BRANCH_LE_S_IMM: u8 = 88;
pub const BRANCH_GE_S_IMM: u8 = 89;
pub const BRANCH_GT_S_IMM: u8 = 90;
pub const MOVE_REG: u8 = 100;
pub const SBRK: u8 = 101;
pub const COUNT_SET_BITS_64: u8 = 102;
pub const COUNT_SET_BITS_32: u8 = 103;
pub const LEADING_ZERO_BITS_64: u8 = 104;
pub const LEADING_ZERO_BITS_32: u8 = 105;
pub const TRAILING_ZERO_BITS_64: u8 = 106;
pub const TRAILING_ZERO_BITS_32: u8 = 107;
pub const SIGN_EXTEND_8: u8 = 108;
pub const SIGN_EXTEND_16: u8 = 109;
pub const ZERO_EXTEND_16: u8 = 110;
pub const REVERSE_BYTES: u8 = 111;
pub const STORE_IND_U8: u8 = 120;
pub const STORE_IND_U16: u8 = 121;
pub const STORE_IND_U32: u8 = 122;
pub const STORE_IND_U64: u8 = 123;
pub const LOAD_IND_U8: u8 = 124;
pub const LOAD_IND_I8: u8 = 125;
pub const LOAD_IND_U16: u8 = 126;
pub const LOAD_IND_I16: u8 = 127;
pub const LOAD_IND_U32: u8 = 128;
pub const LOAD_IND_I32: u8 = 129;
pub const LOAD_IND_U64: u8 = 130;
pub const ADD_IMM_32: u8 = 131;
pub const AND_IMM: u8 = 132;
pub const XOR_IMM: u8 = 133;
pub const OR_IMM: u8 = 134;
pub const MUL_IMM_32: u8 = 135;
pub const SET_LT_U_IMM: u8 = 136;
pub const SET_LT_S_IMM: u8 = 137;
pub const SHLO_L_IMM_32: u8 = 138;
pub const SHLO_R_IMM_32: u8 = 139;
pub const SHAR_R_IMM_32: u8 = 140;
pub const NEG_ADD_IMM_32: u8 = 141;
pub const SET_GT_U_IMM: u8 = 142;
pub const SET_GT_S_IMM: u8 = 143;
pub const SHLO_L_IMM_ALT_32: u8 = 144;
pub const SHLO_R_IMM_ALT_32: u8 = 145;
pub const SHAR_R_IMM_ALT_32: u8 = 146;
pub const CMOV_IZ_IMM: u8 = 147;
pub const CMOV_NZ_IMM: u8 = 148;
pub const ADD_IMM_64: u8 = 149;
pub const MUL_IMM_64: u8 = 150;
pub const SHLO_L_IMM_64: u8 = 151;
pub const SHLO_R_IMM_64: u8 = 152;
pub const SHAR_R_IMM_64: u8 = 153;
pub const NEG_ADD_IMM_64: u8 = 154;
pub const SHLO_L_IMM_ALT_64: u8 = 155;
pub const SHLO_R_IMM_ALT_64: u8 = 156;
pub const SHAR_R_IMM_ALT_64: u8 = 157;
pub const ROT_R_64_IMM: u8 = 158;
pub const ROT_R_64_IMM_ALT: u8 = 159;
pub const ROT_R_32_IMM: u8 = 160;
pub const ROT_R_32_IMM_ALT: u8 = 161;
pub const BRANCH_EQ: u8 = 170;
pub const BRANCH_NE: u8 = 171;
pub const BRANCH_LT_U: u8 = 172;
pub const BRANCH_LT_S: u8 = 173;
pub const BRANCH_GE_U: u8 = 174;
pub const BRANCH_GE_S: u8 = 175;
pub const LOAD_IMM_JUMP_IND: u8 = 180;
pub const ADD_32: u8 = 190;
pub const SUB_32: u8 = 191;
pub const MUL_32: u8 = 192;
pub const DIV_U_32: u8 = 193;
pub const DIV_S_32: u8 = 194;
pub const REM_U_32: u8 = 195;
pub const REM_S_32: u8 = 196;
pub const SHLO_L_32: u8 = 197;
pub const SHLO_R_32: u8 = 198;
pub const SHAR_R_32: u8 = 199;
pub const ADD_64: u8 = 200;
pub const SUB_64: u8 = 201;
pub const MUL_64: u8 = 202;
pub const DIV_U_64: u8 = 203;
pub const DIV_S_64: u8 = 204;
pub const REM_U_64: u8 = 205;
pub const REM_S_64: u8 = 206;
pub const SHLO_L_64: u8 = 207;
pub const SHLO_R_64: u8 = 208;
pub const SHAR_R_64: u8 = 209;
pub const AND: u8 = 210;
pub const XOR: u8 = 211;
pub const OR: u8 = 212;
pub const MUL_UPPER_S_S: u8 = 213;
pub const MUL_UPPER_U_U: u8 = 214;
pub const MUL_UPPER_S_U: u8 = 215;
pub const SET_LT_U: u8 = 216;
pub const SET_LT_S: u8 = 217;
pub const CMOV_IZ: u8 = 218;
pub const CMOV_NZ: u8 = 219;
pub const ROT_L_64: u8 = 220;
pub const ROT_L_32: u8 = 221;
pub const ROT_R_64: u8 = 222;
pub const ROT_R_32: u8 = 223;
pub const AND_INV: u8 = 224;
pub const OR_INV: u8 = 225;
pub const XNOR: u8 = 226;
pub const MAX: u8 = 227;
pub const MAX_U: u8 = 228;
pub const MIN: u8 = 229;
pub const MIN_U: u8 = 230;
pub const UNKNOWN: &str = "UNKNOWN";

pub const OPCODE_NAMES: [&str; 256] = [
    "TRAP",                 // 0
    "FALLTHROUGH",          // 1
    UNKNOWN,              // 2
    UNKNOWN,              // 3
    UNKNOWN,              // 4
    UNKNOWN,              // 5
    UNKNOWN,              // 6
    UNKNOWN,              // 7
    UNKNOWN,              // 8
    UNKNOWN,              // 9
    "ECALLI",               // 10
    UNKNOWN,              // 11
    UNKNOWN,              // 12
    UNKNOWN,              // 13
    UNKNOWN,              // 14
    UNKNOWN,              // 15
    UNKNOWN,              // 16
    UNKNOWN,              // 17
    UNKNOWN,              // 18
    UNKNOWN,              // 19
    "LOAD_IMM_64",          // 20
    UNKNOWN,              // 21
    UNKNOWN,              // 22
    UNKNOWN,              // 23
    UNKNOWN,              // 24
    UNKNOWN,              // 25
    UNKNOWN,              // 26
    UNKNOWN,              // 27
    UNKNOWN,              // 28
    UNKNOWN,              // 29
    "STORE_IMM_U8",         // 30
    "STORE_IMM_U16",        // 31
    "STORE_IMM_U32",        // 32
    "STORE_IMM_U64",        // 33
    UNKNOWN,              // 34
    UNKNOWN,              // 35
    UNKNOWN,              // 36
    UNKNOWN,              // 37
    UNKNOWN,              // 38
    UNKNOWN,              // 39
    "JUMP",                 // 40
    UNKNOWN,              // 41
    UNKNOWN,              // 42
    UNKNOWN,              // 43
    UNKNOWN,              // 44
    UNKNOWN,              // 45
    UNKNOWN,              // 46
    UNKNOWN,              // 47
    UNKNOWN,              // 48
    UNKNOWN,              // 49
    "JUMP_IND",             // 50
    "LOAD_IMM",             // 51
    "LOAD_U8",              // 52
    "LOAD_I8",              // 53
    "LOAD_U16",             // 54
    "LOAD_I16",             // 55
    "LOAD_U32",             // 56
    "LOAD_I32",             // 57
    "LOAD_U64",             // 58
    "STORE_U8",             // 59
    "STORE_U16",            // 60
    "STORE_U32",            // 61
    "STORE_U64",            // 62
    UNKNOWN,              // 63
    UNKNOWN,              // 64
    UNKNOWN,              // 65
    UNKNOWN,              // 66
    UNKNOWN,              // 67
    UNKNOWN,              // 68
    UNKNOWN,              // 69
    "STORE_IMM_IND_U8",     // 70
    "STORE_IMM_IND_U16",    // 71
    "STORE_IMM_IND_U32",    // 72
    "STORE_IMM_IND_U64",    // 73
    UNKNOWN,              // 74
    UNKNOWN,              // 75
    UNKNOWN,              // 76
    UNKNOWN,              // 77
    UNKNOWN,              // 78
    UNKNOWN,              // 79
    "LOAD_IMM_JUMP",        // 80
    "BRANCH_EQ_IMM",        // 81
    "BRANCH_NE_IMM",        // 82
    "BRANCH_LT_U_IMM",      // 83
    "BRANCH_LE_U_IMM",      // 84
    "BRANCH_GE_U_IMM",      // 85
    "BRANCH_GT_U_IMM",      // 86
    "BRANCH_LT_S_IMM",      // 87
    "BRANCH_LE_S_IMM",      // 88
    "BRANCH_GE_S_IMM",      // 89
    "BRANCH_GT_S_IMM",      // 90
    UNKNOWN,              // 91
    UNKNOWN,              // 92
    UNKNOWN,              // 93
    UNKNOWN,              // 94
    UNKNOWN,              // 95
    UNKNOWN,              // 96
    UNKNOWN,              // 97
    UNKNOWN,              // 98
    UNKNOWN,              // 99
    "MOVE_REG",             // 100
    "SBRK",                 // 101
    "COUNT_SET_BITS_64",    // 102
    "COUNT_SET_BITS_32",    // 103
    "LEADING_ZERO_BITS_64", // 104
    "LEADING_ZERO_BITS_32", // 105
    "TRAILING_ZERO_BITS_64", // 106
    "TRAILING_ZERO_BITS_32", // 107
    "SIGN_EXTEND_8",        // 108
    "SIGN_EXTEND_16",       // 109
    "ZERO_EXTEND_16",       // 110
    "REVERSE_BYTES",        // 111
    UNKNOWN,              // 112
    UNKNOWN,              // 113
    UNKNOWN,              // 114
    UNKNOWN,              // 115
    UNKNOWN,              // 116
    UNKNOWN,              // 117
    UNKNOWN,              // 118
    UNKNOWN,              // 119
    "STORE_IND_U8",         // 120
    "STORE_IND_U16",        // 121
    "STORE_IND_U32",        // 122
    "STORE_IND_U64",        // 123
    "LOAD_IND_U8",          // 124
    "LOAD_IND_I8",          // 125
    "LOAD_IND_U16",         // 126
    "LOAD_IND_I16",         // 127
    "LOAD_IND_U32",         // 128
    "LOAD_IND_I32",         // 129
    "LOAD_IND_U64",         // 130
    "ADD_IMM_32",           // 131
    "AND_IMM",              // 132
    "XOR_IMM",              // 133
    "OR_IMM",               // 134
    "MUL_IMM_32",           // 135
    "SET_LT_U_IMM",         // 136
    "SET_LT_S_IMM",         // 137
    "SHLO_L_IMM_32",        // 138
    "SHLO_R_IMM_32",        // 139
    "SHAR_R_IMM_32",        // 140
    "NEG_ADD_IMM_32",       // 141
    "SET_GT_U_IMM",         // 142
    "SET_GT_S_IMM",         // 143
    "SHLO_L_IMM_ALT_32",    // 144
    "SHLO_R_IMM_ALT_32",    // 145
    "SHAR_R_IMM_ALT_32",    // 146
    "CMOV_IZ_IMM",          // 147
    "CMOV_NZ_IMM",          // 148
    "ADD_IMM_64",           // 149
    "MUL_IMM_64",           // 150
    "SHLO_L_IMM_64",        // 151
    "SHLO_R_IMM_64",        // 152
    "SHAR_R_IMM_64",        // 153
    "NEG_ADD_IMM_64",       // 154
    "SHLO_L_IMM_ALT_64",    // 155
    "SHLO_R_IMM_ALT_64",    // 156
    "SHAR_R_IMM_ALT_64",    // 157
    "ROT_R_64_IMM",         // 158
    "ROT_R_64_IMM_ALT",     // 159
    "ROT_R_32_IMM",         // 160
    "ROT_R_32_IMM_ALT",     // 161
    UNKNOWN,              // 162
    UNKNOWN,              // 163
    UNKNOWN,              // 164
    UNKNOWN,              // 165
    UNKNOWN,              // 166
    UNKNOWN,              // 167
    UNKNOWN,              // 168
    UNKNOWN,              // 169
    "BRANCH_EQ",            // 170
    "BRANCH_NE",            // 171
    "BRANCH_LT_U",          // 172
    "BRANCH_LT_S",          // 173
    "BRANCH_GE_U",          // 174
    "BRANCH_GE_S",          // 175
    UNKNOWN,              // 176
    UNKNOWN,              // 177
    UNKNOWN,              // 178
    UNKNOWN,              // 179
    "LOAD_IMM_JUMP_IND",    // 180
    UNKNOWN,              // 181
    UNKNOWN,              // 182
    UNKNOWN,              // 183
    UNKNOWN,              // 184
    UNKNOWN,              // 185
    UNKNOWN,              // 186
    UNKNOWN,              // 187
    UNKNOWN,              // 188
    UNKNOWN,              // 189
    "ADD_32",               // 190
    "SUB_32",               // 191
    "MUL_32",               // 192
    "DIV_U_32",             // 193
    "DIV_S_32",             // 194
    "REM_U_32",             // 195
    "REM_S_32",             // 196
    "SHLO_L_32",            // 197
    "SHLO_R_32",            // 198
    "SHAR_R_32",            // 199
    "ADD_64",               // 200
    "SUB_64",               // 201
    "MUL_64",               // 202
    "DIV_U_64",             // 203
    "DIV_S_64",             // 204
    "REM_U_64",             // 205
    "REM_S_64",             // 206
    "SHLO_L_64",            // 207
    "SHLO_R_64",            // 208
    "SHAR_R_64",            // 209
    "AND",                  // 210
    "XOR",                  // 211
    "OR",                   // 212
    "MUL_UPPER_S_S",        // 213
    "MUL_UPPER_U_U",        // 214
    "MUL_UPPER_S_U",        // 215
    "SET_LT_U",             // 216
    "SET_LT_S",             // 217
    "CMOV_IZ",              // 218
    "CMOV_NZ",              // 219
    "ROT_L_64",             // 220
    "ROT_L_32",             // 221
    "ROT_R_64",             // 222
    "ROT_R_32",             // 223
    "AND_INV",              // 224
    "OR_INV",               // 225
    "XNOR",                 // 226
    "MAX",                  // 227
    "MAX_U",                // 228
    "MIN",                  // 229
    "MIN_U",                // 230
    UNKNOWN,              // 231
    UNKNOWN,              // 232
    UNKNOWN,              // 233
    UNKNOWN,              // 234
    UNKNOWN,              // 235
    UNKNOWN,              // 236
    UNKNOWN,              // 237
    UNKNOWN,              // 238
    UNKNOWN,              // 239
    UNKNOWN,              // 240
    UNKNOWN,              // 241
    UNKNOWN,              // 242
    UNKNOWN,              // 243
    UNKNOWN,              // 244
    UNKNOWN,              // 245
    UNKNOWN,              // 246
    UNKNOWN,              // 247
    UNKNOWN,              // 248
    UNKNOWN,              // 249
    UNKNOWN,              // 250
    UNKNOWN,              // 251
    UNKNOWN,              // 252
    UNKNOWN,              // 253
    UNKNOWN,              // 254
    UNKNOWN,              // 255
];

pub fn opcode_to_name(opcode: u8) -> &'static str {
    OPCODE_NAMES[opcode as usize]
}
