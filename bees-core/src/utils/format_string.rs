use crate::resource;


#[derive(Debug, Clone)]
pub struct FormatString {
    parts: Vec<FormattableStringPart>
}

impl FormatString {
    pub fn new(raw: impl AsRef<str>) -> Self {
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
                                        panic!("invalid formattable string in FormatString: lone \'<\' inside formattable section (did you mean \'<<\'?)")
                                    }
                                }

                                a => part.push(a),
                            }
                        }

                        if part.starts_with('?') {
                            let part = part.chars().skip(1).collect::<String>();
                            parts.push(FormattableStringPart::ResourceReplace(part));
                        } else {
                            parts.push(FormattableStringPart::HashMapReplace(part));
                        }
                    }
                },

                '>' => {
                    if let Some(&'>') = chars.peek() {
                        let _ = chars.next();
                        raw_sting_buffer.push('>')
                    } else {
                        panic!("invalid formattable string in FormatString: unpaired \'>\' inside raw section (did you mean \'>>\'?)")
                    }
                }

                c => raw_sting_buffer.push(c),
            }
        }

        if !raw_sting_buffer.is_empty() {
            parts.push(FormattableStringPart::Raw(raw_sting_buffer));
        }

        Self {
            parts
        }
    }

    pub async fn to_formatted_now(&self, values: impl std::borrow::Borrow<std::collections::HashMap<String, String>>) -> Result<String, &'static str> {
        let values = values.borrow();
        let mut result = String::new();

        for part in self.parts.iter() {
            match part {
                FormattableStringPart::Raw(raw) => result.push_str(raw),
                FormattableStringPart::HashMapReplace(replace_name) => result.push_str(values.get(replace_name).ok_or("hashmap replace field not specified in values hashmap.")?),
                FormattableStringPart::ResourceReplace(resource_replace) => {
                    let resource = resource!(option resource_replace);
                    // println!("{:#?}", resource);
                    // println!("{}", resource_replace);
                    let resource = resource.ok_or("No resource with that name registered. Did you spell it right?")?;

                    let data = resource.data().await;
                    result.push_str(&data.to_string());
                },
            }
        }
        
        print!("{result}");

        Ok(result)
    }

    #[allow(dead_code)]
    pub(crate) fn inner_vec(&self) -> &Vec<FormattableStringPart> {
        &self.parts
    }

    pub(crate) fn inner_vec_mut(&mut self) -> &mut Vec<FormattableStringPart> {
        &mut self.parts
    }
}

impl<S: Into<String>> From<S> for FormatString {
    fn from(value: S) -> Self {
        let string = value.into();
        Self::new(string)
    }
}

#[derive(Debug, Clone)]
pub enum FormattableStringPart {
    Raw(String),
    HashMapReplace(String),
    ResourceReplace(String),
}