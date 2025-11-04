//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois      //
//                                                                             //
// https://github.com/type-one/PublishSubscribeRust                            //
//                                                                             //
// MIT License                                                                 //
//                                                                             //
// This software is provided 'as-is', without any express or implied           //
// warranty.In no event will the authors be held liable for any damages        //
// arising from the use of this software.                                      //
//                                                                             //
// Permission is granted to anyone to use this software for any purpose,       //
// including commercial applications, and to alter itand redistribute it       //
// freely, subject to the following restrictions :                             //
//                                                                             //
// 1. The origin of this software must not be misrepresented; you must not     //
// claim that you wrote the original software.If you use this software         //
// in a product, an acknowledgment in the product documentation would be       //
// appreciated but is not required.                                            //
// 2. Altered source versions must be plainly marked as such, and must not be  //
// misrepresented as being the original software.                              //
// 3. This notice may not be removed or altered from any source distribution.  //
//-----------------------------------------------------------------------------//

use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
/// A simple histogram implementation to track occurrences of values.
pub struct Histogram<T> {
    occurences: HashMap<T, usize>,
    total_count: usize,
    top_occurence: Option<(T, usize)>,
    top_value: Option<T>,
}

/// Implementation of the Histogram methods.
impl<T> Histogram<T>
where
    T: Hash + Eq + Clone + Ord, // Ensure T can be used as a key in HashMap and cloned
{
    /// Creates a new Histogram.
    pub fn new() -> Self {
        Histogram {
            occurences: HashMap::new(),
            total_count: 0,
            top_occurence: None,
            top_value: None,
        }
    }

    /// Adds a value to the histogram.
    pub fn add(&mut self, value: T) {
        let count = self.occurences.entry(value.clone()).or_insert(0);
        *count += 1;
        self.total_count += 1;

        if let Some((_, top_count)) = &self.top_occurence {
            if *count > *top_count {
                self.top_occurence = Some((value.clone(), *count));
                self.top_value = Some(value);
            }
        } else {
            self.top_occurence = Some((value.clone(), *count));
            self.top_value = Some(value);
        }
    }

    /// Returns the total count of values in the histogram.
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    /// Returns the value with the highest occurrence.
    pub fn top_value(&self) -> Option<&T> {
        self.top_value.as_ref()
    }

    /// Returns the count of the value with the highest occurrence.
    pub fn top_count(&self) -> Option<usize> {
        self.top_occurence.as_ref().map(|(_, count)| *count)
    }

    /// Returns the value with the highest occurrence along with its count.
    pub fn top_value_with_count(&self) -> Option<(&T, usize)> {
        self.top_occurence
            .as_ref()
            .map(|(value, count)| (value, *count))
    }

    /// Clears the histogram.
    pub fn clear(&mut self) {
        self.occurences.clear();
        self.total_count = 0;
        self.top_occurence = None;
        self.top_value = None;
    }

    // average() is provided via a usize-specialized impl below.
}

/// Implementation of the Default trait for Histogram.
impl<T> Default for Histogram<T>
where
    T: Hash + Eq + Clone + Ord, // Ensure T can be used as a key in HashMap and cloned
{
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized implementation of statistical methods for Histogram with primitive integer values (any size, signed or not).
impl<T> Histogram<T>
where
    T: Hash + Eq + Clone + Ord + Add + Copy + Into<f64>, // Ensure T is a primitive integer AND can be used as a key in HashMap and cloned
{
    /// Calculates the average of the sampled integer values.
    pub fn average(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let sum: f64 = self
            .occurences
            .iter()
            .map(|(value, count)| (*value).into() * (*count as f64))
            .sum();

        sum / (self.total_count as f64)
    }

    /// Calculates the variance of the sampled usize values given the average.
    pub fn variance(&self, average: f64) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let variance_sum: f64 = self
            .occurences
            .iter()
            .map(|(value, count)| {
                // https://www.calculatorsoup.com/calculators/statistics/standard-deviation-calculator.php
                let diff = (*value).into() - average;
                diff * diff * (*count as f64)
            })
            .sum();

        variance_sum / (self.total_count as f64)
    }

    /// Calculates the standard deviation given the variance.
    pub fn standard_deviation(&self, variance: f64) -> f64 {
        variance.sqrt()
    }

    /// Calculates the median of the sampled usize values.
    pub fn median(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let mut sorted_values: Vec<(&T, &usize)> = self.occurences.iter().collect();
        sorted_values.sort_by_key(|&(value, _)| *value);

        // https://www.calculator.net/mean-median-mode-range-calculator.html

        let mid_index = self.total_count / 2;
        let mut cumulative_count = 0;

        for (value, count) in sorted_values {
            cumulative_count += *count;
            if cumulative_count > mid_index {
                return (*value).into();
            }
        }

        0.0
    }

    /// Calculates the Gaussian density function for a given x, mean, and standard deviation.
    pub fn gaussian_density_function(&self, x: f64, mean: f64, std_dev: f64) -> f64 {
        // https://fr.wikipedia.org/wiki/Loi_normale
        // https://www.savarese.org/math/gaussianintegral.html

        if std_dev == 0.0 {
            return 0.0;
        }

        let coeff = 1.0 / (std_dev * (2.0 * std::f64::consts::PI).sqrt());
        let exponent = -((x - mean).powi(2)) / (2.0 * std_dev.powi(2));

        coeff * exponent.exp()
    }

    /// Estimates the probability of a value falling between lower_bound and upper_bound
    /// using the Gaussian distribution with the specified mean and standard deviation
    pub fn gaussian_probability_between(
        &self,
        lower_bound: f64,
        upper_bound: f64,
        mean: f64,
        std_dev: f64,
        montecarlo_samples: usize,
    ) -> f64 {
        // https://en.wikipedia.org/wiki/Normal_distribution#Cumulative_distribution_function
        // https://www.probabilitycourse.com/chapter5/5_2_3_normal_distribution.php
        // https://cameron-mcelfresh.medium.com/monte-carlo-integration-313b37157852
        // https://www.savarese.org/math/gaussianintegral.html
        // https://en.wikipedia.org/wiki/Gaussian_integral
        // https://stackoverflow.com/questions/288739/generate-random-numbers-uniformly-over-an-entire-range

        let mut result = 0.0;

        if std_dev > 0.0 && montecarlo_samples > 0 {
            let mut rng = rand::rng();

            for _ in 0..montecarlo_samples {
                let sample = rng.random_range(lower_bound..upper_bound);
                let density = self.gaussian_density_function(sample, mean, std_dev);

                result += density;
            }

            result = result * (upper_bound - lower_bound) / ((montecarlo_samples - 1) as f64);
        } // end if

        result
    }
}
