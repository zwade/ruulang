fn parse_multiline_comment(input: String) -> Option<String> {
    let trimmed = input.trim();
    if !trimmed.starts_with("/*") {
        return None;
    }

    let mut result = vec![];
    let lines = trimmed.lines().into_iter().collect::<Vec<_>>();

    if lines.len() == 0 {
        return None;
    }

    let first_line = lines[0];
    let trimmed_first_line = first_line
        .trim_start()
        .trim_start_matches("/*")
        .trim_start_matches("*");

    if lines.len() == 1 {
        let trimmed_single_line = trimmed_first_line
            .trim_end()
            .trim_end_matches("*/")
            .trim_end_matches("*");

        return Some(trimmed_single_line.to_string());
    }

    if trimmed_first_line.trim().len() > 0 {
        result.push(trimmed_first_line);
    }

    for line in lines[1..lines.len() - 1].iter() {
        let trimmed_line = line.trim_start().trim_start_matches("*");

        result.push(trimmed_line);
    }

    let last_line = lines[lines.len() - 1];
    let trimmed_last_line = last_line
        .trim_end()
        .trim_end_matches("*/")
        .trim_end_matches("*");

    if trimmed_last_line.trim().len() > 0 {
        result.push(trimmed_last_line);
    }

    let joined = result.join("\n");

    Some(joined)
}

pub fn parse_docstrings(inputs: Vec<String>) -> Option<String> {
    let results = inputs
        .into_iter()
        .filter_map(|input| parse_multiline_comment(input))
        .collect::<Vec<_>>();

    if results.len() == 0 {
        None
    } else {
        Some(results.join("\n"))
    }
}
