//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use std::collections::BTreeMap;

use vobject::vcard::Vcard;

#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    pub address: String,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

impl From<::vobject::vcard::Email> for Email {
    fn from(voemail: ::vobject::vcard::Email) -> Self {
        let address    = voemail.raw().clone();
        let properties = voemail.params().clone();

        Email { address, properties }
    }
}

/// A type which can be build from a Vcard and be serialized.
#[derive(Serialize, Deserialize, Debug)]
pub struct DeserVcard {

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    adr          : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    anniversary  : Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    bday         : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    categories   : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    clientpidmap : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    fullname     : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    gender       : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    geo          : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    impp         : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    key          : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    lang         : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    logo         : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    member       : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    name         : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    nickname     : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    note         : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    org          : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    photo        : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    proid        : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    related      : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    rev          : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    role         : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    sound        : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tel          : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    title        : Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tz           : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    uid          : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    url          : Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    version      : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    email        : Vec<Email>,
}

impl From<Vcard> for DeserVcard {
    fn from(card: Vcard) -> DeserVcard {
        macro_rules! arystr {
            ($v:expr) => {
                $v.into_iter().map(|o| o.raw().clone()).collect()
            };
        };
        macro_rules! optstr {
            ($o:expr) => {
                $o.map(|o| o.raw().clone())
            };
        };

        DeserVcard {
            adr          : arystr!(card.adr()),
            anniversary  : optstr!(card.anniversary()),
            bday         : optstr!(card.bday()),
            categories   : arystr!(card.categories()),
            clientpidmap : optstr!(card.clientpidmap()),
            email        : card.email().into_iter().map(Email::from).collect::<Vec<Email>>(),
            fullname     : arystr!(card.fullname()),
            gender       : optstr!(card.gender()),
            geo          : arystr!(card.geo()),
            impp         : arystr!(card.impp()),
            key          : arystr!(card.key()),
            lang         : arystr!(card.lang()),
            logo         : arystr!(card.logo()),
            member       : arystr!(card.member()),
            name         : optstr!(card.name()),
            nickname     : arystr!(card.nickname()),
            note         : arystr!(card.note()),
            org          : arystr!(card.org()),
            photo        : arystr!(card.photo()),
            proid        : optstr!(card.proid()),
            related      : arystr!(card.related()),
            rev          : optstr!(card.rev()),
            role         : arystr!(card.role()),
            sound        : arystr!(card.sound()),
            tel          : arystr!(card.tel()),
            title        : arystr!(card.title()),
            tz           : arystr!(card.tz()),
            uid          : optstr!(card.uid()),
            url          : arystr!(card.url()),
            version      : optstr!(card.version()),
        }
    }
}

impl DeserVcard {

    pub fn adr(&self) -> &Vec<String> {
        &self.adr
    }

    pub fn anniversary(&self) -> Option<&String> {
        self.anniversary.as_ref()
    }

    pub fn bday(&self) -> Option<&String> {
        self.bday.as_ref()
    }

    pub fn categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn clientpidmap(&self) -> Option<&String> {
        self.clientpidmap.as_ref()
    }

    pub fn email(&self) -> &Vec<Email> {
        &self.email
    }

    pub fn fullname(&self) -> &Vec<String> {
        &self.fullname
    }

    pub fn gender(&self) -> Option<&String> {
        self.gender.as_ref()
    }

    pub fn geo(&self) -> &Vec<String> {
        &self.geo
    }

    pub fn impp(&self) -> &Vec<String> {
        &self.impp
    }

    pub fn key(&self) -> &Vec<String> {
        &self.key
    }

    pub fn lang(&self) -> &Vec<String> {
        &self.lang
    }

    pub fn logo(&self) -> &Vec<String> {
        &self.logo
    }

    pub fn member(&self) -> &Vec<String> {
        &self.member
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn nickname(&self) -> &Vec<String> {
        &self.nickname
    }

    pub fn note(&self) -> &Vec<String> {
        &self.note
    }

    pub fn org(&self) -> &Vec<String> {
        &self.org
    }

    pub fn photo(&self) -> &Vec<String> {
        &self.photo
    }

    pub fn proid(&self) -> Option<&String> {
        self.proid.as_ref()
    }

    pub fn related(&self) -> &Vec<String> {
        &self.related
    }

    pub fn rev(&self) -> Option<&String> {
        self.rev.as_ref()
    }

    pub fn role(&self) -> &Vec<String> {
        &self.role
    }

    pub fn sound(&self) -> &Vec<String> {
        &self.sound
    }

    pub fn tel(&self) -> &Vec<String> {
        &self.tel
    }

    pub fn title(&self) -> &Vec<String> {
        &self.title
    }

    pub fn tz(&self) -> &Vec<String> {
        &self.tz
    }

    pub fn uid(&self) -> Option<&String> {
        self.uid.as_ref()
    }

    pub fn url(&self) -> &Vec<String> {
        &self.url
    }

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }

}

