//! OCRモジュール - Apple Vision APIを使用したテキスト認識

use crate::error::OcrError;
use std::path::Path;
use std::process::Command;

/// Apple Vision APIを使用してOCRを実行
pub fn recognize_text(image_path: &Path) -> Result<String, OcrError> {
    if !image_path.exists() {
        return Err(OcrError::ImageNotFound(
            image_path.to_string_lossy().to_string(),
        ));
    }

    // osascript経由でSwiftのVision APIを呼び出す
    let script = format!(
        r#"
use framework "Vision"
use framework "AppKit"
use scripting additions

set imagePath to "{}"
set theImage to current application's NSImage's alloc()'s initWithContentsOfFile:imagePath

if theImage is missing value then
    return "ERROR: Could not load image"
end if

set requestHandler to current application's VNImageRequestHandler's alloc()'s initWithData:(theImage's TIFFRepresentation()) options:(current application's NSDictionary's dictionary())

set textRequest to current application's VNRecognizeTextRequest's alloc()'s init()
textRequest's setRecognitionLevel:(current application's VNRequestTextRecognitionLevelAccurate)
textRequest's setRecognitionLanguages:{{"ja", "en"}}
textRequest's setUsesLanguageCorrection:true

set {{theResult, theError}} to requestHandler's performRequests:({{textRequest}}) |error|:(reference)

if theError is not missing value then
    return "ERROR: " & (theError's localizedDescription() as text)
end if

set recognizedTexts to {{}}
set observations to textRequest's results()

repeat with observation in observations
    set topCandidate to (observation's topCandidates:1)'s firstObject()
    if topCandidate is not missing value then
        set end of recognizedTexts to (topCandidate's |string|() as text)
    end if
end repeat

set AppleScript's text item delimiters to linefeed
return recognizedTexts as text
"#,
        image_path.to_string_lossy().replace('"', r#"\""#)
    );

    let output = Command::new("osascript")
        .arg("-l")
        .arg("AppleScript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| OcrError::ExecutionFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OcrError::ExecutionFailed(stderr.to_string()));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.starts_with("ERROR:") {
        return Err(OcrError::ExecutionFailed(text));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_recognize_text_missing_file() {
        let result = recognize_text(&PathBuf::from("/nonexistent/image.jpg"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OcrError::ImageNotFound(_)));
    }
}
