#![feature(generic_const_exprs)]

use core::iter::zip;
use core::mem::{forget, MaybeUninit};
use core::ptr::copy_nonoverlapping;
use realfft::RealFftPlanner;

type Float = f32;

#[rustfmt::skip]
const SIX_STAR_RATES: [Float; 99] = [
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.04, 0.06, 0.08, 0.10, 0.12, 0.14, 0.16, 0.18, 0.20, 0.22,
    0.24, 0.26, 0.28, 0.30, 0.32, 0.34, 0.36, 0.38, 0.40, 0.42,
    0.44, 0.46, 0.48, 0.50, 0.52, 0.54, 0.56, 0.58, 0.60, 0.62,
    0.64, 0.66, 0.68, 0.70, 0.72, 0.74, 0.76, 0.78, 0.80, 0.82,
    0.84, 0.86, 0.88, 0.90, 0.92, 0.94, 0.96, 0.98, 1.00,
];

fn banner<const TARGET: usize, const PULLS: usize>(subrate: Float) -> [Float; PULLS]
where
    [(); TARGET + 1]:, // generic const exprs need a `where` bound
{
    let mut pdist = [0.; PULLS];

    let mut probs = [[0.; 100]; { TARGET + 1 }];
    probs[0][0] = 1.;

    let mut temp_probs = [[0.; 100]; { TARGET + 1 }];

    for prob in &mut pdist {
        for (pity_count, rate) in SIX_STAR_RATES.iter().enumerate() {
            for target_count in 0..TARGET {
                let prob = probs[target_count][pity_count];

                temp_probs[target_count][pity_count + 1] += prob * (1. - rate);
                temp_probs[target_count][0] += prob * rate * (1. - subrate);
                temp_probs[target_count + 1][0] += prob * rate * subrate;
            }
        }

        probs = temp_probs;
        *prob = probs[TARGET][0];
        temp_probs = [[0.; 100]; { TARGET + 1 }];
    }

    pdist
}

#[allow(clippy::ptr_as_ptr)]
fn concat<T, const M: usize, const N: usize>(a: [T; M], b: [T; N]) -> [T; M + N] {
    let mut result = MaybeUninit::uninit();
    let dest = result.as_mut_ptr() as *mut T;
    unsafe {
        copy_nonoverlapping(a.as_ptr(), dest, M);
        copy_nonoverlapping(b.as_ptr(), dest.add(M), N);
        forget(a);
        forget(b);
        result.assume_init()
    }
}

#[allow(clippy::cast_precision_loss, clippy::similar_names)]
fn main() {
    const PULLS: usize = 170;
    const FREE_PULLS: usize = 24;
    // sum of the max possible # of pulls on each banner
    const CONV_SIZE: usize = (PULLS + FREE_PULLS - 1) + (PULLS - 1);

    // arrays are padded with 0s to calculate a "full" convolution
    let mut pdist_1 = concat(
        banner::<1, { PULLS + FREE_PULLS - 1 }>(0.35),
        [0.; { PULLS - 1 }],
    );
    let mut pdist_2 = concat(
        banner::<1, { PULLS - 1 }>(0.5),
        [0.; { PULLS + FREE_PULLS - 1 }],
    );

    // initialize FFT calculator
    let mut planner = RealFftPlanner::<Float>::new();
    let fft = planner.plan_fft_forward(CONV_SIZE);

    // apply FFT to probability distributions
    let mut dft_1 = fft.make_output_vec();
    fft.process(&mut pdist_1, &mut dft_1).unwrap();

    let mut dft_2 = fft.make_output_vec();
    fft.process(&mut pdist_2, &mut dft_2).unwrap();

    // multiply the DFTs together
    let mut combined_dft: Vec<_> = zip(dft_1, dft_2).map(|(a, b)| a * b).collect();

    // apply IFFT to get the convolved distribution
    // result needs to be divided by CONV_SIZE to get the actual convolution values
    let ifft = planner.plan_fft_inverse(CONV_SIZE);
    let mut combined_seq = ifft.make_output_vec();
    ifft.process(&mut combined_dft, &mut combined_seq).unwrap();

    // calculate final probability
    let prob = combined_seq
        .into_iter()
        .take(PULLS + FREE_PULLS - 1) // up to the max possible total # of pulls
        .sum::<Float>()
        / (CONV_SIZE as Float);

    println!("Probability: {prob}");
}
