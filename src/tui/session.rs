#[must_use]
pub fn make_session_title(task: &str) -> String {
    let stop_words = [
        "the",
        "this",
        "that",
        "with",
        "from",
        "into",
        "your",
        "about",
        "please",
        "task",
        "pull",
        "request",
        "http",
        "https",
        "github",
        "com",
        "что",
        "это",
        "как",
        "для",
        "надо",
        "нужно",
        "and",
        "сделай",
        "пожалуйста",
        "через",
        "чтобы",
    ];
    let mut words = Vec::new();

    for raw in task.split(|ch: char| !ch.is_alphanumeric()) {
        let word = raw.trim().to_lowercase();
        if word.len() < 3 || stop_words.contains(&word.as_str()) {
            continue;
        }
        words.push(word);
        if words.len() == 4 {
            break;
        }
    }

    if words.is_empty() {
        "new-session".to_string()
    } else {
        words.join("-")
    }
}
