//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2018 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use libimagrt::runtime::Runtime;
use libimagerror::errors::ErrorMsg as EM;

use toml::Value;
use toml_query::read::TomlValueReadExt;
use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;

pub fn get_diary_name(rt: &Runtime) -> Option<String> {
    use libimagdiary::config::get_default_diary_name;

    rt.cli()
        .value_of("diaryname")
        .map(String::from)
        .or_else(|| get_default_diary_name(rt))
}

pub enum Timed {
    Daily,
    Hourly,
    Minutely,
    Secondly,
}

/// Returns true if the diary should always create timed entries, which is whenever
///
/// ```toml
/// diary.diaries.<diary>.timed = true
/// ```
///
/// # Returns
///
/// * Ok(Some(Timed::Hourly)) if diary should create timed entries
/// * Ok(Some(Timed::Minutely)) if diary should not create timed entries
/// * Ok(None) if config is not available
/// * Err(e) if reading the toml failed, type error or something like this
///
pub fn get_diary_timed_config(rt: &Runtime, diary_name: &str) -> Result<Option<Timed>> {
    match rt.config() {
        None      => Ok(None),
        Some(cfg) => {
            let v = cfg
                .read(&format!("diary.diaries.{}.timed", diary_name))
                .context(EM::IO)
                .map_err(Error::from);

            match v {
                Ok(Some(&Value::String(ref s))) => parse_timed_string(s, diary_name).map(Some),

                Ok(Some(_)) => {
                    let s = format!("Type error at 'diary.diaryies.{}.timed': should be either 'd'/'daily', 'h'/'hourly', 'm'/'minutely' or 's'/'secondly'", diary_name);
                    Err(format_err!("{}", s))
                },

                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        }
    }
}

pub fn parse_timed_string(s: &str, diary_name: &str) -> Result<Timed> {
    if s == "d" || s == "daily" {
        Ok(Timed::Daily)
    } else if s == "h" || s == "hourly" {
        Ok(Timed::Hourly)
    } else if s == "m" || s == "minutely" {
        Ok(Timed::Minutely)
    } else if s == "s" || s == "secondly" {
        Ok(Timed::Secondly)
    } else {
        let s = format!("Cannot parse config: 'diary.diaries.{}.timed = {}'",
                        diary_name, s);
        Err(format_err!("{}", s))
    }
}
