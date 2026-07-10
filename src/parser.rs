use crate::error::{ParsePackageSourceError, ParseWingetError};

/// A single row parsed from `winget upgrade` tabular output.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WingetUpdateOutput {
    /// Display name of the package (the "Name" column).
    pub name: String,
    /// Unique package identifier (the "Id" column).
    pub id: String,
    /// Currently installed version (the "Version" column).
    pub version: String,
    /// Newest version available in the source (the "Available" column).
    pub available: String,
    /// Repository that provides the package (the "Source" column).
    pub source: PackageSource,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageSource {
    /// [winget]
    WinGet,
    /// [msstore]
    MsStore,
}

impl TryFrom<String> for PackageSource {
    type Error = ParsePackageSourceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "winget" => Ok(PackageSource::WinGet),
            "msstore" => Ok(PackageSource::MsStore),
            rest => Err(ParsePackageSourceError {
                input: rest.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Positions {
    name_offset: usize,
    id_offset: usize,
    version_offset: usize,
    available_offset: usize,
    source_offset: usize,
}

fn parse_positions_from_header(header_line: &str) -> Option<Positions> {
    let lower = header_line.to_lowercase();
    Some(Positions {
        name_offset: lower.find("name")?,
        id_offset: lower.find("id")?,
        version_offset: lower.find("version")?,
        available_offset: lower.find("available")?,
        source_offset: lower.find("source")?,
    })
}

fn is_end_row(row: &str) -> bool {
    row.contains("upgrades available")
}

pub fn parse_winget_raw(s: &str) -> Result<Vec<WingetUpdateOutput>, ParseWingetError> {
    let mut lines = s.lines();
    let header_line = match lines.next() {
        Some(line) => line,
        None => return Ok(Vec::new()),
    };

    let positions =
        parse_positions_from_header(header_line).ok_or(ParseWingetError::HeaderParseFailed)?;

    let _ = lines.next(); // skip -------- separator row

    let mut result = Vec::new();
    for row in lines {
        if is_end_row(row) {
            break;
        }
        let name = row
            .get(positions.name_offset..positions.id_offset)
            .unwrap_or("")
            .trim()
            .to_string();
        let id = row
            .get(positions.id_offset..positions.version_offset)
            .unwrap_or("")
            .trim()
            .to_string();
        let version = row
            .get(positions.version_offset..positions.available_offset)
            .unwrap_or("")
            .trim()
            .to_string();
        let available = row
            .get(positions.available_offset..positions.source_offset)
            .unwrap_or("")
            .trim()
            .to_string();
        let source_str = row
            .get(positions.source_offset..)
            .unwrap_or("")
            .trim()
            .to_string();
        let source = PackageSource::try_from(source_str)?;
        result.push(WingetUpdateOutput {
            name,
            id,
            version,
            available,
            source,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Name                                                        Id                               Version        Available     Source
---------------------------------------------------------------------------------------------------------------------------------
7-Zip 26.01 (x64)                                           7zip.7zip                        26.01          26.02         winget
AWS Command Line Interface v2                               Amazon.AWSCLI                    2.35.9.0       2.35.15       winget
AWS SAM Command Line Interface                              Amazon.SAM-CLI                   1.158.0        1.163.0       winget
balenaEtcher                                                Balena.Etcher                    2.1.4          2.1.6         winget
Oh My Posh                                                  XP8K0HKJFRXGCK                   29.17.0        29.20.0       msstore
20 upgrades available.";

    #[test]
    fn test_parse_positions_from_header() {
        let header = "Name                                                        Id                               Version        Available     Source";
        let pos = parse_positions_from_header(header).expect("should find positions");
        assert_eq!(pos.name_offset, 0);
        assert!(pos.id_offset > pos.name_offset);
        assert!(pos.version_offset > pos.id_offset);
        assert!(pos.available_offset > pos.version_offset);
        assert!(pos.source_offset > pos.available_offset);
    }

    #[test]
    fn test_parse_positions_empty() {
        assert!(parse_positions_from_header("").is_none());
        assert!(parse_positions_from_header("garbage line").is_none());
    }

    #[test]
    fn test_is_end_row() {
        assert!(is_end_row("20 upgrades available."));
        assert!(is_end_row("1 upgrades available."));
        assert!(!is_end_row(
            "7-Zip 26.01 (x64)  7zip.7zip  26.01  26.02  winget"
        ));
    }

    #[test]
    fn test_package_source_try_from() {
        assert_eq!(
            PackageSource::try_from("winget".to_string()),
            Ok(PackageSource::WinGet)
        );
        assert_eq!(
            PackageSource::try_from("msstore".to_string()),
            Ok(PackageSource::MsStore)
        );
        assert!(PackageSource::try_from("unknown".to_string()).is_err());
    }

    #[test]
    fn test_parse_winget_raw_empty() {
        let result = parse_winget_raw("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_winget_raw_full_sample() {
        let result = parse_winget_raw(SAMPLE).unwrap();
        assert_eq!(result.len(), 5);

        assert_eq!(result[0].name, "7-Zip 26.01 (x64)");
        assert_eq!(result[0].id, "7zip.7zip");
        assert_eq!(result[0].version, "26.01");
        assert_eq!(result[0].available, "26.02");
        assert_eq!(result[0].source, PackageSource::WinGet);

        assert_eq!(result[4].name, "Oh My Posh");
        assert_eq!(result[4].id, "XP8K0HKJFRXGCK");
        assert_eq!(result[4].source, PackageSource::MsStore);
    }

    #[test]
    fn test_parse_winget_raw_missing_header() {
        // A line that exists but lacks the required column names
        let input = "garbage\n---\nsome row\n";
        let result = parse_winget_raw(input);
        assert!(matches!(result, Err(ParseWingetError::HeaderParseFailed)));
    }

    #[test]
    fn test_parse_winget_raw_bad_source() {
        let input = "\
Name                                                        Id                               Version        Available     Source
---------------------------------------------------------------------------------------------------------------------------------
7-Zip 26.01 (x64)                                           7zip.7zip                        26.01          26.02         badvalue
1 upgrades available.";
        let result = parse_winget_raw(input);
        assert!(matches!(result, Err(ParseWingetError::InvalidSource(_))));
    }
}
