use std::{collections::HashMap, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until},
    character::complete::{alpha1, digit1, multispace0},
    combinator::{eof, map_parser, map_res, opt},
    error::context,
    multi::many0,
    IResult,
};
use serde::Serialize;
use tap::TapFallible;

use data::IssuerIdentification;

pub mod data;

#[derive(Debug, Serialize)]
pub struct Data<'a> {
    pub header: Header,
    pub subfiles: HashMap<SubfileType, HashMap<&'a str, Option<&'a str>>>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Header {
    pub issuer_id: u32,
    pub version_number: u8,
    pub jurisdiction_version_number: Option<u8>,
    pub number_of_entries: u8,
    pub subfile_designators: Vec<SubfileDesignator>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SubfileDesignator {
    pub subfile_type: SubfileType,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct DataElement<'a> {
    id: &'a str,
    value: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubfileType {
    DL,
    EN,
    ID,
    JurisdictionSpecific(char),
}

impl std::fmt::Display for SubfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DL => write!(f, "DL"),
            Self::EN => write!(f, "EN"),
            Self::ID => write!(f, "ID"),
            Self::JurisdictionSpecific(c) => write!(f, "Z{c}"),
        }
    }
}

impl Serialize for SubfileType {
    // Serialize into the original string representation, it doesn't need
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();

        serializer.serialize_str(&s)
    }
}

#[derive(Debug)]
pub struct UnknownSubfileType {
    pub data: String,
}

impl std::fmt::Display for UnknownSubfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subfile had unknown type: {}", self.data)
    }
}

impl std::error::Error for UnknownSubfileType {}

impl FromStr for SubfileType {
    type Err = UnknownSubfileType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DL" => Ok(Self::DL),
            "EN" => Ok(Self::EN),
            "ID" => Ok(Self::ID),
            s if s.starts_with("Z") => {
                let c = s.chars().nth(1).ok_or_else(|| UnknownSubfileType {
                    data: s.to_string(),
                })?;
                Ok(Self::JurisdictionSpecific(c))
            }
            _ => Err(UnknownSubfileType {
                data: s.to_string(),
            }),
        }
    }
}

fn parse_header(input: &str) -> IResult<&str, (&str, Header)> {
    let (start, _) = take_until("@")(input)?;
    let (input, _) = context("compliance indicator", tag("@"))(start)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = context("record separator", take_until("A"))(input)?;
    let (input, _) = context("file type", alt((tag("ANSI "), tag("AAMVA"))))(input)?;

    let (input, issuer_id) = context(
        "issuer identification number",
        map_res(map_parser(take(6usize), digit1), |s| {
            u32::from_str_radix(s, 10)
        }),
    )(input)?;

    let issuer = IssuerIdentification::try_from(issuer_id)
        .tap_err(|err| tracing::warn!("could not decode issuer identification number: {err}"))
        .ok();

    let (input, version_number) = context("aamva version number", digit_0_to_99)(input)?;

    let (input, jurisdiction_version_number) = if version_number > 2 {
        let (input, jurisdiction_version_number) =
            context("jurisdiction version number", digit_0_to_99)(input)?;
        (input, Some(jurisdiction_version_number))
    } else {
        (input, None)
    };

    let (input, number_of_entries) = context("number of entries", digit_0_to_99)(input)?;
    let (input, subfile_designators) = context(
        "subfile designators",
        many0(|s| parse_subfile_designator(s, start, issuer, version_number)),
    )(input)?;

    Ok((
        input,
        (
            start,
            Header {
                issuer_id,
                version_number,
                jurisdiction_version_number,
                number_of_entries,
                subfile_designators,
            },
        ),
    ))
}

