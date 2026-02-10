use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiEntity {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub original: String,
}

pub fn detect_emails(text: &str) -> Vec<PiiEntity> {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    re.find_iter(text)
        .map(|m| PiiEntity {
            entity_type: "email".to_string(),
            start: m.start(),
            end: m.end(),
            original: m.as_str().to_string(),
        })
        .collect()
}

pub fn detect_files(text: &str) -> Vec<PiiEntity> {
    let re = Regex::new(
        r"(?x)
        # UNC paths: \\server\share\file
        \\\\[a-zA-Z0-9._-]+(?:\\[a-zA-Z0-9._-]+)+
        |
        # Windows drive paths: C:\Users\Bob\file (2+ segments after drive)
        [A-Za-z]:\\[a-zA-Z0-9._-]+(?:\\[a-zA-Z0-9._-]+)+
        |
        # Home-relative: ~/documents/report.pdf
        ~/[a-zA-Z0-9._-]+(?:/[a-zA-Z0-9._-]+)*
        |
        # Relative dot-prefix: ./config.yml or ../data/file.csv
        \.\.?/[a-zA-Z0-9._-]+(?:/[a-zA-Z0-9._-]+)*
        |
        # Unix absolute: /home/user/file (2+ segments)
        /[a-zA-Z0-9._-]+(?:/[a-zA-Z0-9._-]+)+
        "
    )
    .unwrap();
    re.find_iter(text)
        .map(|m| PiiEntity {
            entity_type: "file".to_string(),
            start: m.start(),
            end: m.end(),
            original: m.as_str().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_unc_path() {
        let text = r"Open \\server\share\docs\report.docx please";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].original, r"\\server\share\docs\report.docx");
        assert_eq!(entities[0].entity_type, "file");
    }

    #[test]
    fn test_detect_windows_drive_path() {
        let text = r"See C:\Users\Bob\Documents\secrets.docx for details";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].original, r"C:\Users\Bob\Documents\secrets.docx");
    }

    #[test]
    fn test_detect_home_relative_path() {
        let text = "Config is at ~/documents/report.pdf here";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].original, "~/documents/report.pdf");
    }

    #[test]
    fn test_detect_relative_dot_path() {
        let text = "Run ./config.yml and ../data/file.csv";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].original, "./config.yml");
        assert_eq!(entities[1].original, "../data/file.csv");
    }

    #[test]
    fn test_detect_unix_absolute_path() {
        let text = "The file /home/alice/Documents/report.pdf is ready";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].original, "/home/alice/Documents/report.pdf");
    }

    #[test]
    fn test_no_match_plain_text() {
        let text = "This is just regular text with no file paths and/or other things.";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_no_match_single_segment() {
        // Single-segment paths like /etc should not match
        let text = "Check /etc for config";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_byte_offset_correctness() {
        let text = "path: /home/user/file.txt end";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].start, 6);
        assert_eq!(entities[0].end, 25);
        assert_eq!(&text[entities[0].start..entities[0].end], "/home/user/file.txt");
    }

    #[test]
    fn test_multiple_paths_in_text() {
        let text = r"Copy /home/alice/doc.txt to C:\Users\Bob\doc.txt";
        let entities = detect_files(text);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].original, "/home/alice/doc.txt");
        assert_eq!(entities[1].original, r"C:\Users\Bob\doc.txt");
    }
}
