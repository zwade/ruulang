pub fn decompose(name: &str) -> Vec<String> {
    let mut acc = Vec::new();
    let mut local_acc = String::new();

    for chr in name.chars() {
        if chr.is_uppercase() || chr == '-' || chr == '_' {
            acc.push(local_acc);
            local_acc = String::new();
        }

        if chr.is_alphanumeric() {
            local_acc.push(chr.to_ascii_lowercase());
        }
    }

    if local_acc.len() > 0 {
        acc.push(local_acc);
    }

    acc
}

pub fn camel_case(name: &str) -> String {
    let mut acc = String::new();

    for part in decompose(name) {
        if part.len() > 0 {
            let mut chars = part.chars();
            let first = chars.next().unwrap().to_ascii_uppercase();
            acc.push(first);
            acc.push_str(chars.as_str());
        }
    }

    acc
}

pub fn _snake_case(name: &str) -> String {
    let mut acc = String::new();

    for part in decompose(name) {
        if part.len() > 0 {
            if acc.len() > 0 {
                acc.push('_');
            }

            acc.push_str(&part);
        }
    }

    acc
}
