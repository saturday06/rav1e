// Copyright (c) 2017-2018, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#![allow(safe_extern_statics)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
#![cfg_attr(feature = "cargo-clippy", allow(unnecessary_mut_passed))]
#![cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
#![cfg_attr(feature = "cargo-clippy", allow(collapsible_if))]

use ec;
use partition::BlockSize::*;
use partition::PredictionMode::*;
use partition::TxType::*;
use partition::*;
use plane::*;
use std::*;

const PLANES: usize = 3;

const PARTITION_PLOFFSET: usize = 4;
const PARTITION_CONTEXTS: usize = 20;
pub const PARTITION_TYPES: usize = 4;

pub const MI_SIZE_LOG2: usize = 2;
const MI_SIZE: usize = (1 << MI_SIZE_LOG2);
const MAX_MIB_SIZE_LOG2: usize = (MAX_SB_SIZE_LOG2 - MI_SIZE_LOG2);
pub const MAX_MIB_SIZE: usize = (1 << MAX_MIB_SIZE_LOG2);
pub const MAX_MIB_MASK: usize = (MAX_MIB_SIZE - 1);

const MAX_SB_SIZE_LOG2: usize = 6;
const MAX_SB_SIZE: usize = (1 << MAX_SB_SIZE_LOG2);
const MAX_SB_SQUARE: usize = (MAX_SB_SIZE * MAX_SB_SIZE);

pub const MAX_TX_SIZE: usize = 32;
const MAX_TX_SQUARE: usize = MAX_TX_SIZE * MAX_TX_SIZE;

const INTRA_MODES: usize = 13;
const UV_INTRA_MODES: usize = 14;
const BLOCK_SIZE_GROUPS: usize = 4;
const MAX_ANGLE_DELTA: usize = 3;
const DIRECTIONAL_MODES: usize = 8;
const KF_MODE_CONTEXTS: usize = 5;

const TX_SIZES: usize = 4;
const TX_SETS: usize = 9;
const TX_SETS_INTRA: usize = 3;
const TX_SETS_INTER: usize = 4;
// Number of transform types in each set type
static num_tx_set: [usize; TX_SETS] =
  [1, 2, 5, 7, 7, 10, 12, 16, 16];
