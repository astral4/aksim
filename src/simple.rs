use crate::Float;
use core::iter::zip;
use core::mem::swap;
use realfft::num_complex::Complex;
use realfft::RealFftPlanner;

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

// Computes the probability distribution (up to `pulls`) of achieving the `target` of a banner.
fn banner_pdist(target: usize, pulls: usize, subrate: Float) -> Vec<Float> {
    // We calculate the probability distribution by keeping track of a 2D matrix of states.
    // One axis is for the target count and one axis is for the pity count.
    // Each element of the matrix represents a "state" that we could be in while pulling
    // and is the probability of being in that state.
    // Every pull, we update the matrix by looking at each state and updating the probabilities of possible next states.
    // For example, the probability of having 2 of the target and being at 38 pity is at probs[2][38].
    // From probs[2][38], we could advance to:
    // - probs[2][39] (didn't pull a 6*, increasing pity by 1),
    // - probs[2][0] (pulled an off-rate 6*, resetting pity to 0), or
    // - probs[3][0] (pulled a target 6*, increasing the target count by 1 and resetting pity to 0).

    let mut pdist = Vec::with_capacity(pulls);

    // No pulls have been done yet, so having a target and pity count of 0 is certain (probability of 1).
    // All other states are impossible, so they are initialized with a probability of 0.
    let mut probs = vec![[0.; 100]; target + 1];
    probs[0][0] = 1.;

    let mut new_probs = vec![[0.; 100]; target + 1];

    for _ in 0..pulls {
        for (pity_count, rate) in SIX_STAR_RATES.iter().enumerate() {
            for target_count in 0..target {
                let old_prob = probs[target_count][pity_count];

                new_probs[target_count][pity_count + 1] += old_prob * (1. - rate);
                new_probs[target_count][0] += old_prob * rate * (1. - subrate);
                new_probs[target_count + 1][0] += old_prob * rate * subrate;
            }
        }

        // unlike direct re-assignment, swapping and filling avoids re-allocating memory
        // update the `probs` matrix
        swap(&mut probs, &mut new_probs);
        // reset the `new_probs` matrix for the next pull
        new_probs.fill([0.; 100]);

        // There are two kinds of states that can reach probs[target][0]:
        // - probs[target][p] (pulled an off-rate 6*, resetting pity to 0), or
        // - probs[target - 1][p] (pulled a target 6*, increasing the target count by 1 and resetting pity to 0),
        // where p is any pity count.
        // However, we only update states in the matrix with target counts from 0 to target - 1,
        // so states of the first kind are never considered in our calculations.
        // This means that the probability at probs[target][0] is the probability that
        // a state of the second kind got here (and the banner target was achieved) on this pull.
        pdist.push(probs[target][0]);
    }

    pdist
}

#[allow(clippy::cast_precision_loss, clippy::similar_names)]
// Calculates the probability of achieving the target counts of all `banners` within `pulls`.
pub fn calculate(banners: &[Banner], pulls: usize) -> Float {
    let mut conv_size = 0;
    let mut pdists = Vec::with_capacity(banners.len());
    let mut total_bonus_pulls = 0;

    for banner in banners {
        // At least 1 pull needs to be spent on each banner, so for any single banner,
        // we subtract the number of other banners to calculate `max_pulls`.
        let max_pulls = pulls + banner.bonus_pulls - (banners.len() - 1);

        let pdist = banner_pdist(banner.target, max_pulls, banner.subrate);

        conv_size += max_pulls;
        pdists.push(pdist);
        total_bonus_pulls += banner.bonus_pulls;
    }

    // initialize the FFT calculator
    let mut planner = RealFftPlanner::<Float>::new();
    let fft = planner.plan_fft_forward(conv_size);

    // We store the result of DFT multiplication in a vector.
    // We need to initialize the elements of this vector with a value X
    // such that X * first DFT frequency = first DFT frequency.
    // Since DFT frequencies are complex numbers, X is the complex multiplication identity: 1 + 0i.
    // Also, this vector has length `conv_size`/2 + 1 because `realfft` transforms
    // the real-valued probability distributions of length `conv_size`
    // into complex vectors of length `conv_size`/2 + 1.
    let mut combined_dft = vec![Complex::new(1., 0.); conv_size / 2 + 1];

    for mut pdist in pdists {
        // vectors are padded with 0s to calculate a "full" convolution
        pdist.resize_with(conv_size, Default::default);

        // apply FFT to probability distribution
        let mut dft = fft.make_output_vec();
        fft.process(&mut pdist, &mut dft).unwrap();

        // multiply the DFTs together (Hadamard product)
        for (sample, combined_sample) in zip(dft, &mut combined_dft) {
            *combined_sample *= sample;
        }
    }

    // We apply the IFFT to get the convolved distribution.
    // According to https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization,
    // the result needs to be divided by `conv_size` to get the actual convolution values.
    // However, instead of dividing each element of the resulting sequence by `conv_size`
    // and then summing the quotients to obtain the final probability,
    // we can sum the elements and then divide the sum by `conv_size`.
    // This reduces the number of operations needed and works because division is right-distributive over addition.
    let ifft = planner.plan_fft_inverse(conv_size);
    let mut combined_seq = ifft.make_output_vec();
    ifft.process(&mut combined_dft, &mut combined_seq).unwrap();

    // calculate the final probability
    combined_seq
        .into_iter()
        .take(pulls + total_bonus_pulls - (banners.len() - 1))
        .sum::<Float>()
        / (conv_size as Float)
}

pub struct Banner {
    pub target: usize,
    pub subrate: Float,
    pub bonus_pulls: usize,
}
