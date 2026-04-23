use std::{any::Any, error::Error as StdError, sync::{Arc, Weak}};
use derive_more::{Error, Display, From};
use crate::{net::Client, resources::resource_handler::ResourceManager};

use super::error::Error;

#[derive(Debug, Clone)]
pub struct FormatString {
    parts: Vec<FormattableStringPart>,
    // an alive FormatString shouldn't keep a ResourceManager from a Client alive,
    // because when the Client dies so should its ResourceManager
    resource_manager: Weak<ResourceManager>
}

impl FormatString {
    pub fn new(client: &Client, raw: impl AsRef<str>) -> Self {
        Self::from_parts(client, Self::make_parts(raw))
    }

    pub fn new_res_manager(res_manager: &Arc<ResourceManager>, raw: impl AsRef<str>) -> Self {
        Self {
            parts: Self::make_parts(raw),
            resource_manager: Arc::downgrade(res_manager)
        }
    }

    fn make_parts(raw: impl AsRef<str>) -> Vec<FormattableStringPart> {
        let raw = raw.as_ref();
        let mut chars = raw.chars().peekable();
        let mut raw_sting_buffer = String::new();
        let mut parts: Vec<FormattableStringPart> = Vec::new();

        'outer: while let Some(c) = chars.next() {
            match c {
                '<' => {
                    if let Some(&'<') = chars.peek() {
                        let _ = chars.next();
                        raw_sting_buffer.push('<');
                        continue 'outer;
                    } else {
                        parts.push(FormattableStringPart::Raw(raw_sting_buffer));
                        raw_sting_buffer = String::new();

                        let mut part = String::new();

                        'inner: while let Some(c_part) = chars.next() {
                            match c_part {
                                '>' => {
                                    if let Some(&'>') = chars.peek() {
                                        let _ = chars.next();
                                        part.push('>');
                                        continue 'inner;
                                    } else {
                                        break 'inner;
                                    }
                                }

                                '<' => {
                                    if let Some(&'<') = chars.peek() {
                                        let _ = chars.next();
                                        part.push('<');
                                        continue 'inner;
                                    } else {
                                        panic!(
                                            "invalid formattable string in FormatString: lone \'<\' inside formattable section (did you mean \'<<\'?)"
                                        )
                                    }
                                }

                                a => part.push(a),
                            }
                        }

                        parts.push(FormattableStringPart::ResourceReplace(part));
                    }
                }

                '>' => {
                    if let Some(&'>') = chars.peek() {
                        let _ = chars.next();
                        raw_sting_buffer.push('>')
                    } else {
                        panic!(
                            "invalid formattable string in FormatString: unpaired \'>\' inside raw section (did you mean \'>>\'?)"
                        )
                    }
                }

                c => raw_sting_buffer.push(c),
            }
        }

        if !raw_sting_buffer.is_empty() {
            parts.push(FormattableStringPart::Raw(raw_sting_buffer));
        }

        parts
    }

    pub fn from_parts(client: &Client, parts: Vec<FormattableStringPart>) -> Self {
        Self {
            parts,
            resource_manager: Arc::downgrade(&client.resource_manager)
        }
    }

    #[allow(clippy::manual_async_fn)]
    pub fn to_formatted_now(&self) -> impl Future<Output = Result<String, FormatStringError>> + Send {
        async move {
            let mut result = String::new();

            for part in self.parts.iter() {
                match part {
                    FormattableStringPart::Raw(raw) => result.push_str(raw),
                    FormattableStringPart::ResourceReplace(resource_replace) => {
                        let upgrade_weak = self.resource_manager.upgrade().ok_or(FormatStringError::ClientGotDropped)?;
                        let binding = upgrade_weak
                            .get(resource_replace.as_str())
                            .ok_or(FormatStringError::NoResFound(resource_replace.clone()))?;

                        let data = binding.data();

                        let data = data.await.map_err(FormatStringError::ResourceError)?;

                        result.push_str(&data.to_string());
                    }
                }
            }

            print!("{result}");

            Ok(result)
        }
    }

    #[allow(unused)]
    pub(crate) fn inner_vec(&self) -> &Vec<FormattableStringPart> {
        &self.parts
    }

    #[allow(unused)]
    pub(crate) fn inner_vec_mut(&mut self) -> &mut Vec<FormattableStringPart> {
        &mut self.parts
    }
}

#[derive(Debug, Display, Error, From)]
#[display("FormatStringError: {_variant}")]
pub enum FormatStringError {
    #[from(skip)]
    #[error(ignore)]
    #[display("The resource with identifier `{_0}` couldn't be found.")]
    NoResFound(String),

    #[display("Resource error: {_0:?}")]
    #[error(ignore)]
    ResourceError(Arc<dyn Any + Send + Sync>),

    #[display("The client this FormattableString refers to got dropped.")]
    #[from(skip)]
    ClientGotDropped,
}

#[derive(Debug, Clone)]
pub enum FormattableStringPart {
    Raw(String),
    ResourceReplace(String),
}
