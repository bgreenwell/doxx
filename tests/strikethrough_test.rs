use doxx::document::{FormattedRun, TextFormatting};

#[cfg(test)]
mod strikethrough_tests {
    use super::*;

    #[test]
    fn test_strikethrough_formatting_structure() {
        // Test that strikethrough field is properly included in TextFormatting
        let formatting = TextFormatting {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: true,
            font_size: None,
            color: None,
        };

        assert_eq!(formatting.strikethrough, true);
        assert_eq!(formatting.bold, false);
        assert_eq!(formatting.italic, false);
        assert_eq!(formatting.underline, false);
    }

    #[test]
    fn test_strikethrough_formatting_default() {
        // Test that default formatting has strikethrough set to false
        let formatting = TextFormatting::default();
        assert_eq!(formatting.strikethrough, false);
    }

    #[test]
    fn test_formatted_run_with_strikethrough() {
        // Test that FormattedRun properly handles strikethrough formatting
        let formatting = TextFormatting {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: true,
            font_size: None,
            color: None,
        };

        let run = FormattedRun {
            text: "This text is struck through".to_string(),
            formatting,
        };

        assert_eq!(run.formatting.strikethrough, true);
        assert_eq!(run.text, "This text is struck through");
    }

    #[test]
    fn test_mixed_formatting_with_strikethrough() {
        // Test strikethrough combined with other formatting
        let formatting = TextFormatting {
            bold: true,
            italic: true,
            underline: false,
            strikethrough: true,
            font_size: Some(12.0),
            color: Some("#FF0000".to_string()),
        };

        assert_eq!(formatting.bold, true);
        assert_eq!(formatting.italic, true);
        assert_eq!(formatting.underline, false);
        assert_eq!(formatting.strikethrough, true);
        assert_eq!(formatting.font_size, Some(12.0));
        assert_eq!(formatting.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_formatting_serialization() {
        // Test that strikethrough formatting can be serialized to JSON
        let formatting = TextFormatting {
            bold: true,
            italic: false,
            underline: true,
            strikethrough: true,
            font_size: Some(14.0),
            color: Some("#0000FF".to_string()),
        };

        let json = serde_json::to_string(&formatting).expect("Failed to serialize");
        let deserialized: TextFormatting =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.bold, true);
        assert_eq!(deserialized.italic, false);
        assert_eq!(deserialized.underline, true);
        assert_eq!(deserialized.strikethrough, true);
        assert_eq!(deserialized.font_size, Some(14.0));
        assert_eq!(deserialized.color, Some("#0000FF".to_string()));
    }

    #[test]
    fn test_run_consolidation_with_strikethrough() {
        // Test that runs with identical strikethrough formatting are properly consolidated
        let formatting1 = TextFormatting {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: true,
            font_size: None,
            color: None,
        };

        let formatting2 = formatting1.clone();

        let runs = vec![
            FormattedRun {
                text: "First part ".to_string(),
                formatting: formatting1,
            },
            FormattedRun {
                text: "second part".to_string(),
                formatting: formatting2,
            },
        ];

        let consolidated = FormattedRun::consolidate_runs(runs);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(consolidated[0].text, "First part second part");
        assert_eq!(consolidated[0].formatting.strikethrough, true);
    }

    #[test]
    fn test_run_consolidation_different_strikethrough() {
        // Test that runs with different strikethrough settings are not consolidated
        let formatting1 = TextFormatting {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: true,
            font_size: None,
            color: None,
        };

        let formatting2 = TextFormatting {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false, // Different strikethrough setting
            font_size: None,
            color: None,
        };

        let runs = vec![
            FormattedRun {
                text: "Struck text ".to_string(),
                formatting: formatting1,
            },
            FormattedRun {
                text: "normal text".to_string(),
                formatting: formatting2,
            },
        ];

        let consolidated = FormattedRun::consolidate_runs(runs);
        assert_eq!(consolidated.len(), 2);
        assert_eq!(consolidated[0].text, "Struck text ");
        assert_eq!(consolidated[0].formatting.strikethrough, true);
        assert_eq!(consolidated[1].text, "normal text");
        assert_eq!(consolidated[1].formatting.strikethrough, false);
    }
}

// Integration tests that would require actual DOCX files
#[cfg(test)]
mod integration_tests {
    use std::process::Command;

    #[test]
    fn test_strikethrough_help_available() {
        // This test ensures the application can start and shows help without panicking
        // when strikethrough formatting is included in the codebase
        let output = Command::new("cargo")
            .args(["run", "--bin", "doxx", "--", "--help"])
            .output()
            .expect("Failed to execute doxx");

        assert!(
            output.status.success(),
            "doxx should show help successfully with strikethrough support"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("doxx"), "Should contain program name");
    }
}
