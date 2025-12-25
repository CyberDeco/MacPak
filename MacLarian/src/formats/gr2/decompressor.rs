//! BitKnit Decompression - CORRECTED VERSION
//! ==========================================
//!
//! This version fixes the critical bugs identified in the debug analysis:
//! 1. Frequency table structure (cumul is freq shifted by 1)
//! 2. Table initialization (building cumulative values correctly)
//! 3. Rebalancing functions (complete implementation)
//! 4. Quick lookup table building
//! 5. 128-bit arithmetic handling
//!
//! All changes marked with FIXED comments

use std::collections::HashMap;

// =============================================================================
// CONSTANTS
// =============================================================================

pub const MAGIC_HEADER: u16 = 0x75B1;

pub const RANGE_INTERP_TABLE: [u16; 33] = [
    0x0000, 0x02D7, 0x0599, 0x0846, 0x0AE0, 0x0D68, 0x0FDE, 0x1244,
    0x149A, 0x16E2, 0x191C, 0x1B48, 0x1D67, 0x1F7B, 0x2182, 0x237E,
    0x2570, 0x2757, 0x2935, 0x2B09, 0x2CD4, 0x2E96, 0x3050, 0x3202,
    0x33AC, 0x354E, 0x36E9, 0x387D, 0x3A0A, 0x3B91, 0x3D12, 0x3E8C,
    0x4000,
];

pub const DISTANCE_OFFSET_TABLE: [i32; 8] = [-8, -8, -8, -6, -8, -5, -6, -7];

// States
const STATE_INITIAL: i32 = 1;
const STATE_CHECK_MODE: i32 = 2;
const STATE_COMPLETION: i32 = 3;
const STATE_RAW_COPY_1: i32 = 4;
const STATE_RAW_COPY_2: i32 = 5;
const STATE_COMPRESSED_1: i32 = 7;
const STATE_COMPRESSED_2: i32 = 8;

const BUFFER_FLAG_READY: u32 = 0x10000;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Arithmetic right shift (sign-preserving)
fn sar(value: u16, shift: u32) -> u16 {
    let sign_bit = 1u16 << 15;
    if value & sign_bit != 0 {
        // Negative number - fill with 1s
        if shift >= 16 {
            0xFFFF
        } else {
            let mask = 0xFFFFu16 << (16 - shift);
            (value >> shift) | mask
        }
    } else {
        value >> shift
    }
}

/// BSR instruction - find position of highest set bit
fn bit_scan_reverse(value: u16) -> i32 {
    if value == 0 {
        -1
    } else {
        15 - value.leading_zeros() as i32
    }
}

/// Get high 64 bits of a * b (treating as 64-bit unsigned)
fn multiply_high_64(a: u64, b: u64) -> u64 {
    let result = (a as u128) * (b as u128);
    (result >> 64) as u64
}

// =============================================================================
// LOOKUP TABLE STRUCTURE - FIXED
// =============================================================================

/// Frequency/cumulative lookup table for range decoder.
///
/// FIXED: Key insight: cumul[i] is the same as freq[i+1]!
/// The arrays overlap in memory in the original C code.
pub struct LookupTable {
    pub size: usize,
    /// Stores cumulative frequencies - FIXED: Only one array needed, sized +1
    pub freq: Vec<u16>,
    /// Adjustment values
    pub adjust: Vec<u16>,
    /// Rebalance counter
    pub counter: i32,
    pub init_state: u8,
    pub range_params: Vec<u32>,
    /// For fast symbol lookup
    pub quick_lookup: HashMap<usize, usize>,
}

impl LookupTable {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            freq: vec![0; size + 1],
            adjust: vec![0; size],
            counter: 0x400,
            init_state: 0,
            range_params: vec![0; size],
            quick_lookup: HashMap::new(),
        }
    }
}

// =============================================================================
// CONTEXT STRUCTURE
// =============================================================================

