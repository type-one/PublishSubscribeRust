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

/// A simple histogram implementation to track occurrences of values.
pub struct Histogram<T> {
    occurences: std::collections::HashMap<T, usize>,
    total_count: usize,
    top_occurence: Option<(T, usize)>,
    top_value: Option<T>,
}

/// Implementation of the Histogram methods.
impl<T> Histogram<T>
where
    T: std::hash::Hash + Eq + Clone + Ord,
{
    /// Creates a new Histogram.
    pub fn new() -> Self {
        Histogram {
            occurences: std::collections::HashMap::new(),
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
}

/// Implementation of the Default trait for Histogram.
impl<T> Default for Histogram<T>
where
    T: std::hash::Hash + Eq + Clone + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}
