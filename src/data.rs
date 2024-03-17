use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use tap::TapFallible;
use time::macros::format_description;

use crate::{Data, SubfileType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssuerCountry {
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
pub struct Name {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,

    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl<'a> Data<'a> {
    pub fn name(&self) -> Option<Name> {
        match self.header.version_number {
            0..=1 => {
                if let Some(last_name) = self.get_field_owned("DAB") {
                    let first_name = self.get_field_owned("DAC")?;
                    let middle_name = self.get_field_owned("DAD");
                    let suffix = self.get_field_owned("DAE");
                    let prefix = self.get_field_owned("DAF");

                    Some(Name {
                        first_name,
                        middle_name,
                        last_name,
                        suffix,
                        prefix,
                    })
                } else {
                    let mut name = self.get_field("DAA")?.split(",");
                    let last_name = name.next()?.to_string();
                    let first_name = name.next()?.to_string();
                    let middle_name = name.next().map(str::to_string);

                    Some(Name {
                        first_name,
                        middle_name,
                        last_name: last_name.to_string(),
                        suffix: None,
                        prefix: None,
                    })
                }
            }
            2..=3 => {
                let names = self.get_field("DCT")?;

                let mut parts = if names.contains(',') {
                    names.split(',')
                } else {
                    names.split(' ')
                };

                let first_name = parts.next()?.to_string();
                let middle_name = parts.next().map(str::to_string);

                Some(Name {
                    first_name,
                    middle_name,
                    last_name: self.get_field_owned("DCS")?,
                    suffix: None,
                    prefix: None,
                })
            }
            4.. => Some(Name {
                first_name: self.get_field_owned("DAC")?,
                middle_name: self.get_field_owned("DAD"),
                last_name: self.get_field_owned("DCS")?,
                suffix: self.get_field_owned("DCU"),
                prefix: None,
            }),
        }
    }

    pub fn date_of_birth(&self) -> Option<time::Date> {
        let country = IssuerIdentification::try_from(self.header.issuer_id)
            .map(|issuer| issuer.country())
            .unwrap_or(IssuerCountry::UnitedStates);

        let date_of_birth = self.get_field("DBB")?;

        self.parse_date(date_of_birth, country)
    }

    fn parse_date(&self, input: &str, country: IssuerCountry) -> Option<time::Date> {
        if input.len() != 8 {
            tracing::warn!("date was incorrect length: {input}");
            return None;
        }

        let input = &input[..8];

        static MDY: &[time::format_description::FormatItem<'_>] =
            format_description!("[month][day][year]");
        static YMD: &[time::format_description::FormatItem<'_>] =
            format_description!("[year][month][day]");

        if country == IssuerCountry::UnitedStates && self.header.version_number != 1 {
            match time::Date::parse(input, &MDY) {
                Err(_err) => time::Date::parse(input, &YMD),
                date => date,
            }
        } else {
            time::Date::parse(input, &YMD)
        }
        .tap_err(|err| tracing::warn!("could not parse date {input} ({country:?}): {err}"))
        .ok()
    }

    /// Attempt to get a field from known subfile types.
    fn get_field(&self, name: &str) -> Option<&'a str> {
        [SubfileType::DL, SubfileType::EN, SubfileType::ID]
            .into_iter()
            .find_map(|subfile_type| {
                self.subfiles
                    .get(&subfile_type)
                    .and_then(|subfile| subfile.get(name).cloned().flatten())
            })
    }

    fn get_field_owned(&self, name: &str) -> Option<String> {
        self.get_field(name).map(str::to_string)
    }
}
