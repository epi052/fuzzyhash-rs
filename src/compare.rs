use std::cmp::{min,max};
use constants;
use roll::{Roll};

const MAX_LENGTH: usize = 64;
const INSERT_COST: u32 = 1;
const REMOVE_COST: u32 = 1;
const REPLACE_COST: u32 = 2;

fn compute_distance(s1: Vec<u8>, s2: Vec<u8>) -> u32 {
    let mut t1: Vec<u32> = vec![0; MAX_LENGTH + 1];
    let mut t2: Vec<u32> = vec![0; MAX_LENGTH + 1];
    
    for i2 in 0..s2.len()+1 {
        t1[i2] = i2 as u32 * REMOVE_COST;
    }
    for i1 in 0..s1.len() {
        t2[0] = (i1 as u32 + 1) * INSERT_COST;
        for i2 in 0..s2.len() {
            let cost_a = t1[i2+1] + INSERT_COST;
            let cost_d = t2[i2] + REMOVE_COST;
            let cost_r = t1[i2] + if s1[i1] == s2[i2] {
                0
            } else {
                REPLACE_COST
            };
            t2[i2+1] = min(min(cost_a, cost_d), cost_r);
        }
        let t3 = t1.clone();
        t1 = t2.clone();
        t2 = t3;
    }
    t1[s2.len()]
}

fn has_common_substring(first: &[u8], second: &[u8]) -> bool {
    let first_length = first.to_vec().len();
    let second_length = second.to_vec().len();
    let mut i: usize = 0;
    let mut hashes: Vec<u32> = vec![0; constants::SPAM_SUM_LENGTH as usize];
    let mut state = Roll::new();

    while i < first_length && first[i] != 0 {
        state.hash(first[i]);
        hashes[i] = state.sum();
        i += 1;
    }

    let num_hashes = i;
    state = Roll::new();

    i = 0;
    while i < second_length && second[i] != 0 {
        state.hash(second[i]);
        let h = state.sum();

        if i < constants::ROLLING_WINDOW - 1 {
            i += 1;
            continue;
        }

        for j in (constants::ROLLING_WINDOW - 1)..num_hashes {
            if hashes[j] != 0 && hashes[j] == h {
                let second_start_pos = i.wrapping_sub(constants::ROLLING_WINDOW).wrapping_add(1);
                let mut len = 0;
                while len + second_start_pos < second_length &&
                    second[len + second_start_pos] != 0 {
                        len += 1;
                    }
                
                if len < constants::ROLLING_WINDOW {
                    continue;
                }

                let mut matched = true;
                let first_start_pos = j.wrapping_sub(constants::ROLLING_WINDOW).wrapping_add(1);
                for pos in 0..constants::ROLLING_WINDOW {
                    let first_char = first[first_start_pos + pos];
                    let second_char = second[second_start_pos + pos];

                    if first_char != second_char {
                        matched = false;
                        break;
                    }

                    if first_char == 0 {
                        break;
                    }
                }
                if matched {
                    return true;
                }
            }
        }
        i += 1;
    }
    false 
}

fn eliminate_sequences(input: Vec<u8>) -> Vec<u8> {
    let mut result: Vec<u8> = vec![0; input.len()];
    let mut i = 0;

    while i < 3 && i < input.len() {
        result[i] = input[i];
        i += 1;
    }

    if input.len() < 3 {
        return result
    }

    i = 3;
    let mut j = 3;

    while i < input.len() {
        let current = input[j];
        if current != input[i - 1] || current != input[i - 2] || current != input[i - 3] {
            result[j] = input[i];
            j += 1;
        }
        i += 1;
    }

    unsafe {
        result.set_len(j);
    }
    result
}

pub fn score_strings(first: Vec<u8>, second: Vec<u8>, block_size: u32) -> u32 {

    if first.len() > constants::SPAM_SUM_LENGTH as usize || 
        second.len() > constants::SPAM_SUM_LENGTH as usize {
        return 0;
    }

    if !has_common_substring(&first, &second) {
        println!("no common substring!");
        return 0;
    }

    let mut score = compute_distance(first.clone(), second.clone());
    score = (score * constants::SPAM_SUM_LENGTH) / (( first.len() + second.len() ) as u32);
    score = (100 * score) / 64;
    if score >= 100 {
        return 0;
    }

    score = 100 - score;
    
    let match_size = block_size / constants::MIN_BLOCK_SIZE * (min(first.len(), second.len()) as u32);
    if score > match_size {
        match_size
    } else {
         score
    }
}

pub fn strings(first: String, second: String) -> u32 {
    let first_parts: Vec<&str> = first.split(':').collect();
    let second_parts: Vec<&str> = second.split(':').collect();

    if first_parts.len() != 3 && second_parts.len() != 3 {
        println!("Badly formatted input strings!");
        return 0;
    }

    let first_block_size = match first_parts[0].parse::<u32>() {
        Ok(s) => s,
        Err(_) => {
            println!("Cannot parse first string's block size!");
            0
        }
    };
    let second_block_size = match second_parts[0].parse::<u32>() {
        Ok(s) => s,
        Err(_) => {
            println!("Cannot parse second string's block size!");
            0
        }
    };

    if first_block_size != second_block_size &&
        first_block_size != second_block_size * 2 &&
            second_block_size != first_block_size * 2 {
                println!("Incompatible block sizes!");
                return 0
            }

    let first_block1 = eliminate_sequences(first_parts[1].as_bytes().to_vec());
    let first_block2 = eliminate_sequences(first_parts[2].as_bytes().to_vec());
    
    let second_block1 = eliminate_sequences(second_parts[1].as_bytes().to_vec());
    let second_block2 = eliminate_sequences(second_parts[2].as_bytes().to_vec());

    if first_block_size == second_block_size && first_block1.len() == second_block1.len() {
        let mut matched = true;
        for i in 0..first_block1.len() {
            if first_block1[i] != second_block1[i] {
                matched = false;
                break;
            }
        }
        if matched {
            return 100;
        }
    }

    if first_block_size == second_block_size {
        let score1 = score_strings(first_block1, second_block1, first_block_size);
        let score2 = score_strings(first_block2, second_block2, first_block_size * 2);
        return max(score1, score2);
    }
    else if first_block_size == second_block_size * 2 {
        return score_strings(first_block1, second_block2, first_block_size);
    }
    else {
        return score_strings(first_block2, second_block1, second_block_size);
    }
}
