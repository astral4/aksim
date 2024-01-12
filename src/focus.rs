use crate::Float;
use core::iter::zip;
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
fn banner_pdist(target: usize, pulls: usize, subrate: Float, focus: bool) -> Vec<Float> {
    let mut pdist = Vec::with_capacity(pulls);

    // probabilities that each pity level would be reached from 0 pity before getting reset
    let mut probs_pity_reached = [1.; 99];
    for i in 1..99 {
        probs_pity_reached[i] = probs_pity_reached[i - 1] * (1.0 - SIX_STAR_RATES[i - 1]);
    }

    // probabilities to get the next any 6-star in exactly N rolls starting from 0 pity
    let mut pdist_6star_in_exactly_nrolls = [0.; 99];
    for i in 1..99 {
        pdist_6star_in_exactly_nrolls[i] = probs_pity_reached[i - 1] - probs_pity_reached[i];
    }

    if focus {
        let mut no_target = [0.; 249];
        no_target[0] = 1.0;
        // probabilities to get the target 6-star in exactly N rolls starting from 0 pity and 0 focus
        let mut pdist_target_in_exactly_nrolls_with_focus = [0.; 249];
        for i in 0..151 {
            for j in 1..99 {
                let x = no_target[i] * pdist_6star_in_exactly_nrolls[j];
                if i + j > 150 {
                    pdist_target_in_exactly_nrolls_with_focus[i + j] += x;
                } else {
                    pdist_target_in_exactly_nrolls_with_focus[i + j] += x * subrate;
                    no_target[i + j] += x * (1.0 - subrate);
                }
            }
        }

        // probs[rolls][wins][focus]
        // where focus: (0="unspent and counter at 0", 1="spent")
        // only states where pity=0 are considered
        let mut probs = vec![vec![[0.; 2]; target + 1]; pulls + 1];
        probs[0][0][0] = 1.0;

        for p in 0..pulls {
            for t in 0..target {
                // focus still active, hitting target without exhausting focus chance
                for next in 1..151 {
                    if p + next > pulls {
                        break;
                    }
                    probs[p + next][t + 1][0] +=
                        probs[p][t][0] * pdist_target_in_exactly_nrolls_with_focus[next];
                }
                // focus still active, hitting target and exhausting focus chance
                for next in 151..249 {
                    if p + next > pulls {
                        break;
                    }
                    probs[p + next][t + 1][1] +=
                        probs[p][t][0] * pdist_target_in_exactly_nrolls_with_focus[next];
                }
                // focus no longer active, relying on subrate to hit target
                for next in 1..99 {
                    if p + next > pulls {
                        break;
                    }
                    probs[p + next][t + 1][1] +=
                        probs[p][t][1] * pdist_6star_in_exactly_nrolls[next] * subrate;
                    probs[p + next][t][1] +=
                        probs[p][t][1] * pdist_6star_in_exactly_nrolls[next] * (1.0 - subrate);
                }
            }
        }
        for p in 1..=pulls {
            pdist.push(probs[p][target][0] + probs[p][target][1]);
        }
    } else {
        // probs[rolls][wins]
        // only states where pity=0 are considered
        let mut probs = vec![vec![0.; target + 1]; pulls + 1];
        probs[0][0] = 1.0;

        for p in 0..pulls {
            for t in 0..target {
                // no focus, relying on subrate to hit target
                for next in 1..99 {
                    if p + next > pulls {
                        break;
                    }
                    probs[p + next][t + 1] +=
                        probs[p][t] * pdist_6star_in_exactly_nrolls[next] * subrate;
                    probs[p + next][t] +=
                        probs[p][t] * pdist_6star_in_exactly_nrolls[next] * (1.0 - subrate);
                }
            }
        }
        for p in 1..=pulls {
            pdist.push(probs[p][target]);
        }
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

        let pdist = banner_pdist(banner.target, max_pulls, banner.subrate, banner.has_focus);

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
    // According to https://docs.rs/rustfft/6.1.0/rustfft/index.html#normalization,
    // the result needs to be divided by `conv_size` to get the actual convolution values.
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
    pub has_focus: bool,
}
