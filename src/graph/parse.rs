use super::model::CommitNode;

pub fn parse_log(raw: &str) -> Vec<CommitNode> {
    let fields: Vec<&str> = raw.split('\0').filter(|s| !s.is_empty()).collect();
    let mut commits = Vec::new();
    for chunk in fields.chunks(6) {
        if chunk.len() < 6 {
            continue;
        }
        let hash = chunk[0].trim().to_string();
        if hash.is_empty() {
            continue;
        }
        let parents: Vec<String> = chunk[1]
            .split_whitespace()
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        let refs = parse_refs(chunk[2]);
        let subject = chunk[3].trim().to_string();
        let author = chunk[4].trim().to_string();
        let timestamp = chunk[5].trim().parse().unwrap_or(0);
        commits.push(CommitNode {
            hash,
            parents,
            refs,
            subject,
            author,
            timestamp,
        });
    }
    commits
}

fn parse_refs(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches("HEAD -> ").to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_null_delimited_records() {
        let raw = "abc123\0def456\0(HEAD -> main)\0Fix bug\0Alice\01700000000\0";
        let commits = parse_log(raw);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].parents, vec!["def456"]);
        assert_eq!(commits[0].refs, vec!["main"]);
        assert_eq!(commits[0].subject, "Fix bug");
    }
}
