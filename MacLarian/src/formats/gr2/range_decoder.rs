//! BitKnit Range Decoder - FINAL CORRECTED VERSION
//! ================================================
//!
//! This version implements the distance cache EXACTLY as shown in the pseudocode.
//!
//! KEY INSIGHT from sub_180074680.m:
//! - Distance cache is 8 × int32 values stored at context offsets 0x20-0x3c
//! - These are loaded into stack variables: decomp_var_71, decomp_var_6d, etc.
//! - Cache lookup uses: rbp + ((magic >> shift & 0x7) * 0x4 - 0x71)
//! - This indexes into the stack array of 8 distances

use crate::formats::gr2::decompressor::{
    rebalance_distance_table, rebalance_extra_distance_table, rebalance_symbol_table,
    BitKnitContext, DISTANCE_OFFSET_TABLE,
};

const DEBUG: bool = false; // Set to true to enable debug logging

/// Read little-endian uint16 safely
fn read_uint16(data: &[u8], pos: usize) -> u16 {
    if pos + 2 > data.len() {
        0
    } else {
        u16::from_le_bytes([data[pos], data[pos + 1]])
    }
}

/// Range decoder + LZ77 with CORRECT distance cache implementation.
pub fn decode_compressed_block(
    ctx: &mut BitKnitContext,
    input_pos: usize,
) -> Result<usize, String> {
    let mut input_pos = input_pos;
    let input_data = &ctx.compressed_data;
    let input_limit = input_data.len();
    let expected_output_size = ctx.expected_output_size;

    let output_start = ctx.output.len();
    let output_limit = output_start + expected_output_size;

    // Load bit buffers
    let mut bit_buf_0 = if !ctx.bit_buffers.is_empty() {
        ctx.bit_buffers[0]
    } else {
        0x100000001
    };
    let mut bit_buf_1 = if ctx.bit_buffers.len() > 1 {
        ctx.bit_buffers[1]
    } else {
        0x100000001
    };

    // Load magic selector
    let mut magic = ctx.magic;

    // FIXED: Load distance cache from context (decomp_var_71 through decomp_var_55)
    // Context stores 4 × uint64 at offsets 0x20, 0x28, 0x30, 0x38
    // When read as int32, this gives us 8 values
    let mut distance_cache: [i32; 8] = [1; 8];
    for i in 0..8 {
        let offset = 0x20 + (i * 4);
        let buf_idx = (offset - 0x20) / 8;
        let is_high = ((offset - 0x20) % 8) >= 4;

        if buf_idx < ctx.bit_buffers.len() {
            let val = ctx.bit_buffers[buf_idx];
            distance_cache[i] = if is_high {
                ((val >> 32) & 0xFFFFFFFF) as i32
            } else {
                (val & 0xFFFFFFFF) as i32
            };
        }
    }

    let mut last_distance: i32 = -1;

    // Initialize range decoder for state 7
    let mut range_value: u32;
    let mut base_value: u32;

    if ctx.state == 7 {
        if input_pos + 4 > input_data.len() {
            ctx.state = -1;
            return Ok(input_pos);
        }

        let word1 = read_uint16(input_data, input_pos);
        let word2 = read_uint16(input_data, input_pos + 2);
        input_pos += 4;

        let combined = ((word1 as u32) << 16) | (word2 as u32);
        let mut r8 = combined >> 4;
        let r10 = (combined & 0xF) + 0x10;

        let mut r9 = r8;
        if r8 < 0x10000 {
            if input_pos + 2 > input_data.len() {
                ctx.state = -1;
                return Ok(input_pos);
            }
            let word3 = read_uint16(input_data, input_pos);
            input_pos += 2;
            r9 = (r8 << 16) | (word3 as u32);
        }

        range_value = r9 >> (r10 & 0xF);

        if range_value < 0x10000 {
            if input_pos + 2 > input_data.len() {
                ctx.state = -1;
                return Ok(input_pos);
            }
            let word4 = read_uint16(input_data, input_pos);
            input_pos += 2;
            range_value = (range_value << 16) | (word4 as u32);
        }

        if r10 >= 0x10 {
            if input_pos + 2 > input_data.len() {
                ctx.state = -1;
                return Ok(input_pos);
            }
            let word5 = read_uint16(input_data, input_pos);
            input_pos += 2;
            r9 = (r9 << 16) | (word5 as u32);
        }

        let r12_initial = (((1u32 << r10).wrapping_sub(1)) & r9) | (1u32 << r10);
        base_value = r12_initial;

        eprintln!("[STATE 7 CALC] r10={} r9=0x{:08X} r12_initial=0x{:08X}", r10, r9, r12_initial);

        if ctx.output.len() == output_start && ctx.output.len() < output_limit {
            if input_pos + 2 > input_data.len() {
                ctx.state = -1;
                return Ok(input_pos);
            }

            // Write first literal from range_value (r15, lower 8 bits)
            ctx.output.push((range_value & 0xFF) as u8);

            // Calculate new range from upper bits of old range_value
            r8 = range_value >> 8;
            let word_next = read_uint16(input_data, input_pos);
            input_pos += 2;

            let new_range = (r8 << 16) | (word_next as u32);
            let temp_range = if r8 < 0x10000 { new_range } else { r8 };

            // SWAP to match pseudocode variable semantics after first literal:
            // r15 (range_value) should hold base calculation
            // r12 (base_value) should hold new range
            range_value = base_value;  // r15 = r12 (base calc)
            base_value = temp_range;   // r12 = new range
        }

        ctx.state = 8;

        eprintln!("[ENTERING MAIN LOOP] range_value=0x{:08X} (r15, for symbol search) base_value=0x{:08X} (r12, for refill)", range_value, base_value);
    } else {
        range_value = bit_buf_0 as u32;
        base_value = bit_buf_1 as u32;
    }

    // Main decompression loop
    let max_iterations = 100000;
    let mut iteration = 0;

    while ctx.output.len() < output_limit && input_pos < input_limit && iteration < max_iterations
    {
        iteration += 1;

        if input_pos >= input_data.len() || ctx.output.len() >= output_limit {
            break;
        }

        // Symbol decoding
        let rdx = (range_value & 0x7FFF) as u16;
        let table_idx = ctx.output.len() & 0x3;
        let table = &ctx.tables[table_idx];

        let quick_idx = (rdx >> 6) as usize;

        // DIAGNOSTIC: Log symbol decoding at problematic iterations
        if DEBUG
            && (iteration == 16
                || iteration == 17
                || iteration == 18
                || (iteration >= 12 && iteration <= 14))
        {
            println!("\n[SYMBOL DECODE] iter={}", iteration);
            println!("  rdx=0x{:04X} ({})", rdx, rdx);
            println!("  table_idx={}", table_idx);
            println!("  quick_idx=0x{:02X} ({})", quick_idx, quick_idx);
        }

        let mut symbol = if let Some(&s) = table.quick_lookup.get(&quick_idx) {
            s
        } else {
            let mut s = 0;
            for i in 0..table.size {
                if i + 1 >= table.freq.len() {
                    break;
                }
                if rdx < table.freq[i + 1] {
                    s = i;
                    break;
                }
            }
            s
        };

        if DEBUG
            && (iteration == 16
                || iteration == 17
                || iteration == 18
                || (iteration >= 12 && iteration <= 14))
        {
            println!("  symbol (after quick)={} (0x{:X})", symbol, symbol);
        }

        // CRITICAL FIX: Quick lookup can overshoot, so go backward if needed
        while symbol > 0 && rdx < table.freq[symbol] {
            symbol -= 1;
        }

        // Then go forward to find the exact symbol
        if symbol + 1 < table.freq.len() && rdx >= table.freq[symbol + 1] {
            symbol += 1;
        }

        // FIXED: Check freq[symbol + 1] not freq[symbol + 2]
        while symbol + 1 < table.freq.len() && rdx >= table.freq[symbol + 1] {
            symbol += 1;
        }

        if symbol >= table.size {
            symbol = table.size - 1;
        }

        // Debug symbol decoding
        if iteration <= 5 || (iteration >= 190 && iteration <= 210) {
            eprintln!("[SYMBOL] iter={} output_pos={} rdx=0x{:04X} table_idx={} symbol=0x{:X}",
                      iteration, ctx.output.len() + output_start, rdx, table_idx, symbol);

            // At key iterations, dump frequency table
            if iteration == 205 || iteration == 1 || iteration == 2 {
                let table = &ctx.tables[table_idx];
                eprintln!("  freq[0xC1]=0x{:04X} freq[0xC2]=0x{:04X} freq[0xC3]=0x{:04X}",
                          table.freq[0xC1], table.freq[0xC2], table.freq[0xC3]);
                eprintln!("  freq[0x100]=0x{:04X} freq[0x101]=0x{:04X} freq[0x102]=0x{:04X}",
                          table.freq[0x100], table.freq[0x101], table.freq[0x102]);
                eprintln!("  rdx=0x{:04X} checking: freq[0xC2] <= rdx < freq[0xC3]? {} <= {} < {}",
                          rdx, table.freq[0xC2], rdx, table.freq[0xC3]);
                eprintln!("  rdx=0x{:04X} checking: freq[0x101] <= rdx < freq[0x102]? {} <= {} < {}",
                          rdx, table.freq[0x101], rdx, table.freq[0x102]);
            }
        }

        if DEBUG
            && (iteration == 16
                || iteration == 17
                || iteration == 18
                || (iteration >= 12 && iteration <= 14))
        {
            println!("  symbol (final)={} (0x{:X})", symbol, symbol);
            if symbol + 1 < table.freq.len() {
                let freq_low_dbg = table.freq[symbol];
                let freq_high_dbg = table.freq[symbol + 1];
                println!(
                    "  freq[{}]=0x{:04X}, freq[{}]=0x{:04X}",
                    symbol,
                    freq_low_dbg,
                    symbol + 1,
                    freq_high_dbg
                );
                println!(
                    "  Check: {} <= {} < {}? {}",
                    freq_low_dbg,
                    rdx,
                    freq_high_dbg,
                    freq_low_dbg <= rdx && rdx < freq_high_dbg
                );
            }
            if symbol < 0x100 {
                println!("  -> LITERAL (symbol={})", symbol);
            } else if symbol < 0x120 {
                println!("  -> MATCH (len={})", symbol - 0xFE);
            } else {
                println!("  -> EXTENDED MATCH (symbol=0x{:X})", symbol);
            }
        }

        let freq_low = if symbol + 1 < table.freq.len() {
            table.freq[symbol]
        } else {
            0
        };

        let freq_high = if symbol + 1 < table.freq.len() {
            table.freq[symbol + 1]
        } else {
            freq_low.wrapping_add(1)
        };

        // Pseudocode line 115: r8 = ((r8 - rax) * (r15 >> 0xf) - rax) + rdx
        let mut r8 = ((freq_high.wrapping_sub(freq_low) as u32)
            .wrapping_mul(range_value >> 15)
            .wrapping_sub(freq_low as u32))
            .wrapping_add(rdx as u32);
        r8 &= 0xFFFFFFFF;

        let range_value_new = base_value;
        base_value = r8;

        if r8 < 0x10000 && input_pos + 2 <= input_data.len() {
            let word = read_uint16(input_data, input_pos);
            input_pos += 2;
            base_value = ((r8 << 16) | word as u32) & 0xFFFFFFFF;
        }

        range_value = range_value_new;

        // Update table adjustments (need mutable access)
        let table = &mut ctx.tables[table_idx];
        if symbol < table.adjust.len() {
            table.adjust[symbol] = std::cmp::min(table.adjust[symbol].wrapping_add(0x1F), 0xFFFF);
        }

        table.counter -= 1;
        if table.counter <= 0 {
            rebalance_symbol_table(table, symbol);
        }

        // Check if literal or match
        if symbol < 0x100 {
            // Literal byte - FIXED: Delta encoding from previous byte
            if ctx.output.len() < output_limit {
                let prev_idx = (ctx.output.len() as i32 + last_distance) as usize;
                let prev_byte = if prev_idx < ctx.output.len() {
                    ctx.output[prev_idx]
                } else {
                    0
                };
                let literal = prev_byte.wrapping_add(symbol as u8);

                // Debug first few literals and around byte 2087
                if iteration <= 5 || (ctx.output.len() + output_start >= 2080 && ctx.output.len() + output_start <= 2095) {
                    eprintln!("[LITERAL] iter={} output_pos={} symbol=0x{:02X} prev_idx={} prev_byte=0x{:02X} last_distance={} -> literal=0x{:02X}",
                              iteration, ctx.output.len() + output_start, symbol, prev_idx, prev_byte, last_distance, literal);
                }

                ctx.output.push(literal);
            }
        } else {
            // Match
            let match_len: u32;

            // Debug match starts
            if iteration >= 190 && iteration <= 210 {
                eprintln!("[MATCH START] iter={} output_pos={} symbol=0x{:X} -> match_len_base={}",
                          iteration, ctx.output.len() + output_start, symbol, symbol - 0xFE);
            }

            // Handle extended lengths
            if symbol >= 0x120 {
                let extra_len_bits = (symbol - 0x11F) as u32;
                let r10_temp = range_value;
                let r9_shifted = range_value >> extra_len_bits;
                let r8_temp = range_value >> extra_len_bits;
                let word_temp = if input_pos + 2 <= input_data.len() {
                    read_uint16(input_data, input_pos)
                } else {
                    0
                };
                let r8_combined = (r8_temp << 16) | word_temp as u32;

                if r8_temp < 0x10000 && input_pos + 2 <= input_data.len() {
                    input_pos += 2;
                    range_value = r8_combined;
                } else {
                    range_value = r8_temp;
                }

                let base_value_temp = if r8_temp < 0x10000 {
                    r8_combined
                } else {
                    r8_temp
                };
                base_value = base_value_temp;

                let mut r10_val = r10_temp;
                if extra_len_bits >= 0x10 && input_pos + 2 <= input_data.len() {
                    let word_extra = read_uint16(input_data, input_pos);
                    input_pos += 2;
                    r10_val = ((r10_temp << 16) | word_extra as u32) & 0xFFFFFFFF;
                }

                let mask_val = ((1u32 << extra_len_bits).wrapping_sub(1)) & r10_val;
                match_len = (1u32 << extra_len_bits) + 0x11E + mask_val;
            } else {
                match_len = (symbol - 0xFE) as u32;
            }

            // Decode distance symbol
            let dist_table = &ctx.dist_table;
            let rdx_dist = (range_value & 0x7FFF) as u16;
            let quick_idx_dist = (rdx_dist >> 9) as usize;

            let mut dist_symbol = if let Some(&s) = dist_table.quick_lookup.get(&quick_idx_dist) {
                s
            } else {
                let mut s = 0;
                for i in 0..dist_table.size {
                    if i + 1 >= dist_table.freq.len() {
                        break;
                    }
                    if rdx_dist < dist_table.freq[i + 1] {
                        s = i;
                        break;
                    }
                }
                s
            };

            if dist_symbol + 1 < dist_table.freq.len()
                && rdx_dist >= dist_table.freq[dist_symbol + 1]
            {
                dist_symbol += 1;
            }

            while dist_symbol + 2 < dist_table.freq.len()
                && rdx_dist >= dist_table.freq[dist_symbol + 2]
            {
                dist_symbol += 1;
            }

            if dist_symbol >= dist_table.size {
                dist_symbol = dist_table.size - 1;
            }

            if DEBUG && iteration <= 10 {
                println!(
                    "  [DIST] iter={} dist_symbol={} (using {})",
                    iteration,
                    dist_symbol,
                    if dist_symbol < 8 { "CACHE" } else { "COMPLEX" }
                );
            }

            // Update arithmetic decoder for distance
            let dist_freq_low = if dist_symbol + 1 < dist_table.freq.len() {
                dist_table.freq[dist_symbol]
            } else {
                0
            };

            let dist_freq_high = if dist_symbol + 2 < dist_table.freq.len() {
                dist_table.freq[dist_symbol + 1]
            } else {
                dist_freq_low.wrapping_add(1)
            };

            let mut r8_dist = ((dist_freq_high.wrapping_sub(dist_freq_low) as u32)
                .wrapping_mul(range_value >> 15)
                .wrapping_sub(dist_freq_low as u32))
                .wrapping_add(rdx_dist as u32);
            r8_dist &= 0xFFFFFFFF;

            let range_value_new = base_value;
            base_value = r8_dist;

            if r8_dist < 0x10000 && input_pos + 2 <= input_data.len() {
                let word = read_uint16(input_data, input_pos);
                input_pos += 2;
                base_value = ((r8_dist << 16) | word as u32) & 0xFFFFFFFF;
            }

            range_value = range_value_new;

            // Update distance table
            let dist_table = &mut ctx.dist_table;
            if dist_symbol < dist_table.adjust.len() {
                dist_table.adjust[dist_symbol] = std::cmp::min(
                    dist_table.adjust[dist_symbol].wrapping_add(0x1F),
                    0xFFFF,
                );
            }

            dist_table.counter -= 1;
            if dist_table.counter <= 0 {
                rebalance_distance_table(dist_table, dist_symbol);
            }

            // FIXED: Calculate distance using CORRECT cache lookup
            let distance: i32;

            if dist_symbol < 8 {
                // Use distance cache - EXACTLY as in pseudocode
                let shift = (dist_symbol + dist_symbol * 2) as u32;
                let cache_idx = if shift < 64 {
                    ((magic >> shift) & 0x7) as usize
                } else {
                    0
                };

                distance = distance_cache[cache_idx];

                if DEBUG && iteration <= 10 {
                    println!(
                        "  [CACHE] iter={} dist_symbol={} shift={} magic=0x{:016X} cache_idx={} distance={}",
                        iteration, dist_symbol, shift, magic, cache_idx, distance
                    );
                    println!("  [CACHE] distance_cache = {:?}", distance_cache);
                }

                // Cache rotation - EXACTLY as in pseudocode
                let extracted = if shift < 64 {
                    (magic >> shift) & 0x7
                } else {
                    0
                };
                let shifted = magic.wrapping_mul(8);
                let mask_low = if shift < 64 {
                    !(0xFFFFFFFFFFFFFFF8u64 << shift)
                } else {
                    0
                };
                let mask_high = if shift < 64 {
                    0xFFFFFFFFFFFFFFF8u64 << shift
                } else {
                    0xFFFFFFFFFFFFFFFF
                };

                let temp = extracted.wrapping_add(shifted);
                magic = (temp & mask_low) | (mask_high & magic);
            } else {
                // Complex distance calculation
                let extra_table = &ctx.extra_dist_table;
                let r8_dist = (range_value & 0x7FFF) as u16;
                let extra_symbol_guess = (r8_dist >> 9) as usize;

                let mut extra_symbol = if extra_symbol_guess >= extra_table.size {
                    extra_table.size - 1
                } else {
                    extra_symbol_guess
                };

                if extra_symbol + 1 < extra_table.freq.len()
                    && r8_dist >= extra_table.freq[extra_symbol + 1]
                {
                    extra_symbol += 1;
                }

                while extra_symbol + 2 < extra_table.freq.len()
                    && r8_dist >= extra_table.freq[extra_symbol + 2]
                {
                    extra_symbol += 1;
                }

                if extra_symbol >= extra_table.size {
                    extra_symbol = extra_table.size - 1;
                }

                // Update arithmetic decoder for extra_symbol
                let extra_freq_low = if extra_symbol + 1 < extra_table.freq.len() {
                    extra_table.freq[extra_symbol]
                } else {
                    0
                };

                let extra_freq_high = if extra_symbol + 2 < extra_table.freq.len() {
                    extra_table.freq[extra_symbol + 1]
                } else {
                    extra_freq_low.wrapping_add(1)
                };

                let rax_temp = range_value >> 15;
                let r15_temp = (extra_freq_high.wrapping_sub(extra_freq_low) as u32).wrapping_mul(rax_temp);
                let mut r15_updated = r15_temp.wrapping_sub(extra_freq_low as u32).wrapping_add(r8_dist as u32);
                r15_updated &= 0xFFFFFFFF;

                let rdx_temp = (r15_updated << 16)
                    | if input_pos + 2 <= input_data.len() {
                        read_uint16(input_data, input_pos) as u32
                    } else {
                        0
                    };

                if r15_updated < 0x10000 && input_pos + 2 <= input_data.len() {
                    input_pos += 2;
                    range_value = rdx_temp;
                } else {
                    range_value = r15_updated;
                }

                // Update extra distance table
                let extra_table = &mut ctx.extra_dist_table;
                if extra_symbol < extra_table.adjust.len() {
                    extra_table.adjust[extra_symbol] = std::cmp::min(
                        extra_table.adjust[extra_symbol].wrapping_add(0x1F),
                        0xFFFF,
                    );
                }

                extra_table.counter -= 1;
                if extra_table.counter <= 0 {
                    rebalance_extra_distance_table(extra_table, extra_symbol);
                }

                // Extract distance bits
                let extra_bits_count = (extra_symbol & 0xF) as u32;

                // CRITICAL FIX: Extract bits from base_value BEFORE any shifts!
                let r9_original = base_value;

                // Now shift base_value for the decoder state
                let base_value_shifted = base_value >> extra_bits_count;

                if base_value_shifted < 0x10000 && input_pos + 2 <= input_data.len() {
                    let word = read_uint16(input_data, input_pos);
                    input_pos += 2;
                    base_value = ((base_value_shifted << 16) | word as u32) & 0xFFFFFFFF;
                } else {
                    base_value = base_value_shifted;
                }

                // If we need more bits (extra_symbol >= 0x10), read them into r9_original
                let mut r9_val = r9_original;
                if extra_symbol >= 0x10 && input_pos + 2 <= input_data.len() {
                    let word = read_uint16(input_data, input_pos);
                    input_pos += 2;
                    r9_val = ((r9_original << 16) | word as u32) & 0xFFFFFFFF;
                }

                // Calculate final distance
                distance = if extra_bits_count > 0 {
                    let mask = (1u32 << extra_bits_count).wrapping_sub(1);
                    let extracted_bits = r9_val & mask;
                    (((extracted_bits.wrapping_sub(1)) << 5)
                        .wrapping_add(dist_symbol as u32)
                        .wrapping_add((0x20u32 << extra_bits_count).wrapping_sub(7))) as i32
                } else {
                    (dist_symbol + 1) as i32
                };

                if DEBUG && (iteration <= 20 || distance > (ctx.output.len() - output_start) as i32)
                {
                    println!("\n[COMPLEX DIST] iter={}", iteration);
                    println!("  extra_symbol={} extra_bits_count={}", extra_symbol, extra_bits_count);
                    println!("  FINAL distance={}", distance);
                    println!("  output_size={}", ctx.output.len() - output_start);
                }

                let distance = if distance > 0x100000 || distance <= 0 {
                    (dist_symbol + 1) as i32
                } else {
                    distance
                };

                // FIXED: Update distance cache - EXACTLY as in pseudocode
                // Rotate magic first
                let shift = (dist_symbol + dist_symbol * 2) as u32;

                let extracted = if shift < 64 {
                    (magic >> shift) & 0x7
                } else {
                    0
                };
                let shifted = magic.wrapping_mul(8);
                let mask_low = if shift < 64 {
                    !(0xFFFFFFFFFFFFFFF8u64 << shift)
                } else {
                    0
                };
                let mask_high = if shift < 64 {
                    0xFFFFFFFFFFFFFFF8u64 << shift
                } else {
                    0xFFFFFFFFFFFFFFFF
                };

                let temp = extracted.wrapping_add(shifted);
                magic = (temp & mask_low) | (mask_high & magic);

                // Then update cache using rotated magic
                let old_idx = ((magic >> 0x12) & 0x7) as usize;
                let new_idx = ((magic >> 0x15) & 0x7) as usize;

                if DEBUG && iteration <= 10 {
                    println!(
                        "  [CACHE UPDATE] old_idx={} new_idx={} distance={}",
                        old_idx, new_idx, distance
                    );
                    println!("  [CACHE UPDATE] Before: {:?}", distance_cache);
                }

                if new_idx < 8 && old_idx < 8 {
                    distance_cache[new_idx] = distance_cache[old_idx];
                    distance_cache[old_idx] = distance;

                    if DEBUG && iteration <= 10 {
                        println!("  [CACHE UPDATE] After: {:?}", distance_cache);
                    }
                }
            }

            // Copy match from history
            if distance > (ctx.output.len() - output_start) as i32 || distance <= 0 {
                continue;
            }

            let bytes_remaining = output_limit - ctx.output.len();
            if bytes_remaining == 0 {
                break;
            }

            let match_len = std::cmp::min(match_len as usize, bytes_remaining);
            let dest_start = ctx.output.len();
            let src_start = dest_start - distance as usize;

            // Debug around byte 2087
            let will_write_target = dest_start + output_start >= 2080 && dest_start + output_start <= 2095;
            if will_write_target {
                eprintln!("[MATCH] iter={} output_pos={} distance={} match_len={} src_start={} dest_start={}",
                          iteration, dest_start + output_start, distance, match_len, src_start + output_start, dest_start + output_start);
            }

            if DEBUG && iteration <= 10 {
                println!(
                    "  [COPY] About to copy first 8 bytes from src_start={}",
                    src_start
                );
            }

            // Copy first 8 bytes
            for i in 0..std::cmp::min(8, match_len) {
                if src_start + i < ctx.output.len() {
                    let byte_val = ctx.output[src_start + i];
                    if DEBUG && iteration <= 10 {
                        println!("  [COPY] output[{}] = 0x{:02X}", src_start + i, byte_val);
                    }

                    if will_write_target && dest_start + i + output_start >= 2080 && dest_start + i + output_start <= 2095 {
                        eprintln!("  [MATCH BYTE] copying byte {} from pos {} (0x{:02X})",
                                  dest_start + i + output_start, src_start + i + output_start, byte_val);
                    }

                    ctx.output.push(byte_val);
                }
            }

            // Adjust source for small distances AFTER first 8 bytes copied
            let mut adjusted_src = src_start;
            if distance < 8 && (distance as usize) < DISTANCE_OFFSET_TABLE.len() {
                adjusted_src = (adjusted_src as i32 + DISTANCE_OFFSET_TABLE[distance as usize])
                    as usize;
            }

            // Copy remaining bytes (if match_len > 8)
            if match_len > 8 {
                let relative_src = adjusted_src as i32 - dest_start as i32;
                let mut remaining = match_len - 8;
                let num_chunks = ((remaining - 1) >> 3) + 1;

                for _ in 0..num_chunks {
                    let dest_pos = ctx.output.len();
                    let chunk_size = std::cmp::min(8, remaining);
                    for i in 0..chunk_size {
                        let src_idx = (dest_pos as i32 + relative_src + i as i32) as usize;
                        if src_idx < ctx.output.len() {
                            let byte_val = ctx.output[src_idx];
                            ctx.output.push(byte_val);
                        }
                    }
                    remaining -= chunk_size;
                }
            }

            last_distance = -distance;
        }
    }

    // FIXED: Save distance cache back to context
    for (i, &dist) in distance_cache.iter().enumerate() {
        let offset = 0x20 + (i * 4);
        let buf_idx = (offset - 0x20) / 8;
        let is_high = ((offset - 0x20) % 8) >= 4;

        if buf_idx < ctx.bit_buffers.len() {
            if is_high {
                // Update high 32 bits
                ctx.bit_buffers[buf_idx] =
                    (ctx.bit_buffers[buf_idx] & 0xFFFFFFFF) | ((dist as u64) << 32);
            } else {
                // Update low 32 bits
                ctx.bit_buffers[buf_idx] =
                    (ctx.bit_buffers[buf_idx] & 0xFFFFFFFF00000000) | (dist as u32 as u64);
            }
        }
    }

    // Save other state
    ctx.magic = magic;

    if DEBUG {
        println!(
            "[DEBUG] Decoded {} bytes in {} iterations",
            ctx.output.len() - output_start,
            iteration
        );
    }

    if ctx.output.len() >= output_limit || input_pos >= input_limit {
        ctx.state = 3;
    }

    Ok(input_pos)
}
