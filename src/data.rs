use std::ops::Not;

use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use tap::TapFallible;
use time::{format_description::FormatItem, macros::format_description, Date};

use crate::{Data, SubfileType};

const YMD_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
time::serde::format_description!(ymd_format, Date, YMD_FORMAT);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IssuerCountry {
    #[default]
    UnitedStates,
    Canada,
    Mexico,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize)]
#[repr(u32)]
pub enum IssuerIdentification {
    Alabama = 636033,
    Alaska = 636059,
    #[num_enum(alternatives = [990876])]
    Alberta = 604432,
    AmericanSamoa = 604427,
    Arizona = 636026,
    Arkansas = 636021,
    BritishColumbia = 636028,
    California = 636014,
    Coahuila = 636056,
    Colorado = 636020,
    Connecticut = 636006,
    Delaware = 636011,
    DistrictOfColumbia = 636043,
    Florida = 636010,
    Georgia = 636055,
    Guam = 636019,
    Hawaii = 636047,
    Hidalgo = 636057,
    Idaho = 636050,
    Illinois = 636035,
    Indiana = 636037,
    Iowa = 636018,
    Kansas = 636022,
    Kentucky = 636046,
    Louisiana = 636007,
    Maine = 636041,
    Manitoba = 636048,
    Maryland = 636003,
    Massachusetts = 636002,
    Michigan = 636032,
    Minnesota = 636038,
    Mississippi = 636051,
    Missouri = 636030,
    Montana = 636008,
    Nebraska = 636054,
    Nevada = 636049,
    NewBrunswick = 636017,
    Newfoundland = 636016,
    NewHampshire = 636039,
    NewJersey = 636036,
    NewMexico = 636009,
    NewYork = 636001,
    NorthCarolina = 636004,
    NorthDakota = 636034,
    NortherMariannaIslands = 604430,
    NorthwestTerritories = 604434,
    NovaScotia = 636013,
    Nunavut = 604433,
    Ohio = 636023,
    Oklahoma = 636058,
    Ontario = 636012,
    Oregon = 636029,
    Pennsylvania = 636025,
    PrinceEdwardIsland = 604426,
    PuertoRico = 604431,
    Quebec = 604428,
    RhodeIsland = 636052,
    Saskatchewan = 636044,
    SouthCarolina = 636005,
    SouthDakota = 636042,
    StateDepartment = 636027,
    Tennessee = 636053,
    Texas = 636015,
    Utah = 636040,
    Vermont = 636024,
    Virginia = 636000,
    VirginIslands = 636062,
    Washington = 636045,
    WestVirginia = 636061,
    Wisconsin = 636031,
    Wyoming = 636060,
    Yukon = 604429,
}

