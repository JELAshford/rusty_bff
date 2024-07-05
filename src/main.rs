use rand::{seq::SliceRandom, thread_rng};

struct Program {
    genome: [u8; 32],
}
impl Program {
    fn random() -> Self {
        let character_set = "<>{}-+/.[] ".as_bytes();
        let random_set: [u8; 32] = character_set
            .choose_multiple(&mut thread_rng(), 32)
            .map(|b| *b)
            .try_into()
            .expect("error generating string");
        // .collect();
        Program { genome: random_set }
    }
}

fn main() {
    let soup: Vec<Program> = (0..2048).map(|_| Program::random()).collect();
    println!("Hello, world!");
}
