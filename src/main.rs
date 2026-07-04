use std::{error::Error, ops::Index, process::{self, ExitCode}};
//TODO: Replace dyn Error with custom error hierachy and Into/From magic to get auto conversion

fn winget_update_raw() -> Result<String, Box<dyn Error>> {
    let output =
        process::Command::new("winget")
        .arg("update")
        .output()?;
    let utf8output = String::from_utf8(output.stdout)?;
    Ok(utf8output)
}


enum ParsePackageSourceError {
    UnexpectedInput {input:String}
}

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
        Some(headerLine) => {
            let positions =
                parse_positions_from_header(headerLine)
                .ok_or("parse_positions_from_header error TODO")?;
            let result = Vec::new();
            for row in lines {
                let entry = WingetUpdateOutput {
                    name: todo!(),
                    id: todo!(),
                    version: todo!(),
                    available: todo!(),
                    source: todo!() };
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

struct Positions {
    name_offset:usize,
    id_offset:usize,
    version_offset:usize,
    available_offset:usize,
    source_offset:usize,
}

fn parse_positions_from_header(header_line: &str) -> Option<Positions> {
    Some(Positions {
        name_offset: header_line.find("name")?,
        id_offset: header_line.find("id")?,
        version_offset: header_line.find("version")?,
        available_offset: header_line.find("available")?,
        source_offset: header_line.find("source")?
    })
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    println!("Starting main!");
    let utf8output = winget_update_raw()?;
    println!("{}", utf8output);
    Ok(ExitCode::SUCCESS)
}