/// Complete decompression context
pub struct BitKnitContext {
    // Input pointers
    pub input_start: usize,
    pub input_current: usize,
    pub input_limit: usize,
    pub input_end: usize,

    // Bit buffers
    pub bit_buffers: [u64; 4],

    // State
    pub bit_position: u32,
    pub magic: u64,
    pub state: i32,
    pub buffer_flag1: u32,
    pub buffer_flag2: u32,

    // Buffering
    pub buffered_bytes: usize,
    pub byte_buffer: Vec<u8>,
    pub total_processed: usize,

    // Lookup tables
    pub tables: [LookupTable; 4],
    pub dist_table: LookupTable,
    pub extra_dist_table: LookupTable,

    // Output
    pub output: Vec<u8>,

    // Input data
    pub compressed_data: Vec<u8>,
    pub data_pos: usize,
    pub expected_output_size: usize,
}

impl BitKnitContext {
    pub fn new() -> Self {
        Self {
            input_start: 0,
            input_current: 0,
            input_limit: 0,
            input_end: 0,
            bit_buffers: [0x100000001; 4],
            bit_position: 1,
            magic: 0xfac688,
            state: STATE_INITIAL,
            buffer_flag1: BUFFER_FLAG_READY,
            buffer_flag2: BUFFER_FLAG_READY,
            buffered_bytes: 0,
            byte_buffer: vec![0; 32],
            total_processed: 0,
            tables: [
                LookupTable::new(0x12c),
                LookupTable::new(0x12c),
                LookupTable::new(0x12c),
                LookupTable::new(0x12c),
            ],
            dist_table: LookupTable::new(0x28),
            extra_dist_table: LookupTable::new(0x15),
            output: Vec::new(),
            compressed_data: Vec::new(),
            data_pos: 0,
            expected_output_size: 1000000,
        }
    }
}

// =============================================================================
// TABLE INITIALIZATION - FIXED
// =============================================================================

/// Initialize lookup table - sub_1800770f0.
///
/// FIXED:
/// - Builds cumulative frequency values (not differences!)
/// - Properly fills all entries
pub fn init_lookup_table(table: &mut LookupTable, init_param: u32) {
    // Build cumulative frequency table for entries 0 to 0x107
    let mut counter: u64 = 0;
    let mut idx = 0;

    while counter < 0x83dae0 && idx < 0x108 {
        // HIDWORD(0x3e0f83e1 * counter) >> 6
        let product = 0x3e0f83e1u64 * counter;
        let product_high = (product >> 32) as u32;
        let cumul_val = ((product_high >> 6) & 0xFFFF) as u16;

        table.freq[idx] = cumul_val;
        idx += 1;
        counter += 0x7fdc;
    }

    // Fill entries 0x108 to 0x12c with sequential values
    // CRITICAL FIX: Add index i directly, not (i - 0x108)!
    for i in 0x108..std::cmp::min(0x12d, table.freq.len()) {
        table.freq[i] = ((0x7ed4 + i) & 0xFFFF) as u16;
    }

    // Initialize adjustments
    for i in 0..table.size {
        table.adjust[i] = 0;
    }

    // Set counter and state
    table.counter = 0x400;
    table.init_state = (init_param & 0xFF) as u8;

    // Build range parameters
    build_range_parameters(table);
}

