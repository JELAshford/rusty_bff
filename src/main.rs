use ::colored::Colorize;
use ::rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use std::collections::HashMap;

fn compress(data: &[u8]) -> Vec<u32> {
    // Borrowed this compression algorithm from @LukasDeco because the
    // package couldn't be found on cargo!
    //https://github.com/LukasDeco/lzw-compress/blob/main/src/lib.rs

    // Build the initial dictionary with single-byte sequences.
    let mut dictionary: HashMap<Vec<u8>, u32> = (0u32..=255).map(|i| (vec![i as u8], i)).collect();

    let mut current_sequence = Vec::new();
    let mut compressed = Vec::new();

    for &byte in data {
        // Try to extend the current sequence.
        current_sequence.push(byte);

        // Check if the extended sequence is in the dictionary.
        if let Some(&_code) = dictionary.get(&current_sequence) {
            // The sequence is in the dictionary, continue to extend it.
            continue;
        }

        // The sequence is not in the dictionary, so add it.
        // Write the code for the current sequence to the output.
        if let Some(&code) = dictionary.get(&current_sequence[..current_sequence.len() - 1]) {
            compressed.push(code);
        } else {
            panic!("Invalid dictionary state.");
        }

        // Add the new sequence to the dictionary.
        dictionary.insert(current_sequence.clone(), dictionary.len() as u32);

        // Reset the current sequence to the last byte.
        current_sequence.clear();
        current_sequence.push(byte);
    }

    // Write the code for the last sequence to the output.
    if let Some(&code) = dictionary.get(&current_sequence) {
        compressed.push(code);
    } else {
        panic!("Invalid dictionary state.");
    }

    compressed
}
#[derive(Debug)]
struct Program {
    genome: Vec<u8>,
}
impl Program {
    fn random(rng: &mut ThreadRng) -> Self {
        let genome: [u8; 32] = rng.gen(); // array construction
        Program {
            genome: genome.to_vec(),
        }
    }
    fn from_string(string_in: String) -> Self {
        Program {
            genome: string_in.into_bytes(),
        }
    }
    fn higher_order_entropy(self) -> f32 {
        // higher_order_entropy = shannon_entropy - kolmogorov_complexity (estimate)
        let genome_len = self.genome.len();
        // Calculate shannon_entropy O(N+M), where N=genome_len, M=u8::MAX
        let mut counts = [0; 255];
        for byte in &self.genome {
            counts[*byte as usize] += 1;
        }
        let mut shannon_entropy = 0.;
        for count in counts {
            if count > 0 {
                let freq = count as f32 / genome_len as f32;
                shannon_entropy += freq * freq.log2();
            }
        }
        shannon_entropy *= -1.;
        // Approximate kolmogorov complexity by measuring compression
        let kolmogorov_complexity =
            (compress(&self.genome.as_slice()).len() as f32 / genome_len as f32) * 8.0;
        println!(
            "shannon: {:?} kolmogorov:{:?}",
            &shannon_entropy, &kolmogorov_complexity
        );
        // Combine to get "higher_order_entropy"
        shannon_entropy - kolmogorov_complexity
    }
}