impl IssuerIdentification {
    pub fn country(&self) -> IssuerCountry {
        use IssuerIdentification::*;

        match self {
            PrinceEdwardIsland | Quebec | Yukon | Alberta | Nunavut | NorthwestTerritories
            | Ontario | NovaScotia | Newfoundland | NewBrunswick | BritishColumbia
            | Saskatchewan | Manitoba => IssuerCountry::Canada,
            Coahuila | Hidalgo => IssuerCountry::Mexico,
            _ => IssuerCountry::UnitedStates,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedData {
    pub issuer_id: u32,
    pub aamva_version: u8,
    pub jurisdiction_version_number: Option<u8>,
    #[serde(with = "ymd_format::option")]
    pub document_expiration_date: Option<Date>,
    pub name: Option<Name>,
    #[serde(with = "ymd_format::option")]
    pub document_issue_date: Option<Date>,
    #[serde(with = "ymd_format::option")]
    pub date_of_birth: Option<Date>,
    pub sex: Option<Sex>,
    pub eye_color: Option<EyeColor>,
    pub height: Option<Height>,
    pub address: Option<Address>,
    pub customer_id_number: Option<String>,
    pub document_discriminator: Option<String>,
    pub country: Option<IssuerCountry>,
    pub hair_color: Option<HairColor>,
    pub place_of_birth: Option<String>,
    pub audit_information: Option<String>,
    pub inventory_control_information: Option<String>,
    pub weight: Option<Weight>,
    pub race: Option<Race>,
    #[serde(with = "ymd_format::option")]
    pub card_revision_date: Option<Date>,
    pub under_age_until: UnderAgeUntil,
}

impl From<Data<'_>> for DecodedData {
    fn from(value: Data<'_>) -> Self {
        Self {
            issuer_id: value.header.issuer_id,
            aamva_version: value.header.version_number,
            jurisdiction_version_number: value.header.jurisdiction_version_number,
            name: value.name(),
            document_expiration_date: value.document_expiration_date(),
            date_of_birth: value.date_of_birth(),
            document_issue_date: value.document_issue_date(),
            sex: value.sex(),
            eye_color: value.eye_color(),
            height: value.height(),
            address: value.address(),
            customer_id_number: value.customer_id_number(),
            document_discriminator: value.document_discriminator(),
            country: value.country(),
            hair_color: value.hair_color(),
            place_of_birth: value.place_of_birth(),
            audit_information: value.audit_information(),
            inventory_control_information: value.inventory_control_information(),
            weight: value.weight(),
            race: value.race(),
            card_revision_date: value.card_revision_date(),
            under_age_until: value.under_age_until(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub family: String,
    pub first: String,
    pub middle: Option<String>,

    pub prefix: Option<String>,
    pub suffix: Option<String>,

    pub alias_family: Option<String>,
    pub alias_given: Option<String>,
    pub alias_suffix: Option<String>,

    pub family_truncation: Option<Truncation>,
    pub first_truncation: Option<Truncation>,
    pub middle_truncation: Option<Truncation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Truncation {
    Truncated,
    NotTruncated,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sex {
    Male,
    Female,
    NotSpecified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: String,
    pub jurisdiction_code: String,
    pub postal_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EyeColor {
    Black,
    Blue,
    Brown,
    Dichromatic,
    Gray,
    Green,
    Hazel,
    Maroon,
    Pink,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HairColor {
    Bald,
    Black,
    Blond,
    Brown,
    Gray,
    RedAuburn,
    Sandy,
    White,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Height {
    Inches(u16),
    Centimeters(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weight {
    Pounds(u16),
    Kilograms(u16),
    KilogramRange { from: u8, to: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    AlaskanAmericanIndian,
    AsianPacificIslander,
    Black,
    HispanicOrigin,
    NonHispanic,
    Unknown,
    White,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderAgeUntil {
    #[serde(with = "ymd_format::option")]
    pub under_18_until: Option<Date>,
    #[serde(with = "ymd_format::option")]
    pub under_19_until: Option<Date>,
    #[serde(with = "ymd_format::option")]
    pub under_21_until: Option<Date>,
}

fn filter_empty_str<S>(input: S) -> Option<S>
where
    S: AsRef<str>,
{
    input.as_ref().is_empty().not().then_some(input)
}

impl<'a> Data<'a> {
    pub fn name(&self) -> Option<Name> {
        match self.header.version_number {
            ..=1 => {
                if let Some(family) = self.get_field_owned("DAB") {
                    let first = self.get_field_owned("DAC")?;
                    let middle = self.get_field_owned("DAD");
                    let suffix = self.get_field_owned("DAE");
                    let prefix = self.get_field_owned("DAF");

                    Some(Name {
                        family,
                        first,
                        middle,
                        suffix,
                        prefix,
                        alias_family: None,
                        alias_given: None,
                        alias_suffix: None,
                        family_truncation: None,
                        first_truncation: None,
                        middle_truncation: None,
                    })
                } else {
                    let field = self.get_field("DAA")?;
                    let split = if field.contains(',') { ',' } else { ' ' };
                    let mut parts = field.split(split);
                    let family = parts.next()?.to_string();
                    let first = parts.next()?.to_string();
                    let middle = filter_empty_str(parts.join(" "));

                    Some(Name {
                        family,
                        first,
                        middle,
                        suffix: None,
                        prefix: None,
                        alias_family: None,
                        alias_given: None,
                        alias_suffix: None,
                        family_truncation: None,
                        first_truncation: None,
                        middle_truncation: None,
                    })
                }
            }
            2..=3 => {
                let names = self.get_field("DCT")?;

                let split = if names.contains(',') { ',' } else { ' ' };
                let mut parts = names.split(split);

                let first = parts.next()?.to_string();
                let middle = filter_empty_str(parts.join(" "));

                Some(Name {
                    family: self.get_field_owned("DCS")?,
                    first,
                    middle,
                    suffix: None,
                    prefix: None,
                    alias_family: None,
                    alias_given: None,
                    alias_suffix: None,
                    family_truncation: None,
                    first_truncation: None,
                    middle_truncation: None,
                })
            }
            4.. => Some(Name {
                family: self.get_field_owned("DCS")?,
                first: self.get_field_owned("DAC")?,
                middle: self.get_field_owned("DAD"),
                suffix: self.get_field_owned("DCU"),
                prefix: None,
                alias_family: self.get_field_owned("DBN"),
                alias_given: self.get_field_owned("DBG"),
                alias_suffix: self.get_field_owned("DBS"),
                family_truncation: self.get_field("DDE").and_then(Self::parse_truncation),
                first_truncation: self.get_field("DDF").and_then(Self::parse_truncation),
                middle_truncation: self.get_field("DDG").and_then(Self::parse_truncation),
            }),
        }
    }

    pub fn document_expiration_date(&self) -> Option<Date> {
        self.date_field("DBA")
    }

    pub fn date_of_birth(&self) -> Option<Date> {
        self.date_field("DBB")
    }

    pub fn document_issue_date(&self) -> Option<Date> {
        self.date_field("DBD")
    }

    pub fn sex(&self) -> Option<Sex> {
        use Sex::*;

        let sex = match self.get_field("DBC")?.to_ascii_uppercase().as_str() {
            "1" | "M" => Male,
            "2" | "F" => Female,
            "9" | "X" => NotSpecified,
            _ => return None,
        };

        Some(sex)
    }

    pub fn eye_color(&self) -> Option<EyeColor> {
        use EyeColor::*;

        let color = match self.get_field("DAY")?.to_ascii_uppercase().as_str() {
            "BLK" => Black,
            "BLU" => Blue,
            "BRO" => Brown,
            "DIC" => Dichromatic,
            "GRN" => Green,
            "GRY" => Gray,
            "HAZ" => Hazel,
            "MAR" => Maroon,
            "PNK" => Pink,
            "UNK" => Unknown,
            _ => return None,
        };

        Some(color)
    }

    pub fn height(&self) -> Option<Height> {
        let height = self.get_field("DAU")?.to_ascii_lowercase();

        let parse_hyphenated_ftin = |feet: &str, inches: &str| {
            let feet: u16 = feet.strip_suffix('\'').unwrap_or(feet).parse().ok()?;
            let inches: u16 = inches.strip_suffix('"').unwrap_or(inches).parse().ok()?;
            Some(Height::Inches(feet * 12 + inches))
        };

        if let Some(centimeters) = height.strip_suffix(" cm") {
            let centimeters = centimeters[..3].parse().ok()?;
            Some(Height::Centimeters(centimeters))
        } else if let Some(inches) = height.strip_suffix(" in") {
            let inches = inches[..3].parse().ok()?;
            Some(Height::Inches(inches))
        } else if height.len() == 3 {
            let feet: u16 = height[..1].parse().ok()?;
            let inches: u16 = height[1..=2].parse().ok()?;
            Some(Height::Inches(feet * 12 + inches))
        } else if let Some((feet, inches)) = height.split_once('-') {
            parse_hyphenated_ftin(feet, inches)
        } else if let Some(centimeters) = self.get_field("DAV") {
            centimeters.parse().ok().map(Height::Centimeters)
        } else if let Some(Some(height)) = self
            .subfiles
            .get(&SubfileType::JurisdictionSpecific('I'))
            .and_then(|subfile| subfile.get("ZIJ"))
        {
            if let Some((feet, inches)) = height.split_once('-') {
                parse_hyphenated_ftin(feet, inches)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn address(&self) -> Option<Address> {
        Some(Address {
            address_1: self.get_field_owned("DAG")?,
            address_2: self.get_field_owned("DAH"),
            city: self.get_field_owned("DAI")?,
            jurisdiction_code: self.get_field_owned("DAJ")?,
            postal_code: self.get_field_owned("DAK")?,
        })
    }

    pub fn customer_id_number(&self) -> Option<String> {
        self.get_field_owned("DAQ")
    }

    pub fn document_discriminator(&self) -> Option<String> {
        self.get_field_owned("DCF")
    }

    pub fn country(&self) -> Option<IssuerCountry> {
        let issuer = match self
            .get_field("DCG")
            .map(str::to_ascii_uppercase)
            .as_deref()
        {
            Some("USA") => IssuerCountry::UnitedStates,
            Some("CAN") => IssuerCountry::Canada,
            Some("MEX") => IssuerCountry::Mexico,
            Some(_) => return None,
            _ => match self.height() {
                Some(Height::Inches(_)) => IssuerCountry::UnitedStates,
                Some(Height::Centimeters(_)) => IssuerCountry::Canada,
                _ => return None,
            },
        };

        Some(issuer)
    }

    pub fn hair_color(&self) -> Option<HairColor> {
        use HairColor::*;

        let color = match self.get_field("DAZ")?.to_ascii_uppercase().as_str() {
            "BAL" => Bald,
            "BLK" => Black,
            "BLN" => Blond,
            "BRO" => Brown,
            "GRY" => Gray,
            "RED" => RedAuburn,
            "SDY" => Sandy,
            "WHI" => White,
            "UNK" => Unknown,
            _ => return None,
        };

        Some(color)
    }

    pub fn place_of_birth(&self) -> Option<String> {
        self.get_field_owned("DCI")
    }

    pub fn audit_information(&self) -> Option<String> {
        self.get_field_owned("DCJ")
    }

    pub fn inventory_control_information(&self) -> Option<String> {
        self.get_field_owned("DCK")
    }

    pub fn weight(&self) -> Option<Weight> {
        use Weight::KilogramRange;

        if let Some(pounds) = self.get_field("DAW") {
            pounds.parse().ok().map(Weight::Pounds)
        } else if let Some(kilograms) = self.get_field("DAX") {
            kilograms.parse().ok().map(Weight::Kilograms)
        } else if let Some(range) = self.get_field("DCE") {
            Some(match range {
                "0" => KilogramRange { from: 0, to: 31 },
                "1" => KilogramRange { from: 32, to: 45 },
                "2" => KilogramRange { from: 46, to: 59 },
                "3" => KilogramRange { from: 60, to: 70 },
                "4" => KilogramRange { from: 71, to: 86 },
                "5" => KilogramRange { from: 87, to: 100 },
                "6" => KilogramRange { from: 101, to: 113 },
                "7" => KilogramRange { from: 114, to: 127 },
                "8" => KilogramRange { from: 128, to: 145 },
                "9" => KilogramRange {
                    from: 146,
                    to: u8::MAX,
                },
                _ => return None,
            })
        } else {
            None
        }
    }

    pub fn race(&self) -> Option<Race> {
        use Race::*;

        let race = match self.get_field("DCL")?.to_ascii_uppercase().as_str() {
            "AI" => AlaskanAmericanIndian,
            "AP" => AsianPacificIslander,
            "BK" => Black,
            "H" => HispanicOrigin,
            "O" => NonHispanic,
            "U" => Unknown,
            "W" => White,
            _ => return None,
        };

        Some(race)
    }

    pub fn card_revision_date(&self) -> Option<Date> {
        self.date_field("DDB")
    }

    pub fn under_age_until(&self) -> UnderAgeUntil {
        UnderAgeUntil {
            under_18_until: self.under_n_until("DDH", 18),
            under_19_until: self.under_n_until("DDH", 19),
            under_21_until: self.under_n_until("DDH", 21),
        }
    }

    fn date_field(&self, name: &str) -> Option<Date> {
        let country = IssuerIdentification::try_from(self.header.issuer_id)
            .map(|issuer| issuer.country())
            .unwrap_or_default();

        let field = self.get_field(name)?;

        self.parse_date(field, country)
    }

    #[tracing::instrument(skip(self))]
    fn parse_date(&self, input: &str, country: IssuerCountry) -> Option<Date> {
        if input.len() != 8 {
            tracing::warn!("date was incorrect length: {input}");
            return None;
        }

        let input = &input[..8];

        const MDY: &[FormatItem<'_>] = format_description!("[month][day][year]");
        const YMD: &[FormatItem<'_>] = format_description!("[year][month][day]");

        if country == IssuerCountry::UnitedStates && self.header.version_number != 1 {
            match Date::parse(input, &MDY) {
                Err(_err) => Date::parse(input, &YMD),
                date => date,
            }
        } else {
            Date::parse(input, &YMD)
        }
        .tap_err(|err| tracing::warn!("could not parse date {input} ({country:?}): {err}"))
        .ok()
    }

    fn parse_truncation(input: &str) -> Option<Truncation> {
        match input.to_ascii_uppercase().as_str() {
            "T" => Some(Truncation::Truncated),
            "N" => Some(Truncation::NotTruncated),
            "U" => Some(Truncation::Unknown),
            _ => None,
        }
    }

    #[tracing::instrument(skip(self))]
    fn under_n_until(&self, name: &str, age: i32) -> Option<Date> {
        if let Some(date) = self.date_field(name) {
            return Some(date);
        }

        let (year, day_of_year) = self.date_of_birth()?.to_ordinal_date();
        let future_year = year + age;

        // We need to handle leap year birthdays here.
        let day_of_year = if day_of_year > 60 {
            let year_is_leap = time::util::is_leap_year(year);
            let future_year_is_leap = time::util::is_leap_year(future_year);

            match (year_is_leap, future_year_is_leap) {
                // Both or neither years are leap years, numbers are the same.
                (true, true) | (false, false) => day_of_year,
                // Only current year is leap year, subtract one.
                (true, false) => day_of_year - 1,
                // Only future year is leap year, add one.
                (false, true) => day_of_year + 1,
            }
        } else {
            day_of_year
        };

        Date::from_ordinal_date(future_year, day_of_year)
            .tap_err(|err| tracing::error!("could not calculate: {err}"))
            .ok()
    }

    /// Attempt to get a field from known subfile types.
    fn get_field(&self, name: &str) -> Option<&'a str> {
        [SubfileType::DL, SubfileType::EN, SubfileType::ID]
            .into_iter()
            .find_map(|subfile_type| {
                self.subfiles
                    .get(&subfile_type)
                    .and_then(|subfile| subfile.get(name).cloned().flatten().map(str::trim))
            })
    }

    fn get_field_owned(&self, name: &str) -> Option<String> {
        self.get_field(name).map(str::to_string)
    }
}
