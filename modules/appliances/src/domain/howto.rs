//! Le mode d'emploi d'un appareil : les étapes que le livret numérote, et les liens de vidéo
//! qu'un lecteur sait ouvrir.

use serde_json::Value;

/// Une étape du mode d'emploi : sa consigne, et le schéma que l'hôte a glissé dedans (§2.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowToStep {
    pub text: String,
    /// La référence `portaki-file:` ou l'URL de l'image, telle que l'éditeur l'a posée.
    pub image: Option<String>,
}

/// Les étapes des listes à puces ou numérotées d'un document TipTap (mode d'emploi libre).
pub fn extract_howto_steps(description: &str) -> Vec<HowToStep> {
    let Ok(value) = serde_json::from_str::<Value>(description.trim()) else {
        return Vec::new();
    };
    if value["type"] != "doc" {
        return Vec::new();
    }
    value["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|child| matches!(child["type"].as_str(), Some("bulletList" | "orderedList")))
        .flat_map(|list| list["content"].as_array().into_iter().flatten())
        .filter_map(|item| {
            let mut parts = Vec::new();
            collect_text(item, &mut parts);
            let text = parts.join(" ").trim().to_string();
            (!text.is_empty()).then(|| HowToStep {
                text,
                image: first_image(item),
            })
        })
        .collect()
}

/// Le `src` du premier nœud image : une étape montre un schéma, pas une galerie.
fn first_image(node: &Value) -> Option<String> {
    if node["type"] == "image" {
        let src = node["attrs"]["src"]
            .as_str()
            .map(str::trim)
            .unwrap_or_default();
        if !src.is_empty() {
            return Some(src.to_string());
        }
    }
    node["content"].as_array()?.iter().find_map(first_image)
}

fn collect_text(node: &Value, out: &mut Vec<String>) {
    if let Some(text) = node["text"].as_str().filter(|t| !t.is_empty()) {
        out.push(text.to_string());
    }
    for child in node["content"].as_array().into_iter().flatten() {
        collect_text(child, out);
    }
}

/// YouTube, Vimeo, Google Drive ou un `.mp4` — vide aussi : pas de vidéo n'est pas une faute.
pub fn is_video_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return url.is_empty();
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = host.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    matches!(
        host,
        "youtube.com"
            | "m.youtube.com"
            | "youtu.be"
            | "vimeo.com"
            | "player.vimeo.com"
            | "drive.google.com"
    ) || path.to_ascii_lowercase().ends_with(".mp4")
}