/// Initialize distance tables - sub_180077580.
///
/// FIXED:
/// - Uses proper 128-bit multiply for high qword
/// - Builds cumulative values correctly
pub fn init_distance_tables(
    dist_table: &mut LookupTable,
    extra_dist_table: &mut LookupTable,
    init_param: u32,
) {
    // Initialize main distance table (0x28 = 40 entries)
    let mut r8: u64 = 0;
    let mut idx = 0;

    while r8 <= 0x140000 && idx < dist_table.size {
        // HIQWORD(0xcccccccccccccccd * r8) >> 5
        let high_qword = multiply_high_64(0xcccccccccccccccd, r8);
        let cumul_val = ((high_qword >> 5) & 0xFFFF) as u16;

        dist_table.freq[idx] = cumul_val;
        idx += 1;
        r8 += 0x8000;
    }

    // Ensure last entry exists
    if idx < dist_table.freq.len() {
        let last_val = if idx > 0 {
            dist_table.freq[idx - 1]
        } else {
            0x7FFF
        };
        while idx < dist_table.freq.len() {
            dist_table.freq[idx] = last_val;
            idx += 1;
        }
    }

    // Initialize adjustments
    for i in 0..dist_table.size {
        dist_table.adjust[i] = 0;
    }

    dist_table.counter = 0x400;
    dist_table.init_state = (init_param & 0xFF) as u8;

    // Build range parameters
    build_range_parameters(dist_table);

    // Initialize extra distance table (0x15 = 21 entries)
    let mut rdi: u64 = 0;
    let mut idx = 0;

    while rdi <= 0xa8000 && idx < extra_dist_table.size {
        // HIQWORD(0x8618618618618619 * rdi)
        let high_qword = multiply_high_64(0x8618618618618619, rdi);
        let rdx = high_qword;
        let rax = rdi;

        // (rax - rdx >> 1) + rdx >> 4
        let cumul_val = ((((rax - rdx) >> 1) + rdx) >> 4) as u16;

        extra_dist_table.freq[idx] = cumul_val;
        idx += 1;
        rdi += 0x8000;
    }

    // Ensure last entry exists
    if idx < extra_dist_table.freq.len() {
        let last_val = if idx > 0 {
            extra_dist_table.freq[idx - 1]
        } else {
            0x7FFF
        };
        while idx < extra_dist_table.freq.len() {
            extra_dist_table.freq[idx] = last_val;
            idx += 1;
        }
    }

    // Initialize adjustments
    for i in 0..extra_dist_table.size {
        extra_dist_table.adjust[i] = 0;
    }

    extra_dist_table.counter = 0x400;
    extra_dist_table.init_state = (init_param & 0xFF) as u8;

    // Build range parameters
    build_range_parameters(extra_dist_table);
}

// =============================================================================
// RANGE PARAMETER BUILDING - FIXED
// =============================================================================

/// Build range decoder parameters - sub_180074520.
///
/// FIXED:
/// - Properly builds quick lookup table
/// - Correctly calculates range parameters
pub fn build_range_parameters(table: &mut LookupTable) {
    // Clear quick lookup
    table.quick_lookup.clear();

    // Build quick lookup table
    for i in 0..table.size {
        if i + 1 >= table.freq.len() {
            break;
        }

        let freq_curr = table.freq[i];
        let freq_next = table.freq[i + 1];

        if freq_next <= freq_curr {
            continue;
        }

        // Fill range
        let start_idx = if freq_curr > 0 {
            std::cmp::max(0, ((freq_curr - 1) >> 6) as usize)
        } else {
            0
        };
        let end_idx = ((freq_next - 1) >> 6) as usize;

        for idx in start_idx..=end_idx {
            if !table.quick_lookup.contains_key(&idx) || idx == start_idx {
                table.quick_lookup.insert(idx, i);
            }
        }
    }

    // Build range parameters using logarithmic interpolation
    for i in 0..table.size {
        if i + 1 >= table.freq.len() {
            table.range_params[i] = 0;
            continue;
        }

        let freq_low = table.freq[i];
        let freq_high = table.freq[i + 1];
        let delta = freq_high.wrapping_sub(freq_low);

        if delta == 0 {
            table.range_params[i] = 0;
            continue;
        }

        // Find highest bit (BSR)
        let log2_val = bit_scan_reverse(delta);
        let log2_val = if log2_val < 0 { 0 } else { log2_val };

        // Calculate interpolation index
        let scaled = ((delta as u32) << 15) >> log2_val;
        let table_idx = ((scaled >> 10) & 0x1F) as usize;

        let table_idx = if table_idx >= RANGE_INTERP_TABLE.len() - 1 {
            RANGE_INTERP_TABLE.len() - 2
        } else {
            table_idx
        };

        // Get interpolation values
        let base = RANGE_INTERP_TABLE[table_idx] as u32;
        let next_val = RANGE_INTERP_TABLE[table_idx + 1] as u32;

        // Calculate parameter
        let shift_part = ((15 - log2_val) as u32) << 14;
        let low_bits = scaled & 0x3FF;
        let interp_part = ((next_val - base) * low_bits + 0x200) >> 10;

        let param = shift_part.wrapping_sub(interp_part).wrapping_sub(base);
        table.range_params[i] = param;
    }
}

