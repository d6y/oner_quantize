// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::interval::merge_neighbours_with_same_class;
use crate::Interval;
use ord_subset::OrdSubset;
use ord_subset::OrdSubsetSliceExt;
use std::fmt::Debug;
use std::hash::Hash;

mod splits;
use splits::{intervals_from_splits, trim_splits};

/// Quantize the given `attribute` (aka feature, column) into an ordered list of `Intervals`.
///
/// # Arguments
///
/// * `attribute` - a single attribute, typically numeric, to be quantized.
/// * `classes` - the corresponsing class for each attribute value.
/// * `small` -  the small disjunct threshold, such as 3. There has to be at least one class in an interval with more than `small` values in the interval.
///
/// # Examples
/// ```
/// use oner_quantize::find_intervals;
/// use oner_quantize::Interval;
/// use oner_quantize::Interval::{Lower, Range, Upper};
///
/// // Fake data that has three clear splits:
/// let attribute = vec![  1, 10,   3,   1,  20,  30,  100];
/// let classes   = vec!["a", "b", "a", "a", "b", "b", "c"];
///
/// let intervals =
///    find_intervals(&attribute, &classes, 2);
///
/// assert_eq!(intervals, vec![
///   Lower { below: 10, class: "a" },
///   Range { from: 10, below: 100, class: "b" },
///   Upper { from: 100, class: "c" }
/// ]);
/// ```
pub fn find_intervals<A, C>(attribute: &[A], classes: &[C], small: usize) -> Vec<Interval<A, C>>
where
    A: OrdSubset + Copy + Debug,
    C: Eq + Hash + Copy + Debug,
{
    // 1. Get the attribute values (plus associated class) in attribute sorted order:
    let mut sorted: Vec<(&A, &C)> = Vec::new();
    for (v, c) in attribute.iter().zip(classes.iter()) {
        sorted.push((v, c));
    }
    sorted.ord_subset_sort_by_key(|pair| pair.0);

    // 2. Create a (tentative) split each time the attribute value changes.

    // `split_index` contains indicies into `sorted` where we might split the attribute into an interval boundary.
    // That is, a value of 1 in `split_index` means the attribute value at sorted[0] differs from sorted[1].
    // The split happens between index 0 and 1 in that example.
    let mut split_index = Vec::new();
    for (prev_index, ((cur_value, _cur_class), (prev_value, _prev_class))) in
        sorted.iter().skip(1).zip(sorted.iter()).enumerate()
    {
        if cur_value > prev_value {
            split_index.push(prev_index + 1);
        }
    }

    // 3. Remove splits that are too small:
    let split_index_trimmed = trim_splits(split_index, small, &sorted);

    // 4. Generate distinct intervals from the splits:
    let intervals: Vec<Interval<A, C>> = intervals_from_splits(split_index_trimmed, &sorted);

    // 5. Remove redundant intervals:
    merge_neighbours_with_same_class(&intervals)
}

#[cfg(test)]
mod tests {
    use super::find_intervals;
    use super::Interval;
    #[test]
    fn test_golf_example() {
        // This example (inputs, and boundary points) comes from:
        // Nevill-Manning, Holmes & Witten (1995)  _The Development of Holte's 1R Classifier_, p. 2

        let attrbibute = vec![64, 65, 68, 69, 70, 71, 72, 72, 75, 75, 80, 81, 83, 85];

        let classes = vec!["p", "d", "p", "p", "p", "d", "p", "d", "p", "p", "d", "p", "p", "d"];

        let actual = find_intervals(&attrbibute, &classes, 3);

        let expected = vec![Interval::lower(85, "p"), Interval::upper(85, "d")];

        assert_eq!(expected, actual);
    }
}

/// Find which interval applies to a given attribute value.
///
/// # Examples
///
/// ```
/// use oner_quantize::Interval;
/// use oner_quantize::quantize;
///
/// let intervals = vec![
///     Interval::lower(15, "x"),
///     Interval::range(15, 20, "y"),
///     Interval::upper(20, "z")
/// ];
///
/// // Quantize into an interval, and extract the corresponding class:
/// let class = |attribute_value| quantize(&intervals, attribute_value)
///   .map(|interval| interval.class());
///
/// assert_eq!(class(10), Some(&"x"));
/// assert_eq!(class(15), Some(&"y"));
/// assert_eq!(class(99), Some(&"z"));
/// ```
pub fn quantize<A, C>(intervals: &[Interval<A, C>], attribute_value: A) -> Option<&Interval<A, C>>
where
    A: PartialOrd + Copy,
    C: Copy,
{
    intervals.iter().find(|interval| interval.matches(attribute_value))
}
