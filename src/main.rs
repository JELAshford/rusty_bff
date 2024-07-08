use ::rand::{thread_rng, Rng};
use rand::{rngs::ThreadRng, seq::SliceRandom};

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
    println!("{:?}", &g1);
    println!("{:?}", &g2);

    // Combine and run emulation
    let mut tape = [g1.genome, g2.genome].concat().clone();
    let tape_len = tape.len();

    let mut head0_pos = 0;
    let mut head1_pos = 0;
    let mut pc_pos = 0;

    let max_iterations = 128;
    let mut iteration = 0;
    let mut skipped = 0;

    let mut state = "Terminated";
    while iteration < max_iterations {
        let instruction = tape[pc_pos];
        match instruction {
            b'<' => head0_pos = (head0_pos - 1) % tape_len,
            b'>' => head0_pos = (head0_pos + 1) % tape_len,
            b'{' => head1_pos = (head1_pos - 1) % tape_len,
            b'}' => head1_pos = (head1_pos + 1) % tape_len,
            b'-' => tape[head0_pos] = (tape[head0_pos] - 1) % 255,
            b'+' => tape[head0_pos] = (tape[head0_pos] + 1) % 255,
            b'.' => tape[head1_pos] = tape[head0_pos],
            b',' => tape[head0_pos] = tape[head1_pos],
            b'[' => {
                if tape[head0_pos] == b'0' {
                    let mut diff = 1;
                    for ind in (pc_pos + 1)..tape_len {
                        if tape[ind] == b'[' {
                            diff += 1;
                        } else if tape[ind] == b']' {
                            diff -= 1;
                        }
                        if diff == 0 {
                            pc_pos = ind;
                            break;
                        }
                    }
                    if diff != 0 {
                        state = "Error, unmatched `[`";
                        break;
                    }
                }
            }
            b']' => {
                if tape[head0_pos] != b'0' {
                    let mut diff = 1;
                    for ind in (0..pc_pos).rev() {
                        if tape[ind] == b']' {
                            diff += 1;
                        } else if tape[ind] == b'[' {
                            diff -= 1;
                        }
                        if diff == 0 {
                            pc_pos = ind;
                            break;
                        }
                    }
                    if diff != 0 {
                        state = "Error, unmatched `]`";
                        break;
                    }
                }
            }
            _ => skipped += 1,
        }
        println!(
            "{:?}",
            String::from_utf8(tape.clone()).expect("Invalid UTF-8")
        );

        iteration += 1;
        pc_pos += 1;
        if pc_pos >= tape_len {
            state = "Finished";
            break;
        }
    }
    println!(
        "{:?}, iteration: {:?}, skipped: {:?}",
        state, iteration, skipped
    );
}
