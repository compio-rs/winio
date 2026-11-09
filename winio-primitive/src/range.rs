/// Fit a page into the `[min, max]` range of a scroll-like control.
///
/// Range controls keep the invariant `max >= min + page`. This helper applies
/// that invariant to a requested `(min, max, page)`:
///
/// * if `min + page` does not overflow, `max` is raised to `min + page` when it
///   is smaller;
/// * otherwise `min` is lowered to `max.saturating_sub(page)`, and `max` is
///   raised to `min + page` if necessary.
///
/// `page` is clamped to at least 1.
pub fn fit_page(mut min: usize, mut max: usize, page: usize) -> (usize, usize) {
    let page = page.max(1);
    match min.checked_add(page) {
        Some(required) => {
            if max < required {
                max = required;
            }
        }
        None => {
            min = max.saturating_sub(page);
            let required = min.saturating_add(page);
            if max < required {
                max = required;
            }
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raises_max() {
        assert_eq!(fit_page(0, 3, 10), (0, 10));
        assert_eq!(fit_page(20, 10, 1), (20, 21));
        assert_eq!(fit_page(0, 3, 3), (0, 3));
    }

    #[test]
    fn keeps_existing_range() {
        assert_eq!(fit_page(0, 10, 1), (0, 10));
        assert_eq!(fit_page(0, 10, 10), (0, 10));
    }

    #[test]
    fn handles_overflow() {
        assert_eq!(fit_page(0, 0, usize::MAX), (0, usize::MAX));
        assert_eq!(fit_page(1, usize::MAX, usize::MAX), (0, usize::MAX));
        assert_eq!(
            fit_page(usize::MAX - 2, usize::MAX, 5),
            (usize::MAX - 5, usize::MAX)
        );
    }

    #[test]
    fn clamps_zero_page() {
        assert_eq!(fit_page(0, 10, 0), (0, 10));
    }
}