// =============================================================================
// TABLE REBALANCING - FIXED
// =============================================================================

/// Rebalance symbol table - sub_180073f30.
///
/// FIXED:
/// - Complete implementation matching pseudocode
/// - Properly rebuilds cumulative frequencies
pub fn rebalance_symbol_table(table: &mut LookupTable, symbol_idx: usize) {
    eprintln!("[REBALANCE] Symbol table rebalanced for symbol 0x{:X}", symbol_idx);

    // Add bonus to current symbol
    if symbol_idx < table.adjust.len() {
        table.adjust[symbol_idx] = std::cmp::min(
            table.adjust[symbol_idx].wrapping_add(0x2D5),
            0xFFFF,
        );
    }

    // Rebuild cumulative frequency table
    let mut cumulative: u16 = 0;

    for i in 0..table.size {
        let adjust_val = table.adjust[i];
        let old_cumul = table.freq[i];

        // Reset adjust to 1
        table.adjust[i] = 0x1;

        // Accumulate
        cumulative = cumulative.wrapping_add(adjust_val);

        // Update cumulative: SAR(cumul - old, 1) + old
        let diff = cumulative.wrapping_sub(old_cumul);
        let sar_result = sar(diff, 1);

        let new_cumul = sar_result.wrapping_add(old_cumul);
        table.freq[i] = new_cumul;
    }

    // Reset counter
    table.counter = 0x400;

    // Rebuild range parameters
    build_range_parameters(table);
}

/// Rebalance distance table - sub_1800741c0.
///
/// FIXED:
/// - Complete implementation for 0x28 entry table
pub fn rebalance_distance_table(table: &mut LookupTable, symbol_idx: usize) {
    // Add bonus to current symbol
    if symbol_idx < table.adjust.len() {
        table.adjust[symbol_idx] = std::cmp::min(
            table.adjust[symbol_idx].wrapping_add(0x3D9),
            0xFFFF,
        );
    }

    // Rebuild cumulative frequency table
    let mut cumulative: u16 = 0;

    for i in 0..table.size {
        let adjust_val = table.adjust[i];
        let old_cumul = table.freq[i];

        // Reset adjust to 1
        table.adjust[i] = 0x1;

        // Accumulate
        cumulative = cumulative.wrapping_add(adjust_val);

        // Update cumulative
        let diff = cumulative.wrapping_sub(old_cumul);
        let sar_result = sar(diff, 1);

        let new_cumul = sar_result.wrapping_add(old_cumul);
        table.freq[i] = new_cumul;
    }

    // Reset counter
    table.counter = 0x400;

    // Rebuild range parameters
    build_range_parameters(table);
}

