use serde::{Deserialize, Serialize};

/// Content that can be sent in messages
/// Designed to be extensible for multimodal (images, audio, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content
    Text(String),
    
    /// Multipart content (for mixing text + images in future)
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    
    /// Reasoning content (for o1 models)
    Reasoning {
        text: String,
    },
    
    // Future: Image support
    // ImageUrl {
    //     image_url: ImageUrl,
    // },
}

// Future multimodal support
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ImageUrl {
//     pub url: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub detail: Option<ImageDetail>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub enum ImageDetail {
//     Auto,
//     Low,
//     High,
// }

impl Content {
    /// Create text content
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }
    
    /// Get as plain text (if possible)
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(parts) => {
                // If single text part, return it
                if parts.len() == 1 {
                    match &parts[0] {
                        ContentPart::Text { text } => return Some(text),
                        _ => return None,
                    }
                }
                None
            }
        }
    }
    
    /// Get reasoning text (if present)
    pub fn reasoning_texts(&self) -> Vec<&str> {
        match self {
            Self::Text(_) => vec![],
            Self::Parts(parts) => {
                parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Reasoning { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect()
            }
        }
    }
    
    /// Get all text (reasoning + message)
    pub fn all_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Parts(parts) => {
                parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } | ContentPart::Reasoning { text } => Some(text.as_str()),
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}
