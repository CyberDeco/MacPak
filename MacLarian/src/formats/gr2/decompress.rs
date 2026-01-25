//! `BitKnit` decompression algorithm for Granny2 files
//!
//! `BitKnit` compression code is adapted from these clean room implementations:
//! - <https://github.com/neptuwunium/Knit/blob/develop/Knit/Compression/GrannyBitKnitCompression.cs>
//! - <https://github.com/eiz/pybg3/blob/9ebda24314822bf35580e74bb7917f666ae046c6/src/rans.h>
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco` (`MacPak`, `PolyForm` Noncommercial), 2025 Legiayayana (Knit, EUPL-1.2), 2024 eiz (pybg3, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation)]

use crate::error::{Error, Result};

// ============================================================================
// Constants
// ============================================================================

/// `BitKnit` magic number (little-endian)
const BITKNIT_MAGIC: u16 = 0x75b1;

/// rANS normalization threshold
const RANS_THRESHOLD: u32 = 0x10000;

/// Quantum size (64KB)
const QUANTUM_SIZE: usize = 0x10000;

// ============================================================================
// Frequency Table
// ============================================================================

struct FrequencyTable {
    frequency_bits: usize,
    vocab_size: usize,
    lookup_shift: usize,
    sums: Vec<u16>,
    lookup: Vec<u16>,
}

impl FrequencyTable {
    fn new(frequency_bits: usize, vocab_size: usize, lookup_bits: usize) -> Self {
        let lookup_shift = frequency_bits - lookup_bits;
        Self {
            frequency_bits,
            vocab_size,
            lookup_shift,
            sums: vec![0; vocab_size + 1],
            lookup: vec![0; 1 << lookup_bits],
        }
    }

    fn find_symbol(&self, code: u32) -> usize {
        let mut sym = self.lookup[(code >> self.lookup_shift) as usize] as usize;
        while code >= u32::from(self.sums[sym + 1]) {
            sym += 1;
        }
        sym
    }

    fn finish_update(&mut self) {
        let mut code = 0usize;
        let mut sym = 0usize;
        let mut next = self.sums[1] as usize;
        let max_code = 1 << self.frequency_bits;
        let step = 1 << self.lookup_shift;

        while code < max_code {
            if code < next {
                self.lookup[code >> self.lookup_shift] = sym as u16;
                code += step;
            } else {
                sym += 1;
                next = self.sums[sym + 1] as usize;
            }
        }
    }

    fn frequency(&self, sym: usize) -> u16 {
        self.sums[sym + 1] - self.sums[sym]
    }

    fn sum_below(&self, sym: usize) -> u16 {
        self.sums[sym]
    }
}

// ============================================================================
// Deferred Adaptive Model
// ============================================================================

struct DeferredAdaptiveModel {
    adaptation_interval: usize,
    frequency_incr: u16,
    last_frequency_incr: u16,
    cdf: FrequencyTable,
    frequency_accumulator: Vec<u16>,
    adaptation_counter: usize,
}

impl DeferredAdaptiveModel {
    fn new(
        adaptation_interval: usize,
        vocab_size: usize,
        num_min_probable_symbols: usize,
        frequency_bits: usize,
        lookup_bits: usize,
    ) -> Self {
        let num_equiprobable_symbols = vocab_size - num_min_probable_symbols;
        let total_sum = 1u16 << frequency_bits;
        let frequency_incr = ((total_sum as usize - vocab_size) / adaptation_interval) as u16;
        let last_frequency_incr =
            (1 + total_sum as usize - vocab_size - frequency_incr as usize * adaptation_interval) as u16;

        let mut cdf = FrequencyTable::new(frequency_bits, vocab_size, lookup_bits);

        // Initialize CDF - equiprobable symbols get equal share
        for i in 0..num_equiprobable_symbols {
            cdf.sums[i] = ((total_sum as usize - num_min_probable_symbols) * i
                / num_equiprobable_symbols) as u16;
        }

        // Min-probable symbols get minimal probability
        for i in num_equiprobable_symbols..=vocab_size {
            cdf.sums[i] = (total_sum as usize - vocab_size + i) as u16;
        }

        let frequency_accumulator = vec![1u16; vocab_size];
        cdf.finish_update();

        Self {
            adaptation_interval,
            frequency_incr,
            last_frequency_incr,
            cdf,
            frequency_accumulator,
            adaptation_counter: 0,
        }
    }

    fn observe_symbol(&mut self, symbol: usize) {
        self.frequency_accumulator[symbol] += self.frequency_incr;
        self.adaptation_counter = (self.adaptation_counter + 1) % self.adaptation_interval;

        if self.adaptation_counter == 0 {
            self.frequency_accumulator[symbol] += self.last_frequency_incr;
            let mut sum: u32 = 0;
            for i in 1..=self.cdf.vocab_size {
                sum += u32::from(self.frequency_accumulator[i - 1]);
                let old = u32::from(self.cdf.sums[i]);
                // C# uses unchecked arithmetic where overflow wraps
                self.cdf.sums[i] = (old.wrapping_add(sum.wrapping_sub(old) / 2)) as u16;
                self.frequency_accumulator[i - 1] = 1;
            }
            self.cdf.finish_update();
        }
    }
}

// ============================================================================
// rANS State
// ============================================================================

struct RANSState {
    bits: u32,
}

impl RANSState {
    fn new() -> Self {
        Self { bits: RANS_THRESHOLD }
    }

    fn with_bits(bits: u32) -> Self {
        Self { bits }
    }

    fn pop_bits(&mut self, stream: &mut BitKnitStream, nbits: usize) -> u32 {
        let sym = self.bits & ((1 << nbits) - 1);
        self.bits >>= nbits;
        self.maybe_refill(stream);
        sym
    }

    fn pop_cdf(&mut self, stream: &mut BitKnitStream, cdf: &FrequencyTable) -> usize {
        let code = self.bits & ((1 << cdf.frequency_bits) - 1);
        let sym = cdf.find_symbol(code);
        let freq = u32::from(cdf.frequency(sym));
        let cumul = u32::from(cdf.sum_below(sym));
        self.bits = (self.bits >> cdf.frequency_bits) * freq + code - cumul;
        self.maybe_refill(stream);
        sym
    }

    fn maybe_refill(&mut self, stream: &mut BitKnitStream) {
        if self.bits < RANS_THRESHOLD {
            self.bits = (self.bits << 16) | u32::from(stream.pop());
        }
    }
}

// ============================================================================
// Stream Reader
// ============================================================================

struct BitKnitStream<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BitKnitStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn pop(&mut self) -> u16 {
        if self.pos + 1 < self.data.len() {
            let val = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;
            val
        } else if self.pos < self.data.len() {
            let val = u16::from(self.data[self.pos]);
            self.pos += 1;
            val
        } else {
            0
        }
    }

    fn peek(&self) -> u16 {
        if self.pos + 1 < self.data.len() {
            u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]])
        } else if self.pos < self.data.len() {
            u16::from(self.data[self.pos])
        } else {
            0
        }
    }

    fn remaining_bytes(&self) -> &[u8] {
        &self.data[self.pos..]
    }

    fn slide(&mut self, n: usize) {
        self.pos += n;
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }
}

// ============================================================================
// Register LRU Cache
// ============================================================================

struct RegisterLRUCache {
    entries: [u32; 8],
    entry_order: u32,
}

impl RegisterLRUCache {
    fn new() -> Self {
        Self {
            entries: [1; 8],
            entry_order: 0x76543210,
        }
    }

    fn insert(&mut self, value: u32) {
        let idx7 = ((self.entry_order >> 28) & 0xF) as usize;
        let idx6 = ((self.entry_order >> 24) & 0xF) as usize;
        self.entries[idx7] = self.entries[idx6];
        self.entries[idx6] = value;
    }

    fn hit(&mut self, index: usize) -> u32 {
        let slot = ((self.entry_order >> (index * 4)) & 0xF) as usize;
        // C# uses unchecked arithmetic where overflow wraps. Match that behavior.
        let rotate_mask = 16u32.wrapping_shl((index * 4) as u32).wrapping_sub(1);
        let rotated_order = ((self.entry_order << 4) | slot as u32) & rotate_mask;
        self.entry_order = (self.entry_order & !rotate_mask) | rotated_order;
        self.entries[slot]
    }
}

// ============================================================================
// BitKnit Decompressor
// ============================================================================

struct Bitknit2State {
    output: Vec<u8>,
    index: usize,
    command_models: [DeferredAdaptiveModel; 4],
    cache_reference_models: [DeferredAdaptiveModel; 4],
    copy_offset_model: DeferredAdaptiveModel,
    copy_offset_cache: RegisterLRUCache,
    delta_offset: usize,
}

impl Bitknit2State {
    fn new(expected_size: usize) -> Self {
        Self {
            output: vec![0u8; expected_size],
            index: 0,
            command_models: [
                DeferredAdaptiveModel::new(1024, 300, 36, 15, 10),
                DeferredAdaptiveModel::new(1024, 300, 36, 15, 10),
                DeferredAdaptiveModel::new(1024, 300, 36, 15, 10),
                DeferredAdaptiveModel::new(1024, 300, 36, 15, 10),
            ],
            cache_reference_models: [
                DeferredAdaptiveModel::new(1024, 40, 0, 15, 10),
                DeferredAdaptiveModel::new(1024, 40, 0, 15, 10),
                DeferredAdaptiveModel::new(1024, 40, 0, 15, 10),
                DeferredAdaptiveModel::new(1024, 40, 0, 15, 10),
            ],
            copy_offset_model: DeferredAdaptiveModel::new(1024, 21, 0, 15, 10),
            copy_offset_cache: RegisterLRUCache::new(),
            delta_offset: 1,
        }
    }

    fn decode(&mut self, data: &[u8]) -> Result<()> {
        let mut stream = BitKnitStream::new(data);

        // Check magic
        if stream.pop() != BITKNIT_MAGIC {
            return Err(Error::DecompressionError("Invalid BitKnit magic".to_string()));
        }

        while self.index < self.output.len() {
            if stream.is_empty() {
                return Err(Error::UnexpectedEof);
            }
            self.decode_quantum(&mut stream)?;
        }

        Ok(())
    }

    fn decode_quantum(&mut self, stream: &mut BitKnitStream) -> Result<()> {
        // Process up to next 64KB boundary
        let boundary = (self.index & 0xFFFF0000) + QUANTUM_SIZE;
        let boundary = boundary.min(self.output.len());

        // Check for uncompressed quantum
        if stream.peek() == 0 {
            stream.pop();
            let remaining = stream.remaining_bytes();
            let copy_length = remaining.len().min(boundary - self.index);
            self.output[self.index..self.index + copy_length]
                .copy_from_slice(&remaining[..copy_length]);
            self.index += copy_length;
            stream.slide(copy_length);
            return Ok(());
        }

        let mut state1 = RANSState::new();
        let mut state2 = RANSState::new();
        self.decode_initial_state(stream, &mut state1, &mut state2);

        // First byte of first quantum
        if self.index == 0 {
            let first_byte = self.pop_bits(stream, 8, &mut state1, &mut state2);
            self.output[self.index] = first_byte as u8;
            self.index += 1;
        }

        while self.index < boundary {
            let model_index = self.index % 4;

            let command = self.pop_model(stream, model_index, &mut state1, &mut state2);

            if command >= 256 {
                self.decode_copy(stream, command, &mut state1, &mut state2)?;
            } else {
                // Literal with delta
                let delta_byte = if self.index >= self.delta_offset {
                    self.output[self.index - self.delta_offset]
                } else {
                    0
                };
                self.output[self.index] = (command as u8).wrapping_add(delta_byte);
                self.index += 1;
            }
        }

        Ok(())
    }

    fn decode_copy(
        &mut self,
        stream: &mut BitKnitStream,
        command: usize,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) -> Result<()> {
        let model_index = self.index % 4;

        let copy_length = if command < 288 {
            command - 254
        } else {
            let copy_length_length = command - 287;
            let copy_length_bits = self.pop_bits(stream, copy_length_length, state1, state2);
            (1 << copy_length_length) + copy_length_bits as usize + 32
        };

        let cache_ref = self.pop_cache_model(stream, model_index, state1, state2);

        let copy_offset = if cache_ref < 8 {
            self.copy_offset_cache.hit(cache_ref)
        } else {
            let copy_offset_length = self.pop_offset_model(stream, state1, state2);
            let copy_offset_bits =
                self.pop_bits(stream, copy_offset_length % 16, state1, state2);

            let copy_offset_bits = if copy_offset_length >= 16 {
                (copy_offset_bits << 16) | u32::from(stream.pop())
            } else {
                copy_offset_bits
            };

            let offset =
                (32u32 << copy_offset_length) + (copy_offset_bits << 5) - 32 + (cache_ref as u32 - 7);
            self.copy_offset_cache.insert(offset);
            offset
        };

        self.delta_offset = copy_offset as usize;

        // Validate copy offset
        if copy_offset as usize > self.index {
            let index = self.index;
            return Err(crate::error::Error::DecompressionError(
                format!("Copy offset {copy_offset} exceeds current position {index}")
            ));
        }

        for _ in 0..copy_length {
            if self.index >= self.output.len() {
                break;
            }
            let copy_pos = self.index - copy_offset as usize;
            self.output[self.index] = self.output[copy_pos];
            self.index += 1;
        }

        Ok(())
    }

    fn pop_bits(
        &self,
        stream: &mut BitKnitStream,
        nbits: usize,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) -> u32 {
        let result = state1.pop_bits(stream, nbits);
        std::mem::swap(&mut state1.bits, &mut state2.bits);
        result
    }

    fn pop_model(
        &mut self,
        stream: &mut BitKnitStream,
        model_index: usize,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) -> usize {
        let result = state1.pop_cdf(stream, &self.command_models[model_index].cdf);
        self.command_models[model_index].observe_symbol(result);
        std::mem::swap(&mut state1.bits, &mut state2.bits);
        result
    }

    fn pop_cache_model(
        &mut self,
        stream: &mut BitKnitStream,
        model_index: usize,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) -> usize {
        let result = state1.pop_cdf(stream, &self.cache_reference_models[model_index].cdf);
        self.cache_reference_models[model_index].observe_symbol(result);
        std::mem::swap(&mut state1.bits, &mut state2.bits);
        result
    }

    fn pop_offset_model(
        &mut self,
        stream: &mut BitKnitStream,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) -> usize {
        let result = state1.pop_cdf(stream, &self.copy_offset_model.cdf);
        self.copy_offset_model.observe_symbol(result);
        std::mem::swap(&mut state1.bits, &mut state2.bits);
        result
    }

    fn decode_initial_state(
        &self,
        stream: &mut BitKnitStream,
        state1: &mut RANSState,
        state2: &mut RANSState,
    ) {
        let init_0 = stream.pop();
        let init_1 = stream.pop();
        let mut merged = RANSState::with_bits((u32::from(init_0) << 16) | u32::from(init_1));

        let split = merged.pop_bits(stream, 4) as usize;
        state1.bits = merged.bits >> split;
        state1.maybe_refill(stream);
        state2.bits = (merged.bits << 16) | u32::from(stream.pop());
        state2.bits &= (1 << (16 + split)) - 1;
        state2.bits |= 1 << (16 + split);
    }
}

/// Decompress data using Granny2 `BitKnit` (format 4)
///
/// # Errors
/// Returns an error if decompression fails.
pub fn decompress_bitknit(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    let mut state = Bitknit2State::new(expected_size);
    state.decode(compressed)?;
    Ok(state.output)
}
