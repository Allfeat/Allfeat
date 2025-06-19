// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use frame_support::sp_runtime::RuntimeDebug;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Representation of a date for use in MIDDS fields.
///
/// This struct contains the year, month, and day in numerical format.
/// It is meant for simple, unambiguous date representation without timezone or time information.
#[derive(
    Clone,
    RuntimeDebug,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// Enum representing the language in which MIDDS metadata is written.
///
/// This is used to identify the language context of the metadata fields.
/// Defaults to English.
#[repr(u8)]
#[derive(
    Clone,
    RuntimeDebug,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum Language {
    English = 0,
    French = 1,
    Spanish = 2,
    German = 3,
    Italian = 4,
    Portuguese = 5,
    Russian = 6,
    Chinese = 7,
    Japanese = 8,
    Korean = 9,
    Arabic = 10,
    Hindi = 11,
    Dutch = 12,
    Swedish = 13,
    Norwegian = 14,
    Finnish = 15,
    Polish = 16,
    Turkish = 17,
    Hebrew = 18,
    Greek = 19,
    Latin = 20,
    Esperanto = 21,
}

/// Enum representing the ISO 3166-1 alpha-2 country codes.
///
/// This enum includes all officially recognized countries and territories.
/// Each variant corresponds to a two-letter country code.
#[derive(
    RuntimeDebug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u16)]
