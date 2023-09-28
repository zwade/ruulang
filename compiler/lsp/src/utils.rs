use tower_lsp::lsp_types::{Position, Range};

pub fn get_line_prefix_sum(contents: &String) -> Vec<u32> {
    contents
        .split("\n")
        .fold((0, Vec::new()), |(mut sum, mut acc), line| {
            acc.push(sum);
            sum += (line.len() as u32) + 1;
            (sum, acc)
        })
        .1
}

fn location_to_position(line_prefix_sum: &Vec<u32>, location: u32) -> Position {
    let line = line_prefix_sum
        .iter()
        .enumerate()
        .find(|(_, sum)| **sum > location)
        .map_or(line_prefix_sum.len() - 1, |idx| idx.0 - 1);

    let column = location - line_prefix_sum[line];

    Position::new(line as u32, column)
}

pub fn position_to_location(line_prefix_sum: &Vec<u32>, position: &Position) -> u32 {
    line_prefix_sum[position.line as usize] + position.character
}

pub fn location_pair_to_range(contents: &String, start: u32, end: u32) -> Range {
    let line_prefix_sum = get_line_prefix_sum(contents);

    let start = location_to_position(&line_prefix_sum, start);
    let end = location_to_position(&line_prefix_sum, end);

    Range::new(start, end)
}