struct BFFProgram {
    tape: Vec<u8>,
    head0_pos: usize,
    head1_pos: usize,
    pc_pos: usize,
    max_iterations: usize,
    iteration: usize,
    skipped: usize,
    state: String,
}
impl BFFProgram {
    fn from_string(input_string: String) -> Self {
        BFFProgram {
            tape: input_string.into_bytes(),
            head0_pos: 0,
            head1_pos: 0,
            pc_pos: 0,
            max_iterations: 128,
            iteration: 0,
            skipped: 0,
            state: "Terminated".to_string(),
        }
    }
    fn print_tape(&self) -> () {
        let base_vis_string = String::from_utf8(self.tape.clone()).expect("Invalid UTF-8");
        print!("Iteration {:0>4}:\t\t", &self.iteration);
        for (ind, char) in base_vis_string.chars().enumerate() {
            if ind == self.head0_pos {
                print!("{}", char.to_string().on_blue());
            } else if ind == self.head1_pos {
                print!("{}", char.to_string().on_red());
            } else if ind == self.pc_pos {
                print!("{}", char.to_string().on_green());
            } else {
                print!("{}", char);
            }
        }
        println!("");
    }
    fn emulate(mut self, verbose: bool) -> Self {
        let tape_len = self.tape.len();
        while self.iteration < self.max_iterations {
            let instruction = self.tape[self.pc_pos];
            match instruction {
                b'<' => self.head0_pos = (self.head0_pos - 1) % tape_len,
                b'>' => self.head0_pos = (self.head0_pos + 1) % tape_len,
                b'{' => self.head1_pos = (self.head1_pos - 1) % tape_len,
                b'}' => self.head1_pos = (self.head1_pos + 1) % tape_len,
                b'-' => self.tape[self.head0_pos] = (self.tape[self.head0_pos] - 1) % 255,
                b'+' => self.tape[self.head0_pos] = (self.tape[self.head0_pos] + 1) % 255,
                b'.' => self.tape[self.head1_pos] = self.tape[self.head0_pos],
                b',' => self.tape[self.head0_pos] = self.tape[self.head1_pos],
                b'[' => {
                    if self.tape[self.head0_pos] == b'0' {
                        let mut diff = 1;
                        for ind in (self.pc_pos + 1)..=tape_len {
                            if self.tape[ind] == b'[' {
                                diff += 1;
                            } else if self.tape[ind] == b']' {
                                diff -= 1;
                            }
                            if diff == 0 {
                                self.pc_pos = ind;
                                break;
                            }
                        }
                        if diff != 0 {
                            self.state = "Error, unmatched `[`".to_string();
                            break;
                        }
                    }
                }
                b']' => {
                    if self.tape[self.head0_pos] != b'0' {
                        let mut diff = 1;
                        for ind in (0..=(self.pc_pos - 1)).rev() {
                            if self.tape[ind] == b']' {
                                diff += 1;
                            } else if self.tape[ind] == b'[' {
                                diff -= 1;
                            }
                            if diff == 0 {
                                self.pc_pos = ind;
                                break;
                            }
                        }
                        if diff != 0 {
                            self.state = "Error, unmatched `]`".to_string();
                            break;
                        }
                    }
                }
                _ => self.skipped += 1,
            }

            // Show the current tape, coloured with head/pc positions
            if verbose {
                self.print_tape();
            }

            self.iteration += 1;
            self.pc_pos += 1;
            if self.pc_pos >= tape_len {
                self.state = "Finished".to_string();
                break;
            }
        }
        println!(
            "{:?}, iteration: {:?}, skipped: {:?}",
            self.state, self.iteration, self.skipped
        );

        BFFProgram {
            tape: self.tape,
            head0_pos: self.head0_pos,
            head1_pos: self.head1_pos,
            pc_pos: self.pc_pos,
            max_iterations: self.max_iterations,
            iteration: self.iteration,
            skipped: self.skipped,
            state: self.state,
        }
    }
}

fn main() {
    // Create soup of programs
    let mut rng = thread_rng();
    let _soup: Vec<Program> = (0..10).map(|_| Program::random(&mut rng)).collect();

    // // Choose two at random
    // let g1 = soup
    //     .choose(&mut rng)
    //     .expect("Could not choose program from soup");
    // let g2 = soup
    //     .choose(&mut rng)
    //     .expect("Could not choose program from soup");
    // println!("{:?}", &g1);
    // println!("{:?}", &g2);

    // Choose two programs
    let g1 = Program::from_string("[[{.>]-]                ]-]>.{[[".to_string());
    let g2 = Program::from_string("00000000000000000000000000000000".to_string());

    // Combine and run emulation
    let tape = [g1.genome, g2.genome].concat().clone();
    let mut bff_program = BFFProgram::from_string(String::from_utf8(tape).expect("Invalif UTF-8"));
    bff_program = bff_program.emulate(false);
    println!(
        "{}",
        Program {
            genome: bff_program.tape
        }
        .higher_order_entropy()
    );
}