pub enum Country {
    /// Andorra
    AD,
    /// United Arab Emirates
    AE,
    /// Afghanistan
    AF,
    /// Antigua and Barbuda
    AG,
    /// Anguilla
    AI,
    /// Albania
    AL,
    /// Armenia
    AM,
    /// Angola
    AO,
    /// Antarctica
    AQ,
    /// Argentina
    AR,
    /// American Samoa
    AS,
    /// Austria
    AT,
    /// Australia
    AU,
    /// Aruba
    AW,
    /// Åland Islands
    AX,
    /// Azerbaijan
    AZ,
    /// Bosnia and Herzegovina
    BA,
    /// Barbados
    BB,
    /// Bangladesh
    BD,
    /// Belgium
    BE,
    /// Burkina Faso
    BF,
    /// Bulgaria
    BG,
    /// Bahrain
    BH,
    /// Burundi
    BI,
    /// Benin
    BJ,
    /// Saint Barthélemy
    BL,
    /// Bermuda
    BM,
    /// Brunei Darussalam
    BN,
    /// Bolivia, Plurinational State of
    BO,
    /// Bonaire, Sint Eustatius and Saba
    BQ,
    /// Brazil
    BR,
    /// Bahamas
    BS,
    /// Bhutan
    BT,
    /// Bouvet Island
    BV,
    /// Botswana
    BW,
    /// Belarus
    BY,
    /// Belize
    BZ,
    /// Canada
    CA,
    /// Cocos (Keeling) Islands
    CC,
    /// Congo, The Democratic Republic of the
    CD,
    /// Central African Republic
    CF,
    /// Congo
    CG,
    /// Switzerland
    CH,
    /// Côte d'Ivoire
    CI,
    /// Cook Islands
    CK,
    /// Chile
    CL,
    /// Cameroon
    CM,
    /// China
    CN,
    /// Colombia
    CO,
    /// Costa Rica
    CR,
    /// Cuba
    CU,
    /// Cabo Verde
    CV,
    /// Curaçao
    CW,
    /// Christmas Island
    CX,
    /// Cyprus
    CY,
    /// Czechia
    CZ,
    /// Germany
    DE,
    /// Djibouti
    DJ,
    /// Denmark
    DK,
    /// Dominica
    DM,
    /// Dominican Republic
    DO,
    /// Algeria
    DZ,
    /// Ecuador
    EC,
    /// Estonia
    EE,
    /// Egypt
    EG,
    /// Western Sahara
    EH,
    /// Eritrea
    ER,
    /// Spain
    ES,
    /// Ethiopia
    ET,
    /// Finland
    FI,
    /// Fiji
    FJ,
    /// Falkland Islands (Malvinas)
    FK,
    /// Micronesia, Federated States of
    FM,
    /// Faroe Islands
    FO,
    /// France
    FR,
    /// Gabon
    GA,
    /// United Kingdom
    GB,
    /// Grenada
    GD,
    /// Georgia
    GE,
    /// French Guiana
    GF,
    /// Guernsey
    GG,
    /// Ghana
    GH,
    /// Gibraltar
    GI,
    /// Greenland
    GL,
    /// Gambia
    GM,
    /// Guinea
    GN,
    /// Guadeloupe
    GP,
    /// Equatorial Guinea
    GQ,
    /// Greece
    GR,
    /// South Georgia and the South Sandwich Islands
    GS,
    /// Guatemala
    GT,
    /// Guam
    GU,
    /// Guinea-Bissau
    GW,
    /// Guyana
    GY,
    /// Hong Kong
    HK,
    /// Heard Island and McDonald Islands
    HM,
    /// Honduras
    HN,
    /// Croatia
    HR,
    /// Haiti
    HT,
    /// Hungary
    HU,
    /// Indonesia
    ID,
    /// Ireland
    IE,
    /// Israel
    IL,
    /// Isle of Man
    IM,
    /// India
    IN,
    /// British Indian Ocean Territory
    IO,
    /// Iraq
    IQ,
    /// Iran, Islamic Republic of
    IR,
    /// Iceland
    IS,
    /// Italy
    IT,
    /// Jersey
    JE,
    /// Jamaica
    JM,
    /// Jordan
    JO,
    /// Japan
    JP,
    /// Kenya
    KE,
    /// Kyrgyzstan
    KG,
    /// Cambodia
    KH,
    /// Kiribati
    KI,
    /// Comoros
    KM,
    /// Saint Kitts and Nevis
    KN,
    /// Korea, Democratic People's Republic of
    KP,
    /// Korea, Republic of
    KR,
    /// Kuwait
    KW,
    /// Cayman Islands
    KY,
    /// Kazakhstan
    KZ,
    /// Lao People's Democratic Republic
    LA,
    /// Lebanon
    LB,
    /// Saint Lucia
    LC,
    /// Liechtenstein
    LI,
    /// Sri Lanka
    LK,
    /// Liberia
    LR,
    /// Lesotho
    LS,
    /// Lithuania
    LT,
    /// Luxembourg
    LU,
    /// Latvia
    LV,
    /// Libya
    LY,
    /// Morocco
    MA,
    /// Monaco
    MC,
    /// Moldova, Republic of
    MD,
    /// Montenegro
    ME,
    /// Saint Martin (French part)
    MF,
    /// Madagascar
    MG,
    /// Marshall Islands
    MH,
    /// North Macedonia
    MK,
    /// Mali
    ML,
    /// Myanmar
    MM,
    /// Mongolia
    MN,
    /// Macao
    MO,
    /// Northern Mariana Islands
    MP,
    /// Martinique
    MQ,
    /// Mauritania
    MR,
    /// Montserrat
    MS,
    /// Malta
    MT,
    /// Mauritius
    MU,
    /// Maldives
    MV,
    /// Malawi
    MW,
    /// Mexico
    MX,
    /// Malaysia
    MY,
    /// Mozambique
    MZ,
    /// Namibia
    NA,
    /// New Caledonia
    NC,
    /// Niger
    NE,
    /// Norfolk Island
    NF,
    /// Nigeria
    NG,
    /// Nicaragua
    NI,
    /// Netherlands
    NL,
    /// Norway
    NO,
    /// Nepal
    NP,
    /// Nauru
    NR,
    /// Niue
    NU,
    /// New Zealand
    NZ,
    /// Oman
    OM,
    /// Panama
    PA,
    /// Peru
    PE,
    /// French Polynesia
    PF,
    /// Papua New Guinea
    PG,
    /// Philippines
    PH,
    /// Pakistan
    PK,
    /// Poland
    PL,
    /// Saint Pierre and Miquelon
    PM,
    /// Pitcairn
    PN,
    /// Puerto Rico
    PR,
    /// Palestine, State of
    PS,
    /// Portugal
    PT,
    /// Palau
    PW,
    /// Paraguay
    PY,
    /// Qatar
    QA,
    /// Réunion
    RE,
    /// Romania
    RO,
    /// Serbia
    RS,
    /// Russian Federation
    RU,
    /// Rwanda
    RW,
    /// Saudi Arabia
    SA,
    /// Solomon Islands
    SB,
    /// Seychelles
    SC,
    /// Sudan
    SD,
    /// Sweden
    SE,
    /// Singapore
    SG,
    /// Saint Helena, Ascension and Tristan da Cunha
    SH,
    /// Slovenia
    SI,
    /// Svalbard and Jan Mayen
    SJ,
    /// Slovakia
    SK,
    /// Sierra Leone
    SL,
    /// San Marino
    SM,
    /// Senegal
    SN,
    /// Somalia
    SO,
    /// Suriname
    SR,
    /// South Sudan
    SS,
    /// Sao Tome and Principe
    ST,
    /// El Salvador
    SV,
    /// Sint Maarten (Dutch part)
    SX,
    /// Syrian Arab Republic
    SY,
    /// Eswatini
    SZ,
    /// Turks and Caicos Islands
    TC,
    /// Chad
    TD,
    /// French Southern Territories
    TF,
    /// Togo
    TG,
    /// Thailand
    TH,
    /// Tajikistan
    TJ,
    /// Tokelau
    TK,
    /// Timor-Leste
    TL,
    /// Turkmenistan
    TM,
    /// Tunisia
    TN,
    /// Tonga
    TO,
    /// Turkey
    TR,
    /// Trinidad and Tobago
    TT,
    /// Tuvalu
    TV,
    /// Taiwan, Province of China
    TW,
    /// Tanzania, United Republic of
    TZ,
    /// Ukraine
    UA,
    /// Uganda
    UG,
    /// United States Minor Outlying Islands
    UM,
    /// United States
    US,
    /// Uruguay
    UY,
    /// Uzbekistan
    UZ,
    /// Holy See (Vatican City State)
    VA,
    /// Saint Vincent and the Grenadines
    VC,
    /// Venezuela, Bolivarian Republic of
    VE,
    /// Virgin Islands, British
    VG,
    /// Virgin Islands, U.S.
    VI,
    /// Viet Nam
    VN,
    /// Vanuatu
    VU,
    /// Wallis and Futuna
    WF,
    /// Samoa
    WS,
    /// Yemen
    YE,
    /// Mayotte
    YT,
    /// South Africa
    ZA,
    /// Zambia
    ZM,
    /// Zimbabwe
    ZW,
}