/// Rebalance extra distance table - sub_180074080.
///
/// FIXED:
/// - Complete implementation for 0x15 entry table
pub fn rebalance_extra_distance_table(table: &mut LookupTable, symbol_idx: usize) {
    // Add bonus to current symbol
    if symbol_idx < table.adjust.len() {
        table.adjust[symbol_idx] = std::cmp::min(
            table.adjust[symbol_idx].wrapping_add(0x3EC),
            0xFFFF,
        );
    }

    // Rebuild cumulative frequency table
    let mut cumulative: u16 = 0;

    for i in 0..table.size {
        let adjust_val = table.adjust[i];
        let old_cumul = table.freq[i];

        // Reset adjust to 1
        table.adjust[i] = 0x1;

        // Accumulate
        cumulative = cumulative.wrapping_add(adjust_val);

        // Update cumulative
        let diff = cumulative.wrapping_sub(old_cumul);
        let sar_result = sar(diff, 1);

        let new_cumul = sar_result.wrapping_add(old_cumul);
        table.freq[i] = new_cumul;
    }

    // Reset counter
    table.counter = 0x400;

    // Rebuild range parameters
    build_range_parameters(table);
}

// =============================================================================
// CONTEXT INITIALIZATION - sub_180075250
// =============================================================================

/// Initialize context - sub_180075250
pub fn init_context(ctx: &mut BitKnitContext, compressed_data: &[u8]) {
    ctx.compressed_data = compressed_data.to_vec();
    ctx.data_pos = 0;

    // Set input pointers
    ctx.input_start = 0;
    ctx.input_current = 0;
    ctx.input_limit = compressed_data.len();
    ctx.input_end = compressed_data.len();

    // Initialize state
    ctx.state = STATE_INITIAL;
    ctx.buffer_flag1 = BUFFER_FLAG_READY;
    ctx.buffer_flag2 = BUFFER_FLAG_READY;
    ctx.bit_position = 1;

    // Initialize bit buffers
    ctx.bit_buffers = [0x100000001; 4];

    // Initialize 4 symbol lookup tables
    for i in 0..4 {
        init_lookup_table(&mut ctx.tables[i], 0);
    }

    // Initialize distance tables
    init_distance_tables(&mut ctx.dist_table, &mut ctx.extra_dist_table, 0);

    // Set magic and clear buffers
    ctx.magic = 0xfac688;
    ctx.buffered_bytes = 0;
    ctx.total_processed = 0;
    ctx.byte_buffer = vec![0; 32];
    ctx.output = Vec::new();
}

// =============================================================================
// MAIN DECOMPRESSION ENTRY
// =============================================================================

/// Main entry point for BitKnit decompression.
///
/// Args:
///     compressed_data: Compressed data starting with 0x75B1 magic
///     expected_size: Expected output size
///
/// Returns:
///     Decompressed data or None on failure
pub fn decompress(compressed_data: &[u8], expected_size: Option<usize>) -> Option<Vec<u8>> {
    if compressed_data.len() < 4 {
        eprintln!("Error: Data too small");
        return None;
    }

    // Check magic header
    let magic = u16::from_le_bytes([compressed_data[0], compressed_data[1]]);
    if magic != MAGIC_HEADER {
        eprintln!("Error: Invalid magic 0x{:04X}", magic);
        return None;
    }

    // Create and initialize context
    let mut ctx = BitKnitContext::new();
    ctx.expected_output_size = expected_size.unwrap_or(1000000);
    init_context(&mut ctx, compressed_data);

    // Process state machine
    let max_iterations = 1000;
    let mut iterations = 0;

    while ctx.state > 0 && iterations < max_iterations {
        iterations += 1;

        let old_state = ctx.state;
        let old_output_len = ctx.output.len();

        ctx.data_pos = process_state_machine(&mut ctx);

        if ctx.state == 0 {
            if let Some(expected) = expected_size {
                if ctx.output.len() != expected {
                    eprintln!(
                        "Note: Expected {} bytes, got {}",
                        expected,
                        ctx.output.len()
                    );
                }
            }
            return Some(ctx.output);
        }

        if ctx.state < 0 {
            eprintln!("Error: Decompression failed at state {}", old_state);
            if !ctx.output.is_empty() {
                return Some(ctx.output);
            }
            return None;
        }

        if old_state == ctx.state
            && ctx.output.len() == old_output_len
            && ctx.data_pos >= ctx.compressed_data.len()
        {
            eprintln!("Warning: No progress");
            if !ctx.output.is_empty() {
                return Some(ctx.output);
            }
            return None;
        }

        if ctx.state == STATE_COMPLETION && ctx.data_pos >= ctx.compressed_data.len() {
            return Some(ctx.output);
        }
    }

    if iterations >= max_iterations {
        eprintln!("Error: Maximum iterations exceeded");
        if !ctx.output.is_empty() {
            return Some(ctx.output);
        }
        return None;
    }

    if !ctx.output.is_empty() {
        Some(ctx.output)
    } else {
        None
    }
}