fn parse_subfile_designator<'a>(
    input: &'a str,
    start: &str,
    issuer: Option<IssuerIdentification>,
    version: u8,
) -> IResult<&'a str, SubfileDesignator> {
    let (input, subfile_type) = context(
        "subfile type",
        map_res(take(2usize), |s: &str| s.parse::<SubfileType>()),
    )(input)?;

    let guess_offset = || {
        let mut offset = 0;
        let matcher = regex_lite::Regex::new(r"(DL|ID)([\d\w]{3,8})(DL|ID|Z\w)([DZ][A-Z]{2})")
            .expect("regex should compile");
        if let Some(m) = matcher.find(start) {
            offset = m.end() as u32 - 5;
        }
        tracing::warn!(
            new_offset = offset,
            "subfile offset was 0, attempted to guess offset"
        );
        offset
    };

    let (input, offset, length) =
        if let Ok((input, _garbage)) = tag::<_, _, nom::error::Error<&str>>("abac")(input) {
            let offset = guess_offset();
            (input, offset, start.len() as u32)
        } else {
            let (input, mut offset) = context("subfile offset", digit_4char)(input)?;

            if version == 1 && issuer == Some(IssuerIdentification::SouthCarolina) && offset == 30 {
                tracing::debug!("applying fix for south carolina offset");
                offset -= 1;
            }

            if offset == 0 {
                offset = guess_offset();
            }

            let (input, length) = context("subfile length", digit_4char)(input)?;

            (input, offset, length)
        };

    Ok((
        input,
        SubfileDesignator {
            subfile_type,
            offset,
            length,
        },
    ))
}

fn parse_data_elements<'a>(
    input: &'a str,
    subfile: SubfileDesignator,
) -> IResult<&'a str, HashMap<&'a str, Option<&'a str>>> {
    let (input, _offset) = take(subfile.offset as usize)(input)?;

    let max_length = std::cmp::min(subfile.length as usize, input.len());

    if max_length != subfile.length as usize {
        tracing::debug!(
            input_len = input.len(),
            subfile_offset = subfile.offset,
            subfile_length = subfile.length,
            clamped_length = max_length,
            "subfile had offset+length that exceeded input length"
        );
    }

    let (_input, element_data) = take(max_length)(input)?;

    let element_data = if matches!(
        subfile.subfile_type,
        SubfileType::DL | SubfileType::EN | SubfileType::ID
    ) {
        let (element_data, _) =
            opt(tag(subfile.subfile_type.to_string().as_bytes()))(element_data)?;
        element_data
    } else {
        element_data
    };

    let (input, elements) =
        many0(|input| parse_data_element(input, subfile.subfile_type))(element_data)?;

    let elements = elements
        .into_iter()
        .map(|elem| (elem.id, elem.value.map(str::trim)))
        .collect();

    Ok((input, elements))
}

fn parse_data_element<'a>(
    input: &'a str,
    subfile_type: SubfileType,
) -> IResult<&'a str, DataElement<'a>> {
    let prefix = match subfile_type {
        SubfileType::DL | SubfileType::EN | SubfileType::ID => "D".to_string(),
        SubfileType::JurisdictionSpecific(c) => format!("Z{c}"),
    };

    // Get the 3-letter ID for this element.
    let (input, id) = map_parser(take(3usize), alpha1)(input)?;

    if !id.starts_with(&prefix) {
        tracing::warn!("element in subfile {subfile_type} had wrong ID prefix: {id}");
    }

    // Take values until we reach a terminator, then take the terminator.
    let (input, value) = take_till(|s| matches!(s, '\r' | '\n'))(input)?;
    let (input, _) = alt((take(1usize), eof))(input)?;

    let value = match value.trim() {
        "NONE" | "unavl" | "" => None,
        value => Some(value),
    };

    Ok((input, DataElement { id, value }))
}

pub fn parse_barcode<'a>(input: &'a str) -> Result<Data<'a>, nom::Err<nom::error::Error<&'a str>>> {
    let (_trailing, (start, header)) = parse_header(input)?;

    let subfiles = header
        .subfile_designators
        .clone()
        .into_iter()
        .flat_map(|desginator| {
            let subfile_type = desginator.subfile_type;
            parse_data_elements(start, desginator)
                .map(|(_input, elements)| (subfile_type, elements))
                .tap_err(|err| tracing::warn!(%subfile_type, "subfile could not be parsed: {err}"))
                .ok()
        })
        .collect::<HashMap<_, _>>();

    Ok(Data { header, subfiles })
}