pub static av1_tx_used: [[usize; TX_TYPES]; TX_SETS] = [
  [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
  [1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
  [1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
  [1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
];

// Maps set types above to the indices used for intra
static tx_set_index_intra: [i8; TX_SETS] =
  [0, -1, 2, -1, 1, -1, -1, -1, -1];
// Maps set types above to the indices used for inter
static tx_set_index_inter: [i8; TX_SETS] =
  [0, 3, -1, -1, -1, -1, 2, -1, 1];

static av1_tx_ind: [[usize; TX_TYPES]; TX_SETS] = [
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [1, 3, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [1, 5, 6, 4, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0, 0, 0],
  [1, 5, 6, 4, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0, 0, 0],
  [1, 2, 3, 6, 4, 5, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0],
  [3, 4, 5, 8, 6, 7, 9, 10, 11, 0, 1, 2, 0, 0, 0, 0],
  [7, 8, 9, 12, 10, 11, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6],
  [7, 8, 9, 12, 10, 11, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6]
];

static av1_coefband_trans_4x4: [u8; 16] =
  [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5];

static av1_coefband_trans_8x8plus: [u8; 32 * 32] = [
  0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5,
  // beyond MAXBAND_INDEX+1 all values are filled as 5
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];

static ss_size_lookup: [[[BlockSize; 2]; 2]; BlockSize::BLOCK_SIZES_ALL] = [
  //  ss_x == 0    ss_x == 0        ss_x == 1      ss_x == 1
  //  ss_y == 0    ss_y == 1        ss_y == 0      ss_y == 1
  [  [ BLOCK_4X4, BLOCK_4X4 ], [BLOCK_4X4, BLOCK_4X4 ] ],
  [  [ BLOCK_4X8, BLOCK_4X4 ], [BLOCK_4X4, BLOCK_4X4 ] ],
  [  [ BLOCK_8X4, BLOCK_4X4 ], [BLOCK_4X4, BLOCK_4X4 ] ],
  [  [ BLOCK_8X8, BLOCK_8X4 ], [BLOCK_4X8, BLOCK_4X4 ] ],
  [  [ BLOCK_8X16, BLOCK_8X8 ], [BLOCK_4X16, BLOCK_4X8 ] ],
  [  [ BLOCK_16X8, BLOCK_16X4 ], [BLOCK_8X8, BLOCK_8X4 ] ],
  [  [ BLOCK_16X16, BLOCK_16X8 ], [BLOCK_8X16, BLOCK_8X8 ] ],
  [  [ BLOCK_16X32, BLOCK_16X16 ], [BLOCK_8X32, BLOCK_8X16 ] ],
  [  [ BLOCK_32X16, BLOCK_32X8 ], [BLOCK_16X16, BLOCK_16X8 ] ],
  [  [ BLOCK_32X32, BLOCK_32X16 ], [BLOCK_16X32, BLOCK_16X16 ] ],
  [  [ BLOCK_32X64, BLOCK_32X32 ], [BLOCK_16X64, BLOCK_16X32 ] ],
  [  [ BLOCK_64X32, BLOCK_64X16 ], [BLOCK_32X32, BLOCK_32X16 ] ],
  [  [ BLOCK_64X64, BLOCK_64X32 ], [BLOCK_32X64, BLOCK_32X32 ] ],
  [  [ BLOCK_4X16, BLOCK_4X8 ], [BLOCK_4X16, BLOCK_4X8 ] ],
  [  [ BLOCK_16X4, BLOCK_16X4 ], [BLOCK_8X4, BLOCK_8X4 ] ],
  [  [ BLOCK_8X32, BLOCK_8X16 ], [BLOCK_INVALID, BLOCK_4X16 ] ],
  [  [ BLOCK_32X8, BLOCK_INVALID ], [BLOCK_16X8, BLOCK_16X4 ] ],
  [  [ BLOCK_16X64, BLOCK_16X32 ], [BLOCK_INVALID, BLOCK_8X32 ] ],
  [  [ BLOCK_64X16, BLOCK_INVALID ], [BLOCK_32X16, BLOCK_32X8 ] ],
];

pub fn get_plane_block_size(bsize: BlockSize, subsampling_x: usize, subsampling_y: usize)
    -> BlockSize {
  ss_size_lookup[bsize as usize][subsampling_x][subsampling_y]
}

// Generates 4 bit field in which each bit set to 1 represents
// a blocksize partition  1111 means we split 64x64, 32x32, 16x16
// and 8x8.  1000 means we just split the 64x64 to 32x32
static partition_context_lookup: [[u8; 2]; BlockSize::BLOCK_SIZES_ALL] = [
  [ 15, 15 ],  // 4X4   - [0b1111, 0b1111]
  [ 15, 14 ],  // 4X8   - [0b1111, 0b1110]
  [ 14, 15 ],  // 8X4   - [0b1110, 0b1111]
  [ 14, 14 ],  // 8X8   - [0b1110, 0b1110]
  [ 14, 12 ],  // 8X16  - [0b1110, 0b1100]
  [ 12, 14 ],  // 16X8  - [0b1100, 0b1110]
  [ 12, 12 ],  // 16X16 - [0b1100, 0b1100]
  [ 12, 8 ],   // 16X32 - [0b1100, 0b1000]
  [ 8, 12 ],   // 32X16 - [0b1000, 0b1100]
  [ 8, 8 ],    // 32X32 - [0b1000, 0b1000]
  [ 8, 0 ],    // 32X64 - [0b1000, 0b0000]
  [ 0, 8 ],    // 64X32 - [0b0000, 0b1000]
  [ 0, 0 ],    // 64X64 - [0b0000, 0b0000]

  [ 15, 12 ],  // 4X16 - [0b1111, 0b1100]
  [ 12, 15 ],  // 16X4 - [0b1100, 0b1111]
  [ 8, 14 ],   // 8X32 - [0b1110, 0b1000]
  [ 14, 8 ],   // 32X8 - [0b1000, 0b1110]
  [ 12, 0 ],   // 16X64- [0b1100, 0b0000]
  [ 0, 12 ],   // 64X16- [0b0000, 0b1100]
];

static size_group_lookup: [u8; BlockSize::BLOCK_SIZES_ALL] = [
  0, 0,
  0, 1,
  1, 1,
  2, 2,
  2, 3,
  3, 3,
  3, 0,
  0, 1,
  1, 2,
  2,
];

static num_pels_log2_lookup: [u8; BlockSize::BLOCK_SIZES_ALL] = [
  4, 5, 5, 6, 7, 7, 8, 9, 9, 10, 11, 11, 12, 6, 6, 8, 8, 10, 10];

pub static subsize_lookup: [[BlockSize; BlockSize::BLOCK_SIZES_ALL]; PARTITION_TYPES] =
[
  [ // PARTITION_NONE
    //                            4X4
                                  BLOCK_4X4,
    // 4X8,        8X4,           8X8
    BLOCK_4X8,     BLOCK_8X4,     BLOCK_8X8,
    // 8X16,       16X8,          16X16
    BLOCK_8X16,    BLOCK_16X8,    BLOCK_16X16,
    // 16X32,      32X16,         32X32
    BLOCK_16X32,   BLOCK_32X16,   BLOCK_32X32,
    // 32X64,      64X32,         64X64
    BLOCK_32X64,   BLOCK_64X32,   BLOCK_64X64,
    // 4X16,       16X4,          8X32
    BLOCK_4X16,    BLOCK_16X4,    BLOCK_8X32,
    // 32X8,       16X64,         64X16
    BLOCK_32X8,    BLOCK_16X64,   BLOCK_64X16,
  ], [  // PARTITION_HORZ
    //                            4X4
                                  BLOCK_INVALID,
    // 4X8,        8X4,           8X8
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_8X4,
    // 8X16,       16X8,          16X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_16X8,
    // 16X32,      32X16,         32X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_32X16,
    // 32X64,      64X32,         64X64
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_64X32,
    // 4X16,       16X4,          8X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
    // 32X8,       16X64,         64X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
  ], [  // PARTITION_VERT
    //                            4X4
                                  BLOCK_INVALID,
    // 4X8,        8X4,           8X8
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_4X8,
    // 8X16,       16X8,          16X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_8X16,
    // 16X32,      32X16,         32X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_16X32,
    // 32X64,      64X32,         64X64
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_32X64,
    // 4X16,       16X4,          8X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
    // 32X8,       16X64,         64X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
  ], [  // PARTITION_SPLIT
    //                            4X4
                                  BLOCK_INVALID,
    // 4X8,        8X4,           8X8
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_4X4,
    // 8X16,       16X8,          16X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_8X8,
    // 16X32,      32X16,         32X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_16X16,
    // 32X64,      64X32,         64X64
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_32X32,
    // 4X16,       16X4,          8X32
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
    // 32X8,       16X64,         64X16
    BLOCK_INVALID, BLOCK_INVALID, BLOCK_INVALID,
  ]
];

#[derive(Copy,Clone,PartialEq)]
#[allow(dead_code)]
enum HeadToken {
    BlockZero = 0,
    Zero = 1,
    OneEOB = 2,
    OneNEOB = 3,
    TwoPlusEOB = 4,
    TwoPlusNEOB = 5,
}

#[derive(Copy,Clone,PartialEq)]
#[allow(dead_code)]
enum TailToken {
    Two = 0,
    Three = 1,
    Four = 2,
    Cat1 = 3,
    Cat2 = 4,
    Cat3 = 5,
    Cat4 = 6,
    Cat5 = 7,
    Cat6 = 8,
}

const PLANE_TYPES: usize = 2;
const REF_TYPES: usize = 2;
const SKIP_CONTEXTS: usize = 3;
const INTRA_INTER_CONTEXTS: usize = 4;

// Level Map
const TXB_SKIP_CONTEXTS: usize =  13;

const EOB_COEF_CONTEXTS: usize =  22;

const SIG_COEF_CONTEXTS_2D: usize =  26;
const SIG_COEF_CONTEXTS_1D: usize =  16;
const SIG_COEF_CONTEXTS_EOB: usize =  4;
const SIG_COEF_CONTEXTS: usize = SIG_COEF_CONTEXTS_2D + SIG_COEF_CONTEXTS_1D;

const COEFF_BASE_CONTEXTS: usize = SIG_COEF_CONTEXTS;
const DC_SIGN_CONTEXTS: usize =  3;

const BR_TMP_OFFSET: usize =  12;
const BR_REF_CAT: usize =  4;
const LEVEL_CONTEXTS: usize =  21;

const NUM_BASE_LEVELS: usize =  2;

const BR_CDF_SIZE: usize = 4;
const COEFF_BASE_RANGE: usize = 4 * (BR_CDF_SIZE - 1);

const COEFF_CONTEXT_BITS: usize = 6;
const COEFF_CONTEXT_MASK: usize = (1 << COEFF_CONTEXT_BITS) - 1;
const MAX_BASE_BR_RANGE: usize = COEFF_BASE_RANGE + NUM_BASE_LEVELS + 1;

const BASE_CONTEXT_POSITION_NUM: usize = 12;

// Pad 4 extra columns to remove horizontal availability check.
const TX_PAD_HOR_LOG2: usize = 2;
const TX_PAD_HOR: usize = 4;
// Pad 6 extra rows (2 on top and 4 on bottom) to remove vertical availability
// check.
const TX_PAD_TOP: usize = 2;
const TX_PAD_BOTTOM: usize = 4;
const TX_PAD_VER: usize = (TX_PAD_TOP + TX_PAD_BOTTOM);
// Pad 16 extra bytes to avoid reading overflow in SIMD optimization.
const TX_PAD_END: usize = 16;
const TX_PAD_2D: usize =
  ((MAX_TX_SIZE + TX_PAD_HOR) * (MAX_TX_SIZE + TX_PAD_VER) + TX_PAD_END);

const TX_CLASSES: usize = 3;

#[derive(Copy, Clone, PartialEq)]
pub enum TxClass {
  TX_CLASS_2D = 0,
  TX_CLASS_HORIZ = 1,
  TX_CLASS_VERT = 2
}

use context::TxClass::*;

static tx_type_to_class: [TxClass; TX_TYPES] = [
  TX_CLASS_2D,    // DCT_DCT
  TX_CLASS_2D,    // ADST_DCT
  TX_CLASS_2D,    // DCT_ADST
  TX_CLASS_2D,    // ADST_ADST
  TX_CLASS_2D,    // FLIPADST_DCT
  TX_CLASS_2D,    // DCT_FLIPADST
  TX_CLASS_2D,    // FLIPADST_FLIPADST
  TX_CLASS_2D,    // ADST_FLIPADST
  TX_CLASS_2D,    // FLIPADST_ADST
  TX_CLASS_2D,    // IDTX
  TX_CLASS_VERT,  // V_DCT
  TX_CLASS_HORIZ, // H_DCT
  TX_CLASS_VERT,  // V_ADST
  TX_CLASS_HORIZ, // H_ADST
  TX_CLASS_VERT,  // V_FLIPADST
  TX_CLASS_HORIZ  // H_FLIPADST
];

static eob_to_pos_small: [u8; 33] = [
    0, 1, 2,                                        // 0-2
    3, 3,                                           // 3-4
    4, 4, 4, 4,                                     // 5-8
    5, 5, 5, 5, 5, 5, 5, 5,                         // 9-16
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6  // 17-32
];

static eob_to_pos_large: [u8; 17] = [
    6,                               // place holder
    7,                               // 33-64
    8,  8,                           // 65-128
    9,  9,  9,  9,                   // 129-256
    10, 10, 10, 10, 10, 10, 10, 10,  // 257-512
    11                               // 513-
];


static k_eob_group_start: [u16; 12] = [ 0, 1, 2, 3, 5, 9,
                                        17, 33, 65, 129, 257, 513 ];
static k_eob_offset_bits: [u16; 12] = [ 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 ];

fn clip_max3(x: u8) -> u8 {
  if x > 3 {
    3
  } else {
    x
  }
}

// The ctx offset table when TX is TX_CLASS_2D.
// TX col and row indices are clamped to 4

#[cfg_attr(rustfmt, rustfmt_skip)]
static av1_nz_map_ctx_offset: [[[i8; 5]; 5]; TxSize::TX_SIZES_ALL] = [
  // TX_4X4
  [
    [ 0,  1,  6,  6, 0],
    [ 1,  6,  6, 21, 0],
    [ 6,  6, 21, 21, 0],
    [ 6, 21, 21, 21, 0],
    [ 0,  0,  0,  0, 0]
  ],
  // TX_8X8
  [
    [ 0,  1,  6,  6, 21],
    [ 1,  6,  6, 21, 21],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_16X16
  [
    [ 0,  1,  6,  6, 21],
    [ 1,  6,  6, 21, 21],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_32X32
  [
    [ 0,  1,  6,  6, 21],
    [ 1,  6,  6, 21, 21],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_64X64
  [
    [ 0,  1,  6,  6, 21],
    [ 1,  6,  6, 21, 21],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_4X8
  [
    [ 0, 11, 11, 11, 0],
    [11, 11, 11, 11, 0],
    [ 6,  6, 21, 21, 0],
    [ 6, 21, 21, 21, 0],
    [21, 21, 21, 21, 0]
  ],
  // TX_8X4
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [ 0,  0,  0,  0, 0]
  ],
  // TX_8X16
  [
    [ 0, 11, 11, 11, 11],
    [11, 11, 11, 11, 11],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_16X8
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21]
  ],
  // TX_16X32
  [
    [ 0, 11, 11, 11, 11],
    [11, 11, 11, 11, 11],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_32X16
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21]
  ],
  // TX_32X64
  [
    [ 0, 11, 11, 11, 11],
    [11, 11, 11, 11, 11],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_64X32
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21]
  ],
  // TX_4X16
  [
    [ 0, 11, 11, 11, 0],
    [11, 11, 11, 11, 0],
    [ 6,  6, 21, 21, 0],
    [ 6, 21, 21, 21, 0],
    [21, 21, 21, 21, 0]
  ],
  // TX_16X4
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [ 0,  0,  0,  0, 0]
  ],
  // TX_8X32
  [
    [ 0, 11, 11, 11, 11],
    [11, 11, 11, 11, 11],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_32X8
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21]
  ],
  // TX_16X64
  [
    [ 0, 11, 11, 11, 11],
    [11, 11, 11, 11, 11],
    [ 6,  6, 21, 21, 21],
    [ 6, 21, 21, 21, 21],
    [21, 21, 21, 21, 21]
  ],
  // TX_64X16
  [
    [ 0, 16,  6,  6, 21],
    [16, 16,  6, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21],
    [16, 16, 21, 21, 21]
  ]
];

const NZ_MAP_CTX_0: usize = SIG_COEF_CONTEXTS_2D;
const NZ_MAP_CTX_5: usize = (NZ_MAP_CTX_0 + 5);
const NZ_MAP_CTX_10: usize = (NZ_MAP_CTX_0 + 10);

static nz_map_ctx_offset_1d: [usize; 32] = [
  NZ_MAP_CTX_0,  NZ_MAP_CTX_5,  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10, NZ_MAP_CTX_10,
  NZ_MAP_CTX_10, NZ_MAP_CTX_10 ];

const CONTEXT_MAG_POSITION_NUM: usize = 3;

static mag_ref_offset_with_txclass: [[[usize; 2]; CONTEXT_MAG_POSITION_NUM]; 3] = [
  [ [ 0, 1 ], [ 1, 0 ], [ 1, 1 ] ],
  [ [ 0, 1 ], [ 1, 0 ], [ 0, 2 ] ],
  [ [ 0, 1 ], [ 1, 0 ], [ 2, 0 ] ] ];

// End of Level Map

pub fn clamp(val: i32, min: i32, max: i32) -> i32 {
  if val < min {
    min
  } else if val > max {
    max
  } else {
    val
  }
}

pub fn has_chroma(
  bo: &BlockOffset, bsize: BlockSize, subsampling_x: usize,
  subsampling_y: usize
) -> bool {
  let bw = bsize.width_mi();
  let bh = bsize.height_mi();

  ((bo.x & 0x01) == 1 || (bw & 0x01) == 0 || subsampling_x == 0)
    && ((bo.y & 0x01) == 1 || (bh & 0x01) == 0 || subsampling_y == 0)
}

pub fn get_tx_set(
  tx_size: TxSize, is_inter: bool, use_reduced_set: bool
) -> TxSet {
  let tx_size_sqr_up = tx_size.sqr_up();
  let tx_size_sqr = tx_size.sqr();
  if tx_size_sqr > TxSize::TX_32X32 {
    TxSet::TX_SET_DCTONLY
  } else if tx_size_sqr_up == TxSize::TX_32X32 {
    if is_inter {
      TxSet::TX_SET_DCT_IDTX
    } else {
      TxSet::TX_SET_DCTONLY
    }
  } else if use_reduced_set {
    if is_inter {
      TxSet::TX_SET_DCT_IDTX
    } else {
      TxSet::TX_SET_DTT4_IDTX
    }
  } else if is_inter {
    if tx_size_sqr == TxSize::TX_16X16 {
      TxSet::TX_SET_DTT9_IDTX_1DDCT
    } else {
      TxSet::TX_SET_ALL16
    }
  } else {
    if tx_size_sqr == TxSize::TX_16X16 {
      TxSet::TX_SET_DTT4_IDTX
    } else {
      TxSet::TX_SET_DTT4_IDTX_1DDCT
    }
  }
}

fn get_tx_set_index(
  tx_size: TxSize, is_inter: bool, use_reduced_set: bool
) -> i8 {
  let set_type = get_tx_set(tx_size, is_inter, use_reduced_set);

  if is_inter {
    tx_set_index_inter[set_type as usize]
  } else {
    tx_set_index_intra[set_type as usize]
  }
}

static intra_mode_to_tx_type_context: [TxType; INTRA_MODES] = [
  DCT_DCT,   // DC
  ADST_DCT,  // V
  DCT_ADST,  // H
  DCT_DCT,   // D45
  ADST_ADST, // D135
  ADST_DCT,  // D117
  DCT_ADST,  // D153
  DCT_ADST,  // D207
  ADST_DCT,  // D63
  ADST_ADST, // SMOOTH
  ADST_DCT,  // SMOOTH_V
  DCT_ADST,  // SMOOTH_H
  ADST_ADST, // PAETH
];

static uv2y: [PredictionMode; UV_INTRA_MODES] = [
  DC_PRED,       // UV_DC_PRED
  V_PRED,        // UV_V_PRED
  H_PRED,        // UV_H_PRED
  D45_PRED,      // UV_D45_PRED
  D135_PRED,     // UV_D135_PRED
  D117_PRED,     // UV_D117_PRED
  D153_PRED,     // UV_D153_PRED
  D207_PRED,     // UV_D207_PRED
  D63_PRED,      // UV_D63_PRED
  SMOOTH_PRED,   // UV_SMOOTH_PRED
  SMOOTH_V_PRED, // UV_SMOOTH_V_PRED
  SMOOTH_H_PRED, // UV_SMOOTH_H_PRED
  PAETH_PRED,    // UV_PAETH_PRED
  DC_PRED        // CFL_PRED
];

pub fn y_intra_mode_to_tx_type_context(pred: PredictionMode) -> TxType {
  intra_mode_to_tx_type_context[pred as usize]
}

pub fn uv_intra_mode_to_tx_type_context(pred: PredictionMode) -> TxType {
  intra_mode_to_tx_type_context[uv2y[pred as usize] as usize]
}

extern "C" {
  static default_partition_cdf:
    [[u16; PARTITION_TYPES + 1]; PARTITION_CONTEXTS];
  static default_kf_y_mode_cdf:
    [[[u16; INTRA_MODES + 1]; KF_MODE_CONTEXTS]; KF_MODE_CONTEXTS];
  static default_if_y_mode_cdf: [[u16; INTRA_MODES + 1]; BLOCK_SIZE_GROUPS];
  static default_uv_mode_cdf: [[[u16; UV_INTRA_MODES + 1]; INTRA_MODES]; 2];
  static default_intra_ext_tx_cdf:
    [[[[u16; TX_TYPES + 1]; INTRA_MODES]; TX_SIZES]; TX_SETS_INTRA];
  static default_inter_ext_tx_cdf:
    [[[u16; TX_TYPES + 1]; TX_SIZES]; TX_SETS_INTER];
  static default_skip_cdfs: [[u16; 3]; SKIP_CONTEXTS];
  static default_intra_inter_cdf: [[u16; 3]; INTRA_INTER_CONTEXTS];
  static default_angle_delta_cdf:
    [[u16; 2 * MAX_ANGLE_DELTA + 1 + 1]; DIRECTIONAL_MODES];
  static default_filter_intra_cdfs: [[u16; 3]; TxSize::TX_SIZES_ALL];

  static av1_inter_scan_orders: [[SCAN_ORDER; TX_TYPES]; TxSize::TX_SIZES_ALL];

  // lv_map
  static av1_default_txb_skip_cdfs:
    [[[[u16; 3]; TXB_SKIP_CONTEXTS]; TxSize::TX_SIZES]; 4];
  static av1_default_dc_sign_cdfs:
    [[[[u16; 3]; DC_SIGN_CONTEXTS]; PLANE_TYPES]; 4];
  static av1_default_eob_extra_cdfs:
    [[[[[u16; 3]; EOB_COEF_CONTEXTS]; PLANE_TYPES]; TxSize::TX_SIZES]; 4];

  static av1_default_eob_multi16_cdfs: [[[[u16; 5 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi32_cdfs: [[[[u16; 6 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi64_cdfs: [[[[u16; 7 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi128_cdfs: [[[[u16; 8 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi256_cdfs: [[[[u16; 9 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi512_cdfs: [[[[u16; 10 + 1]; 2]; PLANE_TYPES]; 4];
  static av1_default_eob_multi1024_cdfs: [[[[u16; 11 + 1]; 2]; PLANE_TYPES]; 4];

  static av1_default_coeff_base_eob_multi_cdfs:
    [[[[[u16; 3 + 1]; SIG_COEF_CONTEXTS_EOB]; PLANE_TYPES]; TxSize::TX_SIZES]; 4];
  static av1_default_coeff_base_multi_cdfs:
    [[[[[u16; 4 + 1]; SIG_COEF_CONTEXTS]; PLANE_TYPES]; TxSize::TX_SIZES]; 4];
  static av1_default_coeff_lps_multi_cdfs: [[[[[u16; BR_CDF_SIZE + 1];
    LEVEL_CONTEXTS]; PLANE_TYPES];
    TxSize::TX_SIZES]; 4];
}

#[repr(C)]
pub struct SCAN_ORDER {
  // FIXME: don't hardcode sizes
  pub scan: &'static [u16; 64 * 64],
  pub iscan: &'static [u16; 64 * 64],
  pub neighbors: &'static [u16; ((64 * 64) + 1) * 2]
}

#[derive(Clone)]
pub struct CDFContext {
  partition_cdf: [[u16; PARTITION_TYPES + 1]; PARTITION_CONTEXTS],
  kf_y_cdf: [[[u16; INTRA_MODES + 1]; KF_MODE_CONTEXTS]; KF_MODE_CONTEXTS],
  y_mode_cdf: [[u16; INTRA_MODES + 1]; BLOCK_SIZE_GROUPS],
  uv_mode_cdf: [[[u16; UV_INTRA_MODES + 1]; INTRA_MODES]; 2],
  intra_tx_cdf:
    [[[[u16; TX_TYPES + 1]; INTRA_MODES]; TX_SIZES]; TX_SETS_INTRA],
  inter_tx_cdf: [[[u16; TX_TYPES + 1]; TX_SIZES]; TX_SETS_INTER],
  skip_cdfs: [[u16; 3]; SKIP_CONTEXTS],
  intra_inter_cdfs: [[u16; 3]; INTRA_INTER_CONTEXTS],
  angle_delta_cdf: [[u16; 2 * MAX_ANGLE_DELTA + 1 + 1]; DIRECTIONAL_MODES],
  filter_intra_cdfs: [[u16; 3]; TxSize::TX_SIZES_ALL],

  // lv_map
  txb_skip_cdf: [[[u16; 3]; TXB_SKIP_CONTEXTS]; TxSize::TX_SIZES],
  dc_sign_cdf: [[[u16; 3]; DC_SIGN_CONTEXTS]; PLANE_TYPES],
  eob_extra_cdf:
    [[[[u16; 3]; EOB_COEF_CONTEXTS]; PLANE_TYPES]; TxSize::TX_SIZES],

  eob_flag_cdf16: [[[u16; 5 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf32: [[[u16; 6 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf64: [[[u16; 7 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf128: [[[u16; 8 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf256: [[[u16; 9 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf512: [[[u16; 10 + 1]; 2]; PLANE_TYPES],
  eob_flag_cdf1024: [[[u16; 11 + 1]; 2]; PLANE_TYPES],

  coeff_base_eob_cdf:
    [[[[u16; 3 + 1]; SIG_COEF_CONTEXTS_EOB]; PLANE_TYPES]; TxSize::TX_SIZES],
  coeff_base_cdf:
    [[[[u16; 4 + 1]; SIG_COEF_CONTEXTS]; PLANE_TYPES]; TxSize::TX_SIZES],
  coeff_br_cdf: [[[[u16; BR_CDF_SIZE + 1]; LEVEL_CONTEXTS]; PLANE_TYPES];
    TxSize::TX_SIZES]
}

impl CDFContext {
    pub fn new(quantizer: u8) -> CDFContext {
    let qctx = match quantizer {
      0...20 => 0,
      21...60 => 1,
      61...120 => 2,
      _ => 3
    };
    CDFContext {
      partition_cdf: default_partition_cdf,
      kf_y_cdf: default_kf_y_mode_cdf,
      y_mode_cdf: default_if_y_mode_cdf,
      uv_mode_cdf: default_uv_mode_cdf,
      intra_tx_cdf: default_intra_ext_tx_cdf,
      inter_tx_cdf: default_inter_ext_tx_cdf,
      skip_cdfs: default_skip_cdfs,
      intra_inter_cdfs: default_intra_inter_cdf,
      angle_delta_cdf: default_angle_delta_cdf,
      filter_intra_cdfs: default_filter_intra_cdfs,

      // lv_map
      txb_skip_cdf: av1_default_txb_skip_cdfs[qctx],
      dc_sign_cdf: av1_default_dc_sign_cdfs[qctx],
      eob_extra_cdf: av1_default_eob_extra_cdfs[qctx],

      eob_flag_cdf16: av1_default_eob_multi16_cdfs[qctx],
      eob_flag_cdf32: av1_default_eob_multi32_cdfs[qctx],
      eob_flag_cdf64: av1_default_eob_multi64_cdfs[qctx],
      eob_flag_cdf128: av1_default_eob_multi128_cdfs[qctx],
      eob_flag_cdf256: av1_default_eob_multi256_cdfs[qctx],
      eob_flag_cdf512: av1_default_eob_multi512_cdfs[qctx],
      eob_flag_cdf1024: av1_default_eob_multi1024_cdfs[qctx],

      coeff_base_eob_cdf: av1_default_coeff_base_eob_multi_cdfs[qctx],
      coeff_base_cdf: av1_default_coeff_base_multi_cdfs[qctx],
      coeff_br_cdf: av1_default_coeff_lps_multi_cdfs[qctx]
    }
  }

  pub fn build_map(&self) -> Vec<(&'static str, usize, usize)> {
    use std::mem::size_of_val;

    let partition_cdf_start =
      self.partition_cdf.first().unwrap().as_ptr() as usize;
    let partition_cdf_end =
      partition_cdf_start + size_of_val(&self.partition_cdf);
    let kf_y_cdf_start = self.kf_y_cdf.first().unwrap().as_ptr() as usize;
    let kf_y_cdf_end = kf_y_cdf_start + size_of_val(&self.kf_y_cdf);
    let y_mode_cdf_start = self.y_mode_cdf.first().unwrap().as_ptr() as usize;
    let y_mode_cdf_end = y_mode_cdf_start + size_of_val(&self.y_mode_cdf);
    let uv_mode_cdf_start =
      self.uv_mode_cdf.first().unwrap().as_ptr() as usize;
    let uv_mode_cdf_end = uv_mode_cdf_start + size_of_val(&self.uv_mode_cdf);
    let intra_tx_cdf_start =
      self.intra_tx_cdf.first().unwrap().as_ptr() as usize;
    let intra_tx_cdf_end =
      intra_tx_cdf_start + size_of_val(&self.intra_tx_cdf);
    let inter_tx_cdf_start =
      self.inter_tx_cdf.first().unwrap().as_ptr() as usize;
    let inter_tx_cdf_end =
      inter_tx_cdf_start + size_of_val(&self.inter_tx_cdf);
    let skip_cdfs_start = self.skip_cdfs.first().unwrap().as_ptr() as usize;
    let skip_cdfs_end = skip_cdfs_start + size_of_val(&self.skip_cdfs);
    let intra_inter_cdfs_start =
      self.intra_inter_cdfs.first().unwrap().as_ptr() as usize;
    let intra_inter_cdfs_end =
      intra_inter_cdfs_start + size_of_val(&self.intra_inter_cdfs);
    let angle_delta_cdf_start =
      self.angle_delta_cdf.first().unwrap().as_ptr() as usize;
    let angle_delta_cdf_end =
      angle_delta_cdf_start + size_of_val(&self.angle_delta_cdf);
    let filter_intra_cdfs_start =
      self.filter_intra_cdfs.first().unwrap().as_ptr() as usize;
    let filter_intra_cdfs_end =
      filter_intra_cdfs_start + size_of_val(&self.filter_intra_cdfs);
    let txb_skip_cdf_start =
      self.txb_skip_cdf.first().unwrap().as_ptr() as usize;
    let txb_skip_cdf_end =
      txb_skip_cdf_start + size_of_val(&self.txb_skip_cdf);
    let dc_sign_cdf_start =
      self.dc_sign_cdf.first().unwrap().as_ptr() as usize;
    let dc_sign_cdf_end = dc_sign_cdf_start + size_of_val(&self.dc_sign_cdf);
    let eob_extra_cdf_start =
      self.eob_extra_cdf.first().unwrap().as_ptr() as usize;
    let eob_extra_cdf_end =
      eob_extra_cdf_start + size_of_val(&self.eob_extra_cdf);
    let eob_flag_cdf16_start =
      self.eob_flag_cdf16.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf16_end =
      eob_flag_cdf16_start + size_of_val(&self.eob_flag_cdf16);
    let eob_flag_cdf32_start =
      self.eob_flag_cdf32.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf32_end =
      eob_flag_cdf32_start + size_of_val(&self.eob_flag_cdf32);
    let eob_flag_cdf64_start =
      self.eob_flag_cdf64.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf64_end =
      eob_flag_cdf64_start + size_of_val(&self.eob_flag_cdf64);
    let eob_flag_cdf128_start =
      self.eob_flag_cdf128.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf128_end =
      eob_flag_cdf128_start + size_of_val(&self.eob_flag_cdf128);
    let eob_flag_cdf256_start =
      self.eob_flag_cdf256.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf256_end =
      eob_flag_cdf256_start + size_of_val(&self.eob_flag_cdf256);
    let eob_flag_cdf512_start =
      self.eob_flag_cdf512.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf512_end =
      eob_flag_cdf512_start + size_of_val(&self.eob_flag_cdf512);
    let eob_flag_cdf1024_start =
      self.eob_flag_cdf1024.first().unwrap().as_ptr() as usize;
    let eob_flag_cdf1024_end =
      eob_flag_cdf1024_start + size_of_val(&self.eob_flag_cdf1024);
    let coeff_base_eob_cdf_start =
      self.coeff_base_eob_cdf.first().unwrap().as_ptr() as usize;
    let coeff_base_eob_cdf_end =
      coeff_base_eob_cdf_start + size_of_val(&self.coeff_base_eob_cdf);
    let coeff_base_cdf_start =
      self.coeff_base_cdf.first().unwrap().as_ptr() as usize;
    let coeff_base_cdf_end =
      coeff_base_cdf_start + size_of_val(&self.coeff_base_cdf);
    let coeff_br_cdf_start =
      self.coeff_br_cdf.first().unwrap().as_ptr() as usize;
    let coeff_br_cdf_end =
      coeff_br_cdf_start + size_of_val(&self.coeff_br_cdf);

    vec![
      ("partition_cdf", partition_cdf_start, partition_cdf_end),
      ("kf_y_cdf", kf_y_cdf_start, kf_y_cdf_end),
      ("y_mode_cdf", y_mode_cdf_start, y_mode_cdf_end),
      ("uv_mode_cdf", uv_mode_cdf_start, uv_mode_cdf_end),
      ("intra_tx_cdf", intra_tx_cdf_start, intra_tx_cdf_end),
      ("inter_tx_cdf", inter_tx_cdf_start, inter_tx_cdf_end),
      ("skip_cdfs", skip_cdfs_start, skip_cdfs_end),
      ("intra_inter_cdfs", intra_inter_cdfs_start, intra_inter_cdfs_end),
      ("angle_delta_cdf", angle_delta_cdf_start, angle_delta_cdf_end),
      ("filter_intra_cdfs", filter_intra_cdfs_start, filter_intra_cdfs_end),
      ("txb_skip_cdf", txb_skip_cdf_start, txb_skip_cdf_end),
      ("dc_sign_cdf", dc_sign_cdf_start, dc_sign_cdf_end),
      ("eob_extra_cdf", eob_extra_cdf_start, eob_extra_cdf_end),
      ("eob_flag_cdf16", eob_flag_cdf16_start, eob_flag_cdf16_end),
      ("eob_flag_cdf32", eob_flag_cdf32_start, eob_flag_cdf32_end),
      ("eob_flag_cdf64", eob_flag_cdf64_start, eob_flag_cdf64_end),
      ("eob_flag_cdf128", eob_flag_cdf128_start, eob_flag_cdf128_end),
      ("eob_flag_cdf256", eob_flag_cdf256_start, eob_flag_cdf256_end),
      ("eob_flag_cdf512", eob_flag_cdf512_start, eob_flag_cdf512_end),
      ("eob_flag_cdf1024", eob_flag_cdf1024_start, eob_flag_cdf1024_end),
      ("coeff_base_eob_cdf", coeff_base_eob_cdf_start, coeff_base_eob_cdf_end),
      ("coeff_base_cdf", coeff_base_cdf_start, coeff_base_cdf_end),
      ("coeff_br_cdf", coeff_br_cdf_start, coeff_br_cdf_end),
    ]
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn cdf_map() {
    use super::*;

    let cdf = CDFContext::new(8);
    let cdf_map = FieldMap {
      map: cdf.build_map()
    };
    let f = &cdf.partition_cdf[2];
    cdf_map.lookup(f.as_ptr() as usize);
  }
}

const SUPERBLOCK_TO_PLANE_SHIFT: usize = MAX_SB_SIZE_LOG2;
const SUPERBLOCK_TO_BLOCK_SHIFT: usize = MAX_MIB_SIZE_LOG2;
const BLOCK_TO_PLANE_SHIFT: usize = MI_SIZE_LOG2;
pub const LOCAL_BLOCK_MASK: usize = (1 << SUPERBLOCK_TO_BLOCK_SHIFT) - 1;

/// Absolute offset in superblocks inside a plane, where a superblock is defined
/// to be an N*N square where N = (1 << SUPERBLOCK_TO_PLANE_SHIFT).
#[derive(Clone)]
pub struct SuperBlockOffset {
  pub x: usize,
  pub y: usize
}

impl SuperBlockOffset {
  /// Offset of a block inside the current superblock.
  pub fn block_offset(&self, block_x: usize, block_y: usize) -> BlockOffset {
    BlockOffset {
      x: (self.x << SUPERBLOCK_TO_BLOCK_SHIFT) + block_x,
      y: (self.y << SUPERBLOCK_TO_BLOCK_SHIFT) + block_y
    }
  }

  /// Offset of the top-left pixel of this block.
  pub fn plane_offset(&self, plane: &PlaneConfig) -> PlaneOffset {
    PlaneOffset {
      x: self.x << (SUPERBLOCK_TO_PLANE_SHIFT - plane.xdec),
      y: self.y << (SUPERBLOCK_TO_PLANE_SHIFT - plane.ydec)
    }
  }
}

/// Absolute offset in blocks inside a plane, where a block is defined
/// to be an N*N square where N = (1 << BLOCK_TO_PLANE_SHIFT).
#[derive(Clone)]
pub struct BlockOffset {
  pub x: usize,
  pub y: usize
}

impl BlockOffset {
  /// Offset of the superblock in which this block is located.
  pub fn sb_offset(&self) -> SuperBlockOffset {
    SuperBlockOffset {
      x: self.x >> SUPERBLOCK_TO_BLOCK_SHIFT,
      y: self.y >> SUPERBLOCK_TO_BLOCK_SHIFT
    }
  }

  /// Offset of the top-left pixel of this block.
  pub fn plane_offset(&self, plane: &PlaneConfig) -> PlaneOffset {
    let po = self.sb_offset().plane_offset(plane);
    let x_offset = self.x & LOCAL_BLOCK_MASK;
    let y_offset = self.y & LOCAL_BLOCK_MASK;
    PlaneOffset {
      x: po.x + (x_offset << BLOCK_TO_PLANE_SHIFT),
      y: po.y + (y_offset << BLOCK_TO_PLANE_SHIFT)
    }
  }

  pub fn y_in_sb(&self) -> usize {
    self.y % MAX_MIB_SIZE
  }
}

#[derive(Copy, Clone)]
pub struct Block {
  pub mode: PredictionMode,
  pub bsize: BlockSize,
  pub partition: PartitionType,
  pub skip: bool
}

impl Block {
  pub fn default() -> Block {
    Block {
      mode: PredictionMode::DC_PRED,
      bsize: BlockSize::BLOCK_64X64,
      partition: PartitionType::PARTITION_NONE,
      skip: false
    }
  }
  pub fn is_inter(&self) -> bool {
    self.mode >= PredictionMode::NEARESTMV
  }
}

pub struct TXB_CTX {
  pub txb_skip_ctx: usize,
  pub dc_sign_ctx: usize
}

#[derive(Clone, Default)]
pub struct BlockContext {
  pub cols: usize,
  pub rows: usize,
  above_partition_context: Vec<u8>,
  left_partition_context: [u8; MAX_MIB_SIZE],
  above_coeff_context: [Vec<u8>; PLANES],
  left_coeff_context: [[u8; MAX_MIB_SIZE]; PLANES],
  blocks: Vec<Vec<Block>>
}

impl BlockContext {
  pub fn new(cols: usize, rows: usize) -> BlockContext {
    // Align power of two
    let aligned_cols = (cols + ((1 << MAX_MIB_SIZE_LOG2) - 1))
      & !((1 << MAX_MIB_SIZE_LOG2) - 1);
    BlockContext {
      cols,
      rows,
      above_partition_context: vec![0; aligned_cols],
      left_partition_context: [0; MAX_MIB_SIZE],
      above_coeff_context: [
        vec![0; cols << (MI_SIZE_LOG2 - TxSize::smallest_width_log2())],
        vec![0; cols << (MI_SIZE_LOG2 - TxSize::smallest_width_log2())],
        vec![0; cols << (MI_SIZE_LOG2 - TxSize::smallest_width_log2())]
      ],
      left_coeff_context: [[0; MAX_MIB_SIZE]; PLANES],
      blocks: vec![vec![Block::default(); cols]; rows]
    }
  }

  pub fn checkpoint(&mut self) -> BlockContext {
    BlockContext {
      cols: self.cols,
      rows: self.rows,
      above_partition_context: self.above_partition_context.clone(),
      left_partition_context: self.left_partition_context,
      above_coeff_context: self.above_coeff_context.clone(),
      left_coeff_context: self.left_coeff_context,
      blocks: vec![vec![Block::default(); 0]; 0]
    }
  }

  pub fn rollback(&mut self, checkpoint: &BlockContext) {
    self.cols = checkpoint.cols;
    self.rows = checkpoint.rows;
    self.above_partition_context = checkpoint.above_partition_context.clone();
    self.left_partition_context = checkpoint.left_partition_context;
    self.above_coeff_context = checkpoint.above_coeff_context.clone();
    self.left_coeff_context = checkpoint.left_coeff_context;
  }

  pub fn at(&mut self, bo: &BlockOffset) -> &mut Block {
    &mut self.blocks[bo.y][bo.x]
  }

  pub fn above_of(&mut self, bo: &BlockOffset) -> Block {
    if bo.y > 0 {
      self.blocks[bo.y - 1][bo.x]
    } else {
      Block::default()
    }
  }

  pub fn left_of(&mut self, bo: &BlockOffset) -> Block {
    if bo.x > 0 {
      self.blocks[bo.y][bo.x - 1]
    } else {
      Block::default()
    }
  }

  pub fn set_dc_sign(&mut self, cul_level: &mut u32, dc_val: i32) {
    if dc_val < 0 {
      *cul_level |= 1 << COEFF_CONTEXT_BITS;
    } else if dc_val > 0 {
      *cul_level += 2 << COEFF_CONTEXT_BITS;
    }
  }

  fn set_coeff_context(
    &mut self, plane: usize, bo: &BlockOffset, tx_size: TxSize, xdec: usize,
    ydec: usize, value: u8
  ) {
    // for subsampled planes, coeff contexts are stored sparsely at the moment
    // so we need to scale our fill by xdec and ydec
    for bx in 0..tx_size.width_mi() {
      self.above_coeff_context[plane][bo.x + (bx << xdec)] = value;
    }
    let bo_y = bo.y_in_sb();
    for by in 0..tx_size.height_mi() {
      self.left_coeff_context[plane][bo_y + (by << ydec)] = value;
    }
  }

  fn reset_left_coeff_context(&mut self, plane: usize) {
    for c in &mut self.left_coeff_context[plane] {
      *c = 0;
    }
  }

  fn reset_left_partition_context(&mut self) {
    for c in &mut self.left_partition_context {
      *c = 0;
    }
  }
  //TODO(anyone): Add reset_left_tx_context() here then call it in reset_left_contexts()

  pub fn reset_skip_context(
    &mut self, bo: &BlockOffset, bsize: BlockSize, xdec: usize, ydec: usize
  ) {
    const num_planes: usize = 3;
    let nplanes = if bsize >= BLOCK_8X8 {
      3
    } else {
      1 + (num_planes - 1) * has_chroma(bo, bsize, xdec, ydec) as usize
    };

    for plane in 0..nplanes {
      let xdec2 = if plane == 0 {
        0
      } else {
        xdec
      };
      let ydec2 = if plane == 0 {
        0
      } else {
        ydec
      };

      let plane_bsize = if plane == 0 {
        bsize
      } else {
        get_plane_block_size(bsize, xdec2, ydec2)
      };
      let bw = plane_bsize.width_mi();
      let bh = plane_bsize.height_mi();

      for bx in 0..bw {
        self.above_coeff_context[plane][bo.x + (bx << xdec2) as usize] = 0;
      }

      let bo_y = bo.y_in_sb();
      for by in 0..bh {
        self.left_coeff_context[plane][bo_y + (by << ydec2) as usize] = 0;
      }
    }
  }

  pub fn reset_left_contexts(&mut self) {
    for p in 0..3 {
      BlockContext::reset_left_coeff_context(self, p);
    }
    BlockContext::reset_left_partition_context(self);

    //TODO(anyone): Call reset_left_tx_context() here.
  }

  pub fn set_mode(
    &mut self, bo: &BlockOffset, bsize: BlockSize, mode: PredictionMode
  ) {
    let bw = bsize.width_mi();
    let bh = bsize.height_mi();

    for y in 0..bh {
      for x in 0..bw {
        self.blocks[bo.y + y as usize][bo.x + x as usize].mode = mode;
      }
    }
  }

  pub fn get_mode(&mut self, bo: &BlockOffset) -> PredictionMode {
    self.blocks[bo.y][bo.x].mode
  }

  fn partition_plane_context(
    &self, bo: &BlockOffset, bsize: BlockSize
  ) -> usize {
    // TODO: this should be way simpler without sub8x8
    let above_ctx = self.above_partition_context[bo.x];
    let left_ctx = self.left_partition_context[bo.y_in_sb()];
    let bsl = bsize.width_log2() - BLOCK_8X8.width_log2();
    let above = (above_ctx >> bsl) & 1;
    let left = (left_ctx >> bsl) & 1;

    assert!(bsize.is_sqr());

    (left * 2 + above) as usize + bsl as usize * PARTITION_PLOFFSET
  }

  pub fn update_partition_context(
    &mut self, bo: &BlockOffset, subsize: BlockSize, bsize: BlockSize
  ) {
    #[allow(dead_code)]
    let bw = bsize.width_mi();
    let bh = bsize.height_mi();

    let above_ctx =
      &mut self.above_partition_context[bo.x..bo.x + bw as usize];
    let left_ctx = &mut self.left_partition_context
      [bo.y_in_sb()..bo.y_in_sb() + bh as usize];

    // update the partition context at the end notes. set partition bits
    // of block sizes larger than the current one to be one, and partition
    // bits of smaller block sizes to be zero.
    for i in 0..bw {
      above_ctx[i as usize] = partition_context_lookup[subsize as usize][0];
    }

    for i in 0..bh {
      left_ctx[i as usize] = partition_context_lookup[subsize as usize][1];
    }
  }

  fn skip_context(&mut self, bo: &BlockOffset) -> usize {
    let above_skip = if bo.y > 0 {
      self.above_of(bo).skip as usize
    } else {
      0
    };
    let left_skip = if bo.x > 0 {
      self.left_of(bo).skip as usize
    } else {
      0
    };
    above_skip + left_skip
  }

  pub fn set_skip(&mut self, bo: &BlockOffset, bsize: BlockSize, skip: bool) {
    let bw = bsize.width_mi();
    let bh = bsize.height_mi();

    for y in 0..bh {
      for x in 0..bw {
        self.blocks[bo.y + y as usize][bo.x + x as usize].skip = skip;
      }
    }
  }

  // The mode info data structure has a one element border above and to the
  // left of the entries corresponding to real macroblocks.
  // The prediction flags in these dummy entries are initialized to 0.
  // 0 - inter/inter, inter/--, --/inter, --/--
  // 1 - intra/inter, inter/intra
  // 2 - intra/--, --/intra
  // 3 - intra/intra
  pub fn intra_inter_context(&mut self, bo: &BlockOffset) -> usize {
    let has_above = bo.y > 0;
    let has_left = bo.x > 0;

    match (has_above, has_left) {
      (true, true) => {
        let above_intra = !self.above_of(bo).is_inter();
        let left_intra = !self.left_of(bo).is_inter();
        if above_intra && left_intra {
          3
        } else {
          (above_intra || left_intra) as usize
        }
      }
      (true, _) | (_, true) =>
        2 * if has_above {
          !self.above_of(bo).is_inter() as usize
        } else {
          !self.left_of(bo).is_inter() as usize
        },
      (_, _) => 0
    }
  }

  pub fn get_txb_ctx(
    &mut self, plane_bsize: BlockSize, tx_size: TxSize, plane: usize,
    bo: &BlockOffset, xdec: usize, ydec: usize
  ) -> TXB_CTX {
    let mut txb_ctx = TXB_CTX {
      txb_skip_ctx: 0,
      dc_sign_ctx: 0
    };
    const MAX_TX_SIZE_UNIT: usize = 16;
    const signs: [i8; 3] = [0, -1, 1];
    const dc_sign_contexts: [usize; 4 * MAX_TX_SIZE_UNIT + 1] = [
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1, 1, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
      2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    ];
    let mut dc_sign: i16 = 0;
    let txb_w_unit = tx_size.width_mi();
    let txb_h_unit = tx_size.height_mi();

    // Decide txb_ctx.dc_sign_ctx
    for k in 0..txb_w_unit {
      let sign = self.above_coeff_context[plane][bo.x + (k << xdec)]
        >> COEFF_CONTEXT_BITS;
      assert!(sign <= 2);
      dc_sign += signs[sign as usize] as i16;
    }

    for k in 0..txb_h_unit {
      let sign = self.left_coeff_context[plane][bo.y_in_sb() + (k << ydec)]
        >> COEFF_CONTEXT_BITS;
      assert!(sign <= 2);
      dc_sign += signs[sign as usize] as i16;
    }

    txb_ctx.dc_sign_ctx =
      dc_sign_contexts[(dc_sign + 2 * MAX_TX_SIZE_UNIT as i16) as usize];

    // Decide txb_ctx.txb_skip_ctx
    if plane == 0 {
      if plane_bsize == tx_size.block_size() {
        txb_ctx.txb_skip_ctx = 0;
      } else {
        // This is the algorithm to generate table skip_contexts[min][max].
        //    if (!max)
        //      txb_skip_ctx = 1;
        //    else if (!min)
        //      txb_skip_ctx = 2 + (max > 3);
        //    else if (max <= 3)
        //      txb_skip_ctx = 4;
        //    else if (min <= 3)
        //      txb_skip_ctx = 5;
        //    else
        //      txb_skip_ctx = 6;
        const skip_contexts: [[u8; 5]; 5] = [
          [1, 2, 2, 2, 3],
          [1, 4, 4, 4, 5],
          [1, 4, 4, 4, 5],
          [1, 4, 4, 4, 5],
          [1, 4, 4, 4, 6]
        ];
        let mut top: u8 = 0;
        let mut left: u8 = 0;

        for k in 0..txb_w_unit {
          top |= self.above_coeff_context[0][bo.x + k];
        }
        top &= COEFF_CONTEXT_MASK as u8;

        for k in 0..txb_h_unit {
          left |= self.left_coeff_context[0][bo.y_in_sb() + k];
        }
        left &= COEFF_CONTEXT_MASK as u8;

        let max = cmp::min(top | left, 4);
        let min = cmp::min(cmp::min(top, left), 4);
        txb_ctx.txb_skip_ctx =
          skip_contexts[min as usize][max as usize] as usize;
      }
    } else {
      let mut top: u8 = 0;
      let mut left: u8 = 0;

      for k in 0..txb_w_unit {
        top |= self.above_coeff_context[plane][bo.x + (k << xdec)];
      }
      for k in 0..txb_h_unit {
        left |= self.left_coeff_context[plane][bo.y_in_sb() + (k << ydec)];
      }
      let ctx_base = (top != 0) as usize + (left != 0) as usize;
      let ctx_offset = if num_pels_log2_lookup[plane_bsize as usize]
        > num_pels_log2_lookup[tx_size.block_size() as usize]
      {
        10
      } else {
        7
      };
      txb_ctx.txb_skip_ctx = ctx_base + ctx_offset;
    }

    txb_ctx
  }
}

#[derive(Debug, Default)]
struct FieldMap {
  map: Vec<(&'static str, usize, usize)>
}

impl FieldMap {
  /// Print the field the address belong to
  fn lookup(&self, addr: usize) {
    for (name, start, end) in &self.map {
      // eprintln!("{} {} {} val {}", name, start, end, addr);
      if addr >= *start && addr < *end {
        eprintln!(" CDF {}", name);
        eprintln!("");
        return;
      }
    }

    eprintln!("  CDF address not found {}", addr);
  }
}

macro_rules! symbol {
  ($self:ident, $s:expr, $cdf:expr) => {
    $self.w.symbol($s, $cdf);
    #[cfg(debug)] {
      if let Some(map) = $self.fc_map.as_ref() {
        map.lookup($cdf.as_ptr() as usize);
      }
    }
  };
}

#[derive(Clone)]
pub struct ContextWriterCheckpoint {
  pub w: ec::WriterCheckpoint,
  pub fc: CDFContext,
  pub bc: BlockContext
}

pub struct ContextWriter {
  pub w: ec::Writer,
  pub bc: BlockContext,
  fc: CDFContext,
  #[cfg(debug)]
  fc_map: Option<FieldMap> // For debugging purposes
}

impl ContextWriter {
  pub fn new(w: ec::Writer, fc: CDFContext, bc: BlockContext) -> Self {
    #[allow(unused_mut)]
    let mut cw = ContextWriter {
      w,
      fc,
      bc,
      #[cfg(debug)]
      fc_map: Default::default()
    };
    #[cfg(debug)] {
      if std::env::var_os("RAV1E_DEBUG").is_some() {
        cw.fc_map = Some(FieldMap {
          map: cw.fc.build_map()
        });
      }
    }

    cw
  }

  fn cdf_element_prob(cdf: &[u16], element: usize) -> u16 {
    (if element > 0 {
      cdf[element - 1]
    } else {
      32768
    }) - cdf[element]
  }

  fn partition_gather_horz_alike(
    out: &mut [u16; 2], cdf_in: &[u16], _bsize: BlockSize
  ) {
    out[0] = 32768;
    out[0] -= ContextWriter::cdf_element_prob(
      cdf_in,
      PartitionType::PARTITION_HORZ as usize
    );
    out[0] -= ContextWriter::cdf_element_prob(
      cdf_in,
      PartitionType::PARTITION_SPLIT as usize
    );
    out[0] = 32768 - out[0];
    out[1] = 0;
  }

  fn partition_gather_vert_alike(
    out: &mut [u16; 2], cdf_in: &[u16], _bsize: BlockSize
  ) {
    out[0] = 32768;
    out[0] -= ContextWriter::cdf_element_prob(
      cdf_in,
      PartitionType::PARTITION_VERT as usize
    );
    out[0] -= ContextWriter::cdf_element_prob(
      cdf_in,
      PartitionType::PARTITION_SPLIT as usize
    );
    out[0] = 32768 - out[0];
    out[1] = 0;
  }

  pub fn write_partition(
    &mut self, bo: &BlockOffset, p: PartitionType, bsize: BlockSize
  ) {
    let hbs = bsize.width_mi() / 2;
    let has_cols = (bo.x + hbs) < self.bc.cols;
    let has_rows = (bo.y + hbs) < self.bc.rows;
    let ctx = self.bc.partition_plane_context(&bo, bsize);
    assert!(ctx < PARTITION_CONTEXTS);
    let partition_cdf = &mut self.fc.partition_cdf[ctx];

    if !has_rows && !has_cols {
      return;
    }

    if has_rows && has_cols {
      symbol!(self, p as u32, partition_cdf);
    } else if !has_rows && has_cols {
      let mut cdf = [0u16; 2];
      ContextWriter::partition_gather_vert_alike(
        &mut cdf,
        partition_cdf,
        bsize
      );
      self.w.cdf((p == PartitionType::PARTITION_SPLIT) as u32, &cdf);
    } else {
      let mut cdf = [0u16; 2];
      ContextWriter::partition_gather_horz_alike(
        &mut cdf,
        partition_cdf,
        bsize
      );
      self.w.cdf((p == PartitionType::PARTITION_SPLIT) as u32, &cdf);
    }
  }
  pub fn write_intra_mode_kf(
    &mut self, bo: &BlockOffset, mode: PredictionMode
  ) {
    static intra_mode_context: [usize; INTRA_MODES] =
      [0, 1, 2, 3, 4, 4, 4, 4, 3, 0, 1, 2, 0];
    let above_mode = self.bc.above_of(bo).mode as usize;
    let left_mode = self.bc.left_of(bo).mode as usize;
    let above_ctx = intra_mode_context[above_mode];
    let left_ctx = intra_mode_context[left_mode];
    let cdf = &mut self.fc.kf_y_cdf[above_ctx][left_ctx];
    symbol!(self, mode as u32, cdf);
  }
  pub fn write_intra_mode(&mut self, bsize: BlockSize, mode: PredictionMode) {
    let cdf =
      &mut self.fc.y_mode_cdf[size_group_lookup[bsize as usize] as usize];
    symbol!(self, mode as u32, cdf);
  }
  pub fn write_intra_uv_mode(
    &mut self, uv_mode: PredictionMode, y_mode: PredictionMode, bs: BlockSize
  ) {
    let cdf =
      &mut self.fc.uv_mode_cdf[bs.cfl_allowed() as usize][y_mode as usize];
    if bs.cfl_allowed() {
      symbol!(self, uv_mode as u32, cdf);
    } else {
      symbol!(self, uv_mode as u32, &mut cdf[..UV_INTRA_MODES]);
    }
  }
  pub fn write_angle_delta(&mut self, angle: i8, mode: PredictionMode) {
    symbol!(
      self,
      (angle + MAX_ANGLE_DELTA as i8) as u32,
      &mut self.fc.angle_delta_cdf
        [mode as usize - PredictionMode::V_PRED as usize]
    );
  }
  pub fn write_use_filter_intra(&mut self, enable: bool, tx_size: TxSize) {
    symbol!(self, enable as u32, &mut self.fc.filter_intra_cdfs[tx_size as usize]);
  }

  pub fn write_tx_type(
    &mut self, tx_size: TxSize, tx_type: TxType, y_mode: PredictionMode,
    is_inter: bool, use_reduced_tx_set: bool
  ) {
    let square_tx_size = tx_size.sqr();
    let tx_set =
      get_tx_set(tx_size, is_inter, use_reduced_tx_set);
    let num_tx_types = num_tx_set[tx_set as usize];

    if num_tx_types > 1 {
      let tx_set_index = get_tx_set_index(tx_size, is_inter, use_reduced_tx_set);
      assert!(tx_set_index > 0);
      assert!(av1_tx_used[tx_set as usize][tx_type as usize] != 0);

      if is_inter {
        symbol!(
          self,
          av1_tx_ind[tx_set as usize][tx_type as usize] as u32,
          &mut self.fc.inter_tx_cdf[tx_set_index as usize]
            [square_tx_size as usize]
            [..num_tx_set[tx_set as usize] + 1]
        );
      } else {
        let intra_dir = y_mode;
        // TODO: Once use_filter_intra is enabled,
        // intra_dir =
        // fimode_to_intradir[mbmi->filter_intra_mode_info.filter_intra_mode];

        symbol!(
          self,
          av1_tx_ind[tx_set as usize][tx_type as usize] as u32,
          &mut self.fc.intra_tx_cdf[tx_set_index as usize]
            [square_tx_size as usize][intra_dir as usize]
            [..num_tx_set[tx_set as usize] + 1]
        );
      }
    }
  }
  pub fn write_skip(&mut self, bo: &BlockOffset, skip: bool) {
    let ctx = self.bc.skip_context(bo);
    symbol!(self, skip as u32, &mut self.fc.skip_cdfs[ctx]);
  }
  pub fn write_is_inter(&mut self, bo: &BlockOffset, is_inter: bool) {
    let ctx = self.bc.intra_inter_context(bo);
    symbol!(self, is_inter as u32, &mut self.fc.intra_inter_cdfs[ctx]);
  }

  pub fn get_txsize_entropy_ctx(&mut self, tx_size: TxSize) -> usize {
    (tx_size.sqr() as usize + tx_size.sqr() as usize + 1) >> 1
  }

  pub fn txb_init_levels(
    &mut self, coeffs: &[i32], width: usize, height: usize,
    levels_buf: &mut [u8]
  ) {
    let mut offset = TX_PAD_TOP * (width + TX_PAD_HOR);

    for y in 0..height {
      for x in 0..width {
        levels_buf[offset] = clamp(coeffs[y * width + x].abs(), 0, 127) as u8;
        offset += 1;
      }
      offset += TX_PAD_HOR;
    }
  }

  pub fn av1_get_adjusted_tx_size(&mut self, tx_size: TxSize) -> TxSize {
    // TODO: Enable below commented out block if TX64X64 is enabled.
/*
      if tx_size == TX_64X64 || tx_size == TX_64X32 || tx_size == TX_32X64 {
        return TX_32X32
      }
      if (tx_size == TX_16X64) {
        return TX_16X32
      }
      if (tx_size == TX_64X16) {
        return TX_32X16
      }
*/
    tx_size
  }

  pub fn get_txb_bwl(&mut self, tx_size: TxSize) -> usize {
    self.av1_get_adjusted_tx_size(tx_size).width_log2()
  }

  pub fn get_eob_pos_token(&mut self, eob: usize, extra: &mut u32) -> u32 {
    let t = if eob < 33 {
      eob_to_pos_small[eob] as u32
    } else {
      let e = cmp::min((eob - 1) >> 5, 16);
      eob_to_pos_large[e as usize] as u32
    };
    assert!(eob as i32 >= k_eob_group_start[t as usize] as i32);
    *extra = eob as u32 - k_eob_group_start[t as usize] as u32;

    t
  }

  pub fn get_nz_mag(
    &mut self, levels: &[u8], bwl: usize, tx_class: TxClass
  ) -> usize {
    // May version.
    // Note: AOMMIN(level, 3) is useless for decoder since level < 3.
    let mut mag = clip_max3(levels[1]); // { 0, 1 }
    mag += clip_max3(levels[(1 << bwl) + TX_PAD_HOR]); // { 1, 0 }

    if tx_class == TX_CLASS_2D {
      mag += clip_max3(levels[(1 << bwl) + TX_PAD_HOR + 1]); // { 1, 1 }
      mag += clip_max3(levels[2]); // { 0, 2 }
      mag += clip_max3(levels[(2 << bwl) + (2 << TX_PAD_HOR_LOG2)]); // { 2, 0 }
    } else if tx_class == TX_CLASS_VERT {
      mag += clip_max3(levels[(2 << bwl) + (2 << TX_PAD_HOR_LOG2)]); // { 2, 0 }
      mag += clip_max3(levels[(3 << bwl) + (3 << TX_PAD_HOR_LOG2)]); // { 3, 0 }
      mag += clip_max3(levels[(4 << bwl) + (4 << TX_PAD_HOR_LOG2)]); // { 4, 0 }
    } else {
      mag += clip_max3(levels[2]); // { 0, 2 }
      mag += clip_max3(levels[3]); // { 0, 3 }
      mag += clip_max3(levels[4]); // { 0, 4 }
    }

    mag as usize
  }

  pub fn get_nz_map_ctx_from_stats(
    &mut self,
    stats: usize,
    coeff_idx: usize, // raster order
    bwl: usize,
    tx_size: TxSize,
    tx_class: TxClass
  ) -> usize {
    if (tx_class as u32 | coeff_idx as u32) == 0 {
      return 0;
    };
    let row = coeff_idx >> bwl;
    let col = coeff_idx - (row << bwl);
    let mut ctx = (stats + 1) >> 1;
    ctx = cmp::min(ctx, 4);

    match tx_class {
      TX_CLASS_2D => {
        // This is the algorithm to generate table av1_nz_map_ctx_offset[].
        // const int width = tx_size_wide[tx_size];
        // const int height = tx_size_high[tx_size];
        // if (width < height) {
        //   if (row < 2) return 11 + ctx;
        // } else if (width > height) {
        //   if (col < 2) return 16 + ctx;
        // }
        // if (row + col < 2) return ctx + 1;
        // if (row + col < 4) return 5 + ctx + 1;
        // return 21 + ctx;
        return ctx + av1_nz_map_ctx_offset[tx_size as usize][cmp::min(row, 4)][cmp::min(col, 4)] as usize;
      }
      TX_CLASS_HORIZ => {
        let row = coeff_idx >> bwl;
        let col = coeff_idx - (row << bwl);
        ctx + nz_map_ctx_offset_1d[col as usize]
      }
      TX_CLASS_VERT => {
        let row = coeff_idx >> bwl;
        ctx + nz_map_ctx_offset_1d[row]
      }
    }
  }

  pub fn get_nz_map_ctx(
    &mut self, levels: &[u8], coeff_idx: usize, bwl: usize, height: usize,
    scan_idx: usize, is_eob: bool, tx_size: TxSize, tx_class: TxClass
  ) -> usize {
    if is_eob {
      if scan_idx == 0 {
        return 0;
      }
      if scan_idx <= (height << bwl) / 8 {
        return 1;
      }
      if scan_idx <= (height << bwl) / 4 {
        return 2;
      }
      return 3;
    }
    let padded_idx = coeff_idx + ((coeff_idx >> bwl) << TX_PAD_HOR_LOG2);
    let stats = self.get_nz_mag(&levels[padded_idx..], bwl, tx_class);

    self.get_nz_map_ctx_from_stats(stats, coeff_idx, bwl, tx_size, tx_class)
  }

  pub fn get_nz_map_contexts(
    &mut self, levels: &mut [u8], scan: &[u16; 4096], eob: u16,
    tx_size: TxSize, tx_class: TxClass, coeff_contexts: &mut [i8]
  ) {
    // TODO: If TX_64X64 is enabled, use av1_get_adjusted_tx_size()
    let bwl = tx_size.width_log2();
    let height = tx_size.height();
    for i in 0..eob {
      let pos = scan[i as usize];
      coeff_contexts[pos as usize] = self.get_nz_map_ctx(
        levels,
        pos as usize,
        bwl,
        height,
        i as usize,
        i == eob - 1,
        tx_size,
        tx_class
      ) as i8;
    }
  }

  pub fn get_br_ctx(
    &mut self,
    levels: &[u8],
    c: usize, // raster order
    bwl: usize,
    tx_class: TxClass
  ) -> usize {
    let row: usize = c >> bwl;
    let col: usize = c - (row << bwl);
    let stride: usize = (1 << bwl) + TX_PAD_HOR;
    let pos: usize = row * stride + col;
    let mut mag: usize = levels[pos + 1] as usize;

    mag += levels[pos + stride] as usize;

    match tx_class {
      TX_CLASS_2D => {
        mag += levels[pos + stride + 1] as usize;
        mag = cmp::min((mag + 1) >> 1, 6);
        if c == 0 {
          return mag;
        }
        if (row < 2) && (col < 2) {
          return mag + 7;
        }
      }
      TX_CLASS_HORIZ => {
        mag += levels[pos + 2] as usize;
        mag = cmp::min((mag + 1) >> 1, 6);
        if c == 0 {
          return mag;
        }
        if col == 0 {
          return mag + 7;
        }
      }
      TX_CLASS_VERT => {
        mag += levels[pos + (stride << 1)] as usize;
        mag = cmp::min((mag + 1) >> 1, 6);
        if c == 0 {
          return mag;
        }
        if row == 0 {
          return mag + 7;
        }
      }
    }

    mag + 14
  }

  pub fn get_level_mag_with_txclass(
    &mut self, levels: &[u8], stride: usize, row: usize, col: usize,
    mag: &mut [usize], tx_class: TxClass
  ) {
    for idx in 0..CONTEXT_MAG_POSITION_NUM {
      let ref_row =
        row + mag_ref_offset_with_txclass[tx_class as usize][idx][0];
      let ref_col =
        col + mag_ref_offset_with_txclass[tx_class as usize][idx][1];
      let pos = ref_row * stride + ref_col;
      mag[idx] = levels[pos] as usize;
    }
  }

  pub fn write_coeffs_lv_map(
    &mut self, plane: usize, bo: &BlockOffset, coeffs_in: &[i32],
    tx_size: TxSize, tx_type: TxType, plane_bsize: BlockSize, xdec: usize,
    ydec: usize, use_reduced_tx_set: bool
  ) {
    let pred_mode = self.bc.get_mode(bo);
    let is_inter = pred_mode >= PredictionMode::NEARESTMV;
    assert!(!is_inter);
    // TODO: If iner mode, scan_order should use inter version of them
    let scan_order =
      &av1_inter_scan_orders[tx_size as usize][tx_type as usize];
    let scan = scan_order.scan;
    let mut coeffs_storage = [0 as i32; 32 * 32];
    let coeffs = &mut coeffs_storage[..tx_size.area()];
    let mut cul_level = 0 as u32;

    for i in 0..tx_size.area() {
      coeffs[i] = coeffs_in[scan[i] as usize];
      cul_level += coeffs[i].abs() as u32;
    }

    let mut eob = 0;

    if cul_level != 0 {
      for (i, v) in coeffs.iter().enumerate() {
        if *v != 0 {
          eob = i + 1;
        }
      }
    }

    let txs_ctx = self.get_txsize_entropy_ctx(tx_size);
    let txb_ctx =
      self.bc.get_txb_ctx(plane_bsize, tx_size, plane, bo, xdec, ydec);

    {
      let cdf = &mut self.fc.txb_skip_cdf[txs_ctx][txb_ctx.txb_skip_ctx];
      symbol!(self, (eob == 0) as u32, cdf);
    }

    if eob == 0 {
      self.bc.set_coeff_context(plane, bo, tx_size, xdec, ydec, 0);
      return;
    }

    let mut levels_buf = [0 as u8; TX_PAD_2D];

    self.txb_init_levels(
      coeffs_in,
      tx_size.width(),
      tx_size.height(),
      &mut levels_buf
    );

    let tx_class = tx_type_to_class[tx_type as usize];
    let plane_type = if plane == 0 {
      0
    } else {
      1
    } as usize;

    // Signal tx_type for luma plane only
    if plane == 0 {
      self.write_tx_type(
        tx_size,
        tx_type,
        pred_mode,
        is_inter,
        use_reduced_tx_set
      );
    }

    // Encode EOB
    let mut eob_extra = 0 as u32;
    let eob_pt = self.get_eob_pos_token(eob, &mut eob_extra);
    let eob_multi_size: usize = tx_size.area_log2() - 4;
    let eob_multi_ctx: usize = if tx_class == TX_CLASS_2D {
      0
    } else {
      1
    };

    match eob_multi_size {
      0 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf16[plane_type][eob_multi_ctx]
        );
      }
      1 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf32[plane_type][eob_multi_ctx]
        );
      }
      2 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf64[plane_type][eob_multi_ctx]
        );
      }
      3 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf128[plane_type][eob_multi_ctx]
        );
      }
      4 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf256[plane_type][eob_multi_ctx]
        );
      }
      5 => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf512[plane_type][eob_multi_ctx]
        );
      }
      _ => {
        symbol!(
          self,
          eob_pt - 1,
          &mut self.fc.eob_flag_cdf1024[plane_type][eob_multi_ctx]
        );
      }
    };

    let eob_offset_bits = k_eob_offset_bits[eob_pt as usize];

    if eob_offset_bits > 0 {
      let mut eob_shift = eob_offset_bits - 1;
      let mut bit = if (eob_extra & (1 << eob_shift)) != 0 {
        1
      } else {
        0
      } as u32;
      symbol!(
        self,
        bit,
        &mut self.fc.eob_extra_cdf[txs_ctx][plane_type][eob_pt as usize]
      );
      for i in 1..eob_offset_bits {
        eob_shift = eob_offset_bits as u16 - 1 - i as u16;
        bit = if (eob_extra & (1 << eob_shift)) != 0 {
          1
        } else {
          0
        };
        self.w.bit(bit as u16);
      }
    }

    let mut coeff_contexts = [0 as i8; MAX_TX_SQUARE];
    let levels =
      &mut levels_buf[TX_PAD_TOP * (tx_size.width() + TX_PAD_HOR)..];

    self.get_nz_map_contexts(
      levels,
      scan,
      eob as u16,
      tx_size,
      tx_class,
      &mut coeff_contexts
    );

    let bwl = self.get_txb_bwl(tx_size);

    let mut update_pos = [0; MAX_TX_SQUARE];
    let mut num_updates = 0;

    for c in (0..eob).rev() {
      let pos = scan[c];
      let coeff_ctx = coeff_contexts[pos as usize];
      let v = coeffs_in[pos as usize];
      let level: u32 = v.abs() as u32;

      if c == eob - 1 {
        symbol!(
          self,
          (cmp::min(level, 3) - 1) as u32,
          &mut self.fc.coeff_base_eob_cdf[txs_ctx][plane_type]
            [coeff_ctx as usize]
        );
      } else {
        symbol!(
          self,
          (cmp::min(level, 3)) as u32,
          &mut self.fc.coeff_base_cdf[txs_ctx][plane_type][coeff_ctx as usize]
        );
      }

      if level > NUM_BASE_LEVELS as u32 {
        let pos = scan[c as usize];
        let v = coeffs_in[pos as usize];
        let level = v.abs() as u16;

        if level <= NUM_BASE_LEVELS as u16 {
          continue;
        }

        let base_range = level - 1 - NUM_BASE_LEVELS as u16;
        let br_ctx = self.get_br_ctx(levels, pos as usize, bwl, tx_class);
        let mut idx = 0;

        loop {
          if idx >= COEFF_BASE_RANGE {
            break;
          }
          let k = cmp::min(base_range - idx as u16, BR_CDF_SIZE as u16 - 1);
          symbol!(
            self,
            k as u32,
            &mut self.fc.coeff_br_cdf
              [cmp::min(txs_ctx, TxSize::TX_32X32 as usize)][plane_type]
              [br_ctx]
          );
          if k < BR_CDF_SIZE as u16 - 1 {
            break;
          }
          idx += BR_CDF_SIZE - 1;
        }
      }
    }

    // Loop to code all signs in the transform block,
    // starting with the sign of DC (if applicable)
    for c in 0..eob {
      let v = coeffs_in[scan[c] as usize];
      let level = v.abs() as u32;
      let sign = if v < 0 {
        1
      } else {
        0
      };

      if level == 0 {
        continue;
      }

      if c == 0 {
        symbol!(
          self,
          sign,
          &mut self.fc.dc_sign_cdf[plane_type][txb_ctx.dc_sign_ctx]
        );
      } else {
        self.w.bit(sign as u16);
      }
      // save extra golomb codes for separate loop
      if level > (COEFF_BASE_RANGE + NUM_BASE_LEVELS) as u32 {
        let pos = scan[c];
        update_pos[num_updates] = pos;
        num_updates += 1;
      }
    }

    for i in 0..num_updates {
      self.w.write_golomb(
        coeffs_in[update_pos[i] as usize].abs() as u16
          - COEFF_BASE_RANGE as u16
          - 1
          - NUM_BASE_LEVELS as u16
      );
    }

    cul_level = cmp::min(COEFF_CONTEXT_MASK as u32, cul_level);

    self.bc.set_dc_sign(&mut cul_level, coeffs[0]);

    self.bc.set_coeff_context(plane, bo, tx_size, xdec, ydec, cul_level as u8);
  }

  pub fn checkpoint(&mut self) -> ContextWriterCheckpoint {
    ContextWriterCheckpoint {
      w: self.w.checkpoint(),
      fc: self.fc.clone(),
      bc: self.bc.checkpoint()
    }
  }

  pub fn rollback(&mut self, checkpoint: &ContextWriterCheckpoint) {
    self.w.rollback(&checkpoint.w);
    self.fc = checkpoint.fc.clone();
    self.bc.rollback(&checkpoint.bc);
    #[cfg(debug)] {
      if self.fc_map.is_some() {
        self.fc_map = Some(FieldMap {
          map: self.fc.build_map()
        });
      }
    }
  }
}