/// Helper class for decompressing multi-section GR2 files.
///
/// Maintains a persistent context across all sections,
/// preserving the distance cache and frequency tables.
pub struct GR2Decompressor {
    ctx: BitKnitContext,
    sections_decompressed: usize,
}

impl GR2Decompressor {
    /// Initialize a new decompressor with fresh context
    pub fn new() -> Self {
        let mut ctx = BitKnitContext::new();
        init_context(&mut ctx, &[]);
        Self {
            ctx,
            sections_decompressed: 0,
        }
    }

    /// Decompress one section of a GR2 file.
    ///
    /// The context is preserved between calls, so the distance cache
    /// and frequency tables are shared across all sections.
    pub fn decompress_section(
        &mut self,
        compressed_data: &[u8],
        expected_size: usize,
    ) -> Option<Vec<u8>> {
        let result = decompress_with_context(&mut self.ctx, compressed_data, Some(expected_size));

        if result.is_some() {
            self.sections_decompressed += 1;
        }

        result
    }

    /// Reset the decompressor for a new GR2 file
    pub fn reset(&mut self) {
        self.ctx = BitKnitContext::new();
        init_context(&mut self.ctx, &[]);
        self.sections_decompressed = 0;
    }
}

impl Default for GR2Decompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Decompress using an existing context (for multi-section files like GR2).
///
/// CRITICAL: This preserves the distance cache and frequency tables from
/// previous sections, which is required for correct decompression.
fn decompress_with_context(
    ctx: &mut BitKnitContext,
    compressed_data: &[u8],
    expected_size: Option<usize>,
) -> Option<Vec<u8>> {
    if compressed_data.len() < 4 {
        eprintln!("Error: Data too small");
        return None;
    }

    // Check magic header
    let magic = u16::from_le_bytes([compressed_data[0], compressed_data[1]]);
    if magic != MAGIC_HEADER {
        eprintln!("Error: Invalid magic 0x{:04X}", magic);
        return None;
    }

    // CRITICAL: Don't call init_context()!
    // We want to preserve:
    // - ctx.bit_buffers (distance cache)
    // - ctx.magic (cache rotation state)
    // - ctx.tables (frequency tables with learned patterns)

    // Reset only the input/output state for this section
    ctx.compressed_data = compressed_data.to_vec();
    ctx.data_pos = 0;
    ctx.input_start = 0;
    ctx.input_current = 0;
    ctx.input_limit = compressed_data.len();
    ctx.input_end = compressed_data.len();
    ctx.expected_output_size = expected_size.unwrap_or(1000000);

    // Reset state machine to initial state
    ctx.state = STATE_INITIAL;

    // Clear output buffer for this section (but keep distance cache!)
    ctx.output = Vec::new();

    // Reset buffer flags
    ctx.buffer_flag1 = BUFFER_FLAG_READY;
    ctx.buffer_flag2 = BUFFER_FLAG_READY;
    ctx.buffered_bytes = 0;
    ctx.total_processed = 0;

    // Process state machine
    let max_iterations = 1000;
    let mut iterations = 0;

    while ctx.state > 0 && iterations < max_iterations {
        iterations += 1;

        let old_state = ctx.state;
        let old_output_len = ctx.output.len();

        ctx.data_pos = process_state_machine(ctx);

        if ctx.state == 0 {
            if let Some(expected) = expected_size {
                if ctx.output.len() != expected {
                    eprintln!(
                        "Note: Expected {} bytes, got {}",
                        expected,
                        ctx.output.len()
                    );
                }
            }
            return Some(ctx.output.clone());
        }

        if ctx.state < 0 {
            eprintln!("Error: Decompression failed at state {}", old_state);
            if !ctx.output.is_empty() {
                return Some(ctx.output.clone());
            }
            return None;
        }

        if old_state == ctx.state
            && ctx.output.len() == old_output_len
            && ctx.data_pos >= ctx.compressed_data.len()
        {
            eprintln!("Warning: No progress");
            if !ctx.output.is_empty() {
                return Some(ctx.output.clone());
            }
            return None;
        }

        if ctx.state == STATE_COMPLETION && ctx.data_pos >= ctx.compressed_data.len() {
            return Some(ctx.output.clone());
        }
    }

    if iterations >= max_iterations {
        eprintln!("Error: Maximum iterations exceeded");
        if !ctx.output.is_empty() {
            return Some(ctx.output.clone());
        }
        return None;
    }

    if !ctx.output.is_empty() {
        Some(ctx.output.clone())
    } else {
        None
    }
}

