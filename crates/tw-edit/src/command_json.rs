use crate::Command;

/// Serialize a [`Command`] to JSON bytes for FFI dispatch.
pub fn command_to_json(command: &Command) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(command)
}

/// Deserialize a [`Command`] from JSON bytes.
pub fn command_from_json(bytes: &[u8]) -> Result<Command, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// Serialize a [`Command`] to a JSON string (tests and debugging).
pub fn command_to_json_str(command: &Command) -> Result<String, serde_json::Error> {
    serde_json::to_string(command)
}

/// Deserialize a [`Command`] from a JSON string.
pub fn command_from_json_str(json: &str) -> Result<Command, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocPosition, DocRange};
    use tw_model::NodeId;
    use uuid::Uuid;

    #[test]
    fn insert_text_roundtrip() {
        let run_id = NodeId::from_uuid(Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap());
        let command = Command::InsertText {
            run_id,
            offset: 3,
            text: "hi".into(),
        };
        let json = command_to_json_str(&command).unwrap();
        assert!(json.contains("\"type\":\"InsertText\""));
        let decoded = command_from_json_str(&json).unwrap();
        match decoded {
            Command::InsertText {
                run_id: decoded_run,
                offset,
                text,
            } => {
                assert_eq!(decoded_run, run_id);
                assert_eq!(offset, 3);
                assert_eq!(text, "hi");
            }
            _ => panic!("expected InsertText"),
        }
    }

    #[test]
    fn set_char_format_range_roundtrip() {
        let run_id = NodeId::from_uuid(Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap());
        let command = Command::SetCharFormatRange {
            range: DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 5,
                },
            },
            format: tw_model::CharFormat {
                character_spacing: Some(2.5),
                ..Default::default()
            },
            merge: true,
        };
        let decoded = command_from_json(&command_to_json(&command).unwrap()).unwrap();
        match decoded {
            Command::SetCharFormatRange { format, merge, .. } => {
                assert_eq!(format.character_spacing, Some(2.5));
                assert!(merge);
            }
            _ => panic!("expected SetCharFormatRange"),
        }
    }
}
