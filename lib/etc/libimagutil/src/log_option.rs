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

/// This macro is used to generate extensions for the `Option<T>` type which only have
/// sideeffects.
///
/// This macro is then used to generate debug/info/log/warning/etc extensions.
///
/// It is exported, so other crates can use it to generate more specific extensions for
/// `Option<T>` types
///
/// # Parameters
///
/// The documentation for the parameters of the macro follow.
///
/// ## `$name`
///
/// name of the trait to generate
///
/// ## `$map_name`
///
/// Name of the function which is generated to call the closure with.
///
/// This function gets `&T` from `Option<T>` and can now build the argument for
/// `$closure`. So, this function can, for example, `|e| format!("Look here: {:?}", e)`, the
/// Option gets fed to `$closure`.
///
/// ## `$map_str_name`
///
/// Name of the function which is generated to call the closure with.
///
/// This function gets simply a `&str` which gets fed to the `$closure` later.
/// So it can be used to `foo().$map_str_name("Something happened")`
///
/// ## `$closure`
///
/// The closure which should be called when mapping.
///
/// This closure can now do things, but the return value of the closure is discarded.
/// So, this closure can be used for its sideeffects (logging for example) only.
///
/// An example would be: `|element| debug!("Element: {:?}", element)`.
///
#[macro_export]
macro_rules! generate_option_logging_extension {
    {
        $name: ident,
        $map_name: ident,
        $map_str_name: ident,
        $closure: expr
    } => {
        pub trait $name<T> : Sized {

            fn $map_name<F: FnOnce(&T) -> String>(self, f: F) -> Self;

            fn $map_str_name(self, s: &str) -> Self {
                self.$map_name(|_| format!("{}", s))
            }

        }

        impl<T> $name<T> for Option<T> {

            fn $map_name<F: FnOnce(&T) -> String>(self, f: F) -> Self {
                self.map(|x| { $closure(f(&x)); x })
            }

        }

    }
}