/// Enum representing all major and minor keys, including sharps, flats,
/// and their enharmonic equivalents.
///
/// This can be used to specify the musical key of a track or composition
/// with precise notation.
///
/// Notes:
/// - `m` suffix indicates minor.
/// - `s` indicates sharp, `b` indicates flat.
/// - Enharmonic equivalents are preserved for clarity and exact notation.
#[repr(u8)]
#[derive(
    RuntimeDebug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum Key {
    A = 0,
    Am = 1,
    As = 2,  // A#
    Asm = 3, // A#m
    Ab = 4,
    Abm = 5,
    B = 6,
    Bm = 7,
    Bs = 8,  // B#
    Bsm = 9, // B#m
    Bb = 10,
    Bbm = 11,
    C = 12,
    Cm = 13,
    Cs = 14,  // C#
    Csm = 15, // C#m
    Cb = 16,
    Cbm = 17,
    D = 18,
    Dm = 19,
    Ds = 20,  // D#
    Dsm = 21, // D#m
    Db = 22,
    Dbm = 23,
    E = 24,
    Em = 25,
    Es = 26,  // E#
    Esm = 27, // E#m
    Eb = 28,
    Ebm = 29,
    F = 30,
    Fm = 31,
    Fs = 32,  // F#
    Fsm = 33, // F#m
    Fb = 34,
    Fbm = 35,
    G = 36,
    Gm = 37,
    Gs = 38,  // G#
    Gsm = 39, // G#m
    Gb = 40,
    Gbm = 41,
}
