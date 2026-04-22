use crate::taylor::series::TaylorSeriesResult;

pub fn highest_nonzero_term(series: &TaylorSeriesResult) -> Option<usize> {
    series
        .coefficients
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, value)| if *value != 0.0 { Some(index) } else { None })
}
