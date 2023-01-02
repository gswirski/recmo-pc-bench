use log::Level;
use plonky2::{
    field::{
        fft::fft_root_table, goldilocks_field::GoldilocksField as F, polynomial::PolynomialValues,
        types::Sample,
    },
    fri::oracle::PolynomialBatch,
    plonk::config::{KeccakGoldilocksConfig, PoseidonGoldilocksConfig},
    util::timing::TimingTree,
};
use rand::{thread_rng, Fill, Rng};
use rayon::prelude::*;
use std::{
    mem::size_of,
    time::{Duration, Instant},
};

// type C = PoseidonGoldilocksConfig;
type C = KeccakGoldilocksConfig;
const D: usize = 2;
const RATE_BITS: usize = 1;

const MAX_EXPONENT: u32 = 32;
const MAX_SIZE: usize = 1 << MAX_EXPONENT;

pub fn rand_vec(size: usize) -> Vec<F> {
    let now = Instant::now();
    println!(
        "Memory allocation ({} GB)",
        (size * size_of::<F>()) as f64 / 1.0e9
    );
    let mut result = vec![F::default(); size];
    println!("Randomizing...");
    result.par_chunks_mut(1024).for_each_init(
        || thread_rng(),
        |rng, chunk| {
            for point in chunk {
                *point = F::sample(&mut *rng);
            }
        },
    );
    println!("Random generation took: {:?}", now.elapsed());
    result
}

fn bench(input: &[F]) -> f64 {
    let mut count = 0;
    let mut duration = 0.0;

    let mut timing = TimingTree::new("bench", Level::Debug);

    let root_table = fft_root_table(input.len() << RATE_BITS);

    // Plonky2 takes care of parallelization.
    loop {
        count += 1;
        let now = Instant::now();

        let input = input.to_vec();
        let _ = PolynomialBatch::<F, C, D>::from_values(
            vec![PolynomialValues::new(input)],
            RATE_BITS,
            false,
            0,
            &mut timing,
            Some(&root_table),
        );

        duration += now.elapsed().as_secs_f64();
        if duration > 5.0 {
            break;
        }
    }
    duration / count as f64
}

fn main() {
    println!("Preparing input...");
    let input = rand_vec(MAX_SIZE);

    println!("size,duration,throughput");

    for i in 10..=MAX_EXPONENT {
        let size = 1_usize << i;
        let duration = bench(&input[..size]);
        let throughput = size as f64 / duration;
        println!("{size},{duration},{throughput}");
    }
}
