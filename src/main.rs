use ::brotli2::bufread::BrotliEncoder;
use ::colored::Colorize;
use ::rand::{distributions::Standard, seq::SliceRandom, thread_rng, Rng};
use std::io::Read;

fn higher_order_entropy(in_array: &[u8], verbose: bool) -> f32 {
    // higher_order_entropy = shannon_entropy - kolmogorov_complexity (estimate)
    let genome_len = in_array.len();
    // Calculate shannon_entropy O(N+M), where N=genome_len, M=u8::MAX
    let mut counts = [0; 256];
    for byte in in_array {
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
    let compressed_len = BrotliEncoder::new(in_array, 2).bytes().count();
    let kolmogorov_complexity = (compressed_len as f32 / genome_len as f32) * 8.0;
    if verbose {
        println!(
            "shannon: {:?} kolmogorov:{:?}",
            &shannon_entropy, &kolmogorov_complexity
        );
    }
    // Combine to get "higher_order_entropy"
    shannon_entropy - kolmogorov_complexity
}

struct BFFRun {
    tape: Vec<u8>,
    head0_pos: usize,
    head1_pos: usize,
    pc_pos: usize,
    max_iterations: usize,
    iteration: usize,
    skipped: usize,
    state: String,
}
impl BFFRun {
    fn from_vec(input_tape: Vec<u8>) -> Self {
        BFFRun {
            tape: input_tape,
            head0_pos: 0,
            head1_pos: PROGRAM_SIZE,
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
                        if self.pc_pos == tape_len {
                            self.state = "Error, unmatched `[`".to_string();
                            break;
                        }
                        let mut diff = 1;
                        for ind in (self.pc_pos + 1)..=(tape_len - 1) {
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
                        if self.pc_pos == 0 {
                            self.state = "Error, unmatched `]`".to_string();
                            break;
                        }

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
        if verbose {
            println!(
                "{:?}, iteration: {:?}, skipped: {:?}",
                self.state, self.iteration, self.skipped
            );
        }

        BFFRun {
            tape: self.tape.clone(),
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

const POPULATION_SIZE: usize = 16384;
const PROGRAM_SIZE: usize = 64;
const MAX_EPOCHS: usize = 100000;
const REPORT_INTERVAL: usize = 100;

fn main() {
    // Create soup of programs
    let mut soup: Vec<u8> = thread_rng()
        .sample_iter(&Standard)
        .take(PROGRAM_SIZE * POPULATION_SIZE)
        .collect();

    // // Example replicator run
    // let mut bff_program = BFFRun::from_vec(
    //     "[[{.>]-]                ]-]>.{[[00000000000000000000000000000000"
    //         .as_bytes()
    //         .to_vec(),
    // );
    // bff_program.head1_pos = 0;
    // bff_program = bff_program.emulate(true);
    // println!("{:?}", higher_order_entropy(&bff_program.tape, false))

    // println!("{}", higher_order_entropy(&[0u8; 256], true));

    // Run the full search
    let mut epoch = 0;
    while epoch < MAX_EPOCHS {
        // Sample random pairs from the population
        let mut random_order: Vec<usize> = (0..POPULATION_SIZE).collect();
        random_order.shuffle(&mut thread_rng());

        // Emulate pairs and replace (ideally in parallel)
        for ind_chunk in random_order.chunks(2) {
            let ind1 = ind_chunk[0];
            let ind2 = ind_chunk[1];
            // Create tape to run through BFF emulator
            let mut tape = Vec::with_capacity(PROGRAM_SIZE * 2);
            for val in ind1 * PROGRAM_SIZE..(ind1 + 1) * PROGRAM_SIZE {
                tape.push(soup[val])
            }
            for val in ind2 * PROGRAM_SIZE..(ind2 + 1) * PROGRAM_SIZE {
                tape.push(soup[val])
            }

            // Emulate program
            let mut bff_program = BFFRun::from_vec(tape);
            bff_program = bff_program.emulate(false);

            // Insert back into soup (for some reason I can't use slices)
            let i1_start = ind1 * PROGRAM_SIZE;
            let i2_start = ind2 * PROGRAM_SIZE;
            for offset in 0..PROGRAM_SIZE {
                soup[i1_start + offset] = bff_program.tape[offset];
                soup[i2_start + offset] = bff_program.tape[offset + PROGRAM_SIZE];
            }
        }

        // Report epoch metrics
        if epoch % REPORT_INTERVAL == 0 {
            let hoe = higher_order_entropy(&soup, true);
            println!("Epoch {:4<0}: Higher-Order Entropy={}", &epoch, hoe);
            for ind in 0..5 {
                println!(
                    "\t\t{}",
                    String::from_utf8_lossy(&soup[ind * PROGRAM_SIZE..(ind + 1) * PROGRAM_SIZE])
                )
            }
        }

        epoch += 1;
    }
}