fn digit_0_to_99(input: &str) -> IResult<&str, u8> {
    map_res(map_parser(take(2usize), digit1), |s| {
        u8::from_str_radix(s, 10)
    })(input)
}

fn digit_4char(input: &str) -> IResult<&str, u32> {
    map_res(map_parser(take(4usize), digit1), |s| {
        u32::from_str_radix(s, 10)
    })(input)
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::path::PathBuf;
    use std::sync::Once;

    use super::*;

    static LICENSE_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/licenses");

    fn get_test_files() -> impl Iterator<Item = walkdir::DirEntry> {
        let path = if let Some(path) = std::env::var_os("AAMVA_TEST_FOLDER") {
            PathBuf::from(path)
        } else {
            PathBuf::from(LICENSE_FOLDER)
        };

        walkdir::WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("txt"))
                    .unwrap_or_default()
            })
    }

    static SETUP: Once = Once::new();

    fn init_subscriber() {
        SETUP.call_once(|| {
            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
        });
    }

    #[test]
    fn test_parse_data_element() {
        let cases = [
            (
                (SubfileType::DL, "DAQ0123456789ABC\rtest"),
                (
                    "test",
                    DataElement {
                        id: "DAQ",
                        value: Some("0123456789ABC"),
                    },
                ),
            ),
            (
                (SubfileType::DL, "DAQ0123456789ABC"),
                (
                    "",
                    DataElement {
                        id: "DAQ",
                        value: Some("0123456789ABC"),
                    },
                ),
            ),
        ];

        for ((prefix, input), expected_output) in cases {
            let actual_output = parse_data_element(input, prefix).unwrap();
            assert_eq!(actual_output, expected_output);
        }
    }

    #[test]
    fn test_parse_header() {
        let cases = [(
            "@\n\x1e\rAAMVA6360000102DL00390188ZV02270031ANSI ",
            (
                "ANSI ",
                (
                    "@\n\x1e\rAAMVA6360000102DL00390188ZV02270031ANSI ",
                    Header {
                        issuer_id: 636000,
                        version_number: 1,
                        number_of_entries: 2,
                        jurisdiction_version_number: None,
                        subfile_designators: vec![
                            SubfileDesignator {
                                subfile_type: SubfileType::DL,
                                offset: 39,
                                length: 188,
                            },
                            SubfileDesignator {
                                subfile_type: SubfileType::JurisdictionSpecific('V'),
                                offset: 227,
                                length: 31,
                            },
                        ],
                    },
                ),
            ),
        )];

        for (input, expected_output) in cases {
            let actual_output = parse_header(input).unwrap();
            assert_eq!(actual_output, expected_output);
        }
    }

    #[test]
    fn it_works() {
        init_subscriber();

        for entry in get_test_files() {
            let _guard =
                tracing::info_span!("entry_parse", path = %entry.path().display()).entered();

            let mut f = std::fs::File::open(entry.path()).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();

            match parse_barcode(&s) {
                Ok(data) => {
                    // tracing::debug!(?data);
                    // let mut out = String::new();
                    // for (subfile_type, elements) in data.subfiles {
                    //     write!(out, "=== {subfile_type} ===\n").unwrap();
                    //     if elements.is_empty() {
                    //         write!(out, "NO ELEMENTS FOUND\n").unwrap();
                    //     }
                    //     for (id, value) in elements {
                    //         write!(out, "{id}: {}\n", value.unwrap_or("N/A")).unwrap();
                    //     }
                    // }
                    // tracing::trace!("decoded data:\n{out}");
                    tracing::info!(name = ?data.name(), birthday = ?data.date_of_birth());
                }
                Err(err) => assert!(false, "all licenses should parse: {err}"),
            }
        }
    }
}
