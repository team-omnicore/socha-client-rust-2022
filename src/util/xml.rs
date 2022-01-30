use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::fmt;
use std::str;
use std::io::{Write, Cursor, BufRead};
use log::{warn, error};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{Event, BytesStart, BytesText, BytesEnd};
use quick_xml::{Reader, Writer};
use super::{SCResult, SCError};

/// A deserialized, in-memory tree-representation
/// of an XML node.
#[derive(Debug, Default)]
pub struct Element {
    name: String,
    content: String,
    attributes: HashMap<String, String>,
    childs: Vec<Element>
}

/// A builder that makes the construction of new
/// XML nodes more convenient.
pub struct ElementBuilder<'a> {
    name: &'a str,
    content: &'a str,
    attributes: HashMap<String, String>,
    childs: Vec<Element>
}

impl Element {
    /// Creates a new XML element builder.
    pub fn new(name: &str) -> ElementBuilder {
        ElementBuilder::new(name)
    }

    /// Deserializes an XML node tree
    /// from the given XML event reader.
    pub fn read_from<R>(reader: &mut Reader<R>) -> SCResult<Element> where R: BufRead {
        let mut node_stack = VecDeque::<Element>::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref start)) => {
                    let node = Element::try_from(start)?;
                    node_stack.push_back(node);
                },
                Ok(Event::End(ref e)) => {
                    if let Some(node) = node_stack.pop_back() {
                        if let Some(mut parent) = node_stack.pop_back() {
                            parent.childs.push(node);
                            node_stack.push_back(parent);
                        } else {
                            return Ok(node);
                        }
                    } else {
                        error!("Found closing element </{}> without an opening element before", str::from_utf8(e.name())?);
                    }
                },
                Ok(Event::Text(ref t)) => {
                    let content = str::from_utf8(t)?;
                    if let Some(node) = node_stack.back_mut() {
                        node.content += content;
                    } else {
                        warn!("Found characters {} outside of any node", content);
                    }
                },
                Err(e) => return Err(e.into()),
                _ => ()
            }
        }
    }
    
    /// Serializes the node to an XML string using a tree traversal.
    pub fn write_to<W>(&self, writer: &mut Writer<W>) -> SCResult<()> where W: Write {
        let start = BytesStart::from(self);
        
        if self.childs.is_empty() {
            // Write self-closing tag, e.g. <Element/>
            writer.write_event(Event::Empty(start))?;
        } else {
            // Write opening tag, e.g. <Element>
            writer.write_event(Event::Start(start))?;
            
            // Write text
            if !self.content.is_empty() {
                writer.write_event(Event::Text(BytesText::from_plain(self.content.as_bytes())))?;
            }

            // Write child elements
            for child in &self.childs {
                child.write_to(writer)?;
            }
            
            // Write closing tag, e.g. </Element>
            writer.write_event(Event::End(BytesEnd::borrowed(self.name.as_bytes())))?;
        }

        Ok(())
    }
    
    /// Fetches the node's tag name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    
    /// Fetches the node's textual contents.
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
    
    /// Fetches an attribute's value by key.
    pub fn attribute(&self, key: &str) -> SCResult<&str> {
        self.attributes.get(key).map(|s| s.as_str()).ok_or_else(|| format!("No attribute with key '{}' found in <{}>!", key, self.name).into())
    }
    
    /// Finds the first child element with the provided tag name.
    pub fn child_by_name<'a, 'n: 'a>(&'a self, name: &'n str) -> SCResult<&'a Element> {
        self.childs_by_name(name).next().ok_or_else(|| format!("No <{}> found in <{}>!", name, self.name).into())
    }
    
    /// Fetches a list of all child elements matching the provided tag name.
    pub fn childs_by_name<'a, 'n: 'a>(&'a self, name: &'n str) -> impl Iterator<Item=&'a Element> + 'a {
        self.childs.iter().filter(move |c| c.name == name)
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Writes the node as XML
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        self.write_to(&mut writer).map_err(|_| fmt::Error)?;
        write!(f, "{}", str::from_utf8(&writer.into_inner().into_inner()).map_err(|_| fmt::Error)?)
    }
}

impl<'a> ElementBuilder<'a> {
    /// Creates a new XML node builder with the
    /// specified tag name.
    pub fn new(name: &'a str) -> Self {
        Self { name: name, content: "", attributes: HashMap::new(), childs: Vec::new() }
    }
    
    /// Sets the tag name of the XML node.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    
    /// Sets the contents of the XML node.
    pub fn content(mut self, data: &'a str) -> Self {
        self.content = data;
        self
    }
    
    /// Adds the specified attributes.
    pub fn attributes(mut self, attributes: impl IntoIterator<Item=(String, String)>) -> Self {
        self.attributes.extend(attributes);
        self
    }
    
    /// Adds the specified attribute.
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
    
    /// Adds the specified children.
    pub fn childs(mut self, childs: impl IntoIterator<Item=Element>) -> Self {
        self.childs.extend(childs);
        self
    }
    
    /// Adds the specified child.
    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.childs.push(child.into());
        self
    }
    
    /// Tries adding the specified child.
    pub fn try_child(mut self, child: impl TryInto<Element, Error=SCError>) -> SCResult<Self> {
        self.childs.push(child.try_into()?);
        Ok(self)
    }
    
    /// Builds the XML node.
    pub fn build(self) -> Element {
        Element {
            name: self.name.to_owned(),
            content: self.content.to_owned(),
            attributes: self.attributes,
            childs: self.childs
        }
    }
}

impl<'a> Default for ElementBuilder<'a> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> From<ElementBuilder<'a>> for Element {
    fn from(builder: ElementBuilder<'a>) -> Self { builder.build() }
}

impl<'a> TryFrom<&BytesStart<'a>> for Element {
    type Error = SCError;

    fn try_from(start: &BytesStart<'a>) -> SCResult<Self> {
        Ok(Element {
            name: str::from_utf8(start.name())?.to_owned(),
            content: String::new(),
            attributes: start.attributes()
                .into_iter()
                .map(|res| {
                    let attribute = res?;
                    let key = str::from_utf8(attribute.key)?.to_owned();
                    let value = str::from_utf8(&attribute.value)?.to_owned();
                    Ok((key, value))
                })
                .collect::<SCResult<HashMap<_, _>>>()?,
            childs: Vec::new()
        })
    }
}

impl<'a> From<&'a Element> for BytesStart<'a> {
    fn from(element: &'a Element) -> Self {
        BytesStart::borrowed_name(element.name.as_bytes())
            .with_attributes(element.attributes.iter().map(|(k, v)| Attribute {
                key: k.as_bytes(),
                value: Cow::Borrowed(v.as_bytes()),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::Element;

    #[test]
    fn test_write() {
        assert_eq!("<Test/>", format!("{}", Element::new("Test").build()));
        assert_eq!("<A><B/><C/></A>", format!("{}", Element::new("A").child(Element::new("B")).child(Element::new("C")).build()))
    }
}