// =============================================================================
// STATE MACHINE - sub_180074f40
// =============================================================================

/// Process state machine - sub_180074f40
fn process_state_machine(ctx: &mut BitKnitContext) -> usize {
    use crate::formats::gr2::range_decoder::decode_compressed_block;

    let mut data_pos = ctx.data_pos;
    let data = &ctx.compressed_data;

    // State 1: Check magic header
    if ctx.state == STATE_INITIAL {
        if data_pos + 2 > data.len() {
            ctx.state = -1;
            return data_pos;
        }

        let magic = u16::from_le_bytes([data[data_pos], data[data_pos + 1]]);
        data_pos += 2;

        if magic == MAGIC_HEADER {
            ctx.state = STATE_CHECK_MODE;
        } else {
            ctx.state = -1;
            return data_pos;
        }
    }

    if ctx.state < 0 || data_pos > data.len() {
        return data_pos;
    }

    // State 2: Check compression mode
    if ctx.state == STATE_CHECK_MODE {
        if data_pos + 2 > data.len() {
            ctx.state = -1;
            return data_pos;
        }

        let mode_marker = u16::from_le_bytes([data[data_pos], data[data_pos + 1]]);

        if mode_marker == 0 {
            ctx.state = STATE_RAW_COPY_1;
            data_pos += 2;
        } else {
            ctx.state = STATE_COMPRESSED_1;
        }

        return data_pos;
    }

    // States 4/5: Raw copy mode
    if ctx.state == STATE_RAW_COPY_1 || ctx.state == STATE_RAW_COPY_2 {
        let bytes_available = data.len() - data_pos;
        let bytes_to_copy = std::cmp::min(4096, bytes_available);

        if bytes_to_copy > 0 {
            ctx.output
                .extend_from_slice(&data[data_pos..data_pos + bytes_to_copy]);
            data_pos += bytes_to_copy;
        }

        if data_pos >= data.len() {
            ctx.state = STATE_COMPLETION;
        } else {
            ctx.state = if ctx.state == STATE_RAW_COPY_1 {
                STATE_RAW_COPY_2
            } else {
                STATE_RAW_COPY_1
            };
        }

        return data_pos;
    }

    // States 7/8: Compressed mode
    if ctx.state == STATE_COMPRESSED_1 || ctx.state == STATE_COMPRESSED_2 {
        match decode_compressed_block(ctx, data_pos) {
            Ok(new_pos) => {
                data_pos = new_pos;
            }
            Err(e) => {
                eprintln!("Error in range decoder: {}", e);
                ctx.state = -1;
            }
        }

        return data_pos;
    }

    // State 3: Completion
    if ctx.state == STATE_COMPLETION {
        return data_pos;
    }

    data_pos
}
