use std::{error::Error, process::{self, ExitCode}};
//TODO: Replace dyn Error with custom error hierachy and Into/From magic to get auto conversion

fn winget_update_raw() -> Result<String, Box<dyn Error>> {
    let output =
        process::Command::new("winget")
        .arg("update")
        .output()?;
    let utf8output = String::from_utf8(output.stdout)?;
    Ok(utf8output)
}


#[derive(Debug, PartialEq, Eq)]
enum ParsePackageSourceError {
    UnexpectedInput {input:String}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PackageSource {
    ///[winget]
    WinGet,
    ///[msstore]
    MsStore,
}

impl TryFrom<String> for PackageSource {
    type Error = ParsePackageSourceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "winget" => Ok(PackageSource::WinGet),
            "msstore" => Ok(PackageSource::MsStore),
            rest => Err(ParsePackageSourceError::UnexpectedInput {input: rest.to_string()})
        }
    }
}


//Name                                                        Id                               Version        Available     Source
#[derive(Debug,PartialEq, Eq, PartialOrd, Ord)]
struct WingetUpdateOutput {
    /// [Name] Package name
    name:String,
    /// [Id] Package id
    id:String,
    /// [Version] Currently installed version
    version:String,
    /// [Available] Newest available version
    available:String,
    /// [Source] Package source
    source:PackageSource
}

fn parse_winget_raw(s:String) -> Result<Vec<WingetUpdateOutput>, Box<dyn Error>> {
    let mut lines = s.lines();
    match lines.next() {
        Some(header_line) => {
            let positions =
                parse_positions_from_header(header_line)
                .ok_or("parse_positions_from_header error TODO")?;

            let _ = lines.next();//skip ----------- row

            let mut result = Vec::new();
            for row in lines {
                if is_end_row(row) {break;}
                let name_str = row.get(positions.name_offset..positions.id_offset)
                    .unwrap_or("").trim().to_string();
                let id_str = row.get(positions.id_offset..positions.version_offset)
                    .unwrap_or("").trim().to_string();
                let version_str = row.get(positions.version_offset..positions.available_offset)
                    .unwrap_or("").trim().to_string();
                let available_str = row.get(positions.available_offset..positions.source_offset)
                    .unwrap_or("").trim().to_string();
                let source_str = row.get(positions.source_offset..)
                    .unwrap_or("").trim().to_string();
                let entry = WingetUpdateOutput {
                    name: name_str,
                    id: id_str,
                    version: version_str,
                    available: available_str,
                    source: PackageSource::try_from(source_str)
                        .map_err(|e| format!("Failed to parse package source {:?}", e))? };
                result.push(entry);
            }
            Ok(result)
        },
        None => Ok(Vec::new())
    }
    //TODO ingest headers line and establish columns start end
    // process rows by cuting on boundaries from previous step
    // trim whitespace
}

fn is_end_row(row: &str) -> bool {
    return row.contains("upgrades available");
}

#[derive(Debug, Clone, Copy)]
struct Positions {
    name_offset:usize,
    id_offset:usize,
    version_offset:usize,
    available_offset:usize,
    source_offset:usize,
}

fn parse_positions_from_header(header_line: &str) -> Option<Positions> {
    let lower = header_line.to_lowercase();
    Some(Positions {
        name_offset: lower.find("name")?,
        id_offset: lower.find("id")?,
        version_offset: lower.find("version")?,
        available_offset: lower.find("available")?,
        source_offset: lower.find("source")?
    })
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    println!("Starting main!");
    let utf8output = winget_update_raw()?;
    let packages = parse_winget_raw(utf8output)?;
    for package in &packages {
        println!("{:?}", package);
    }
    Ok(ExitCode::SUCCESS)
}
