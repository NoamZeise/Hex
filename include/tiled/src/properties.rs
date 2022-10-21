use std::collections::HashMap;

use super::Properties;
use super::helper::*;
use super::TiledError;

use quick_xml::events::BytesStart;
use quick_xml::events::attributes::Attribute;

enum PropertyType {
    Bool,
    Int,
}

impl Properties {
    pub fn blank() -> Properties {
        Properties { booleans: HashMap::new(), integers: HashMap::new(), }
    }
    
    fn add_property(&mut self, attribs : Vec<Attribute>) -> Result<(), TiledError>{
        let mut name = String::new();
        let mut prop_type : Option<PropertyType> = None;
        for a in attribs {
             match a.key.as_ref() {
                 b"name" => name = get_string(&a.value)?.to_string(),
                 b"type" => prop_type = match get_string(&a.value)? {
                     "bool" => Some(PropertyType::Bool),
                     "int" => Some(PropertyType::Int),
                     _ =>  {
                         println!("warning: unrecognized type {:?}", get_string(&a.value)?);
                         None
                     }, 
                 },
                 b"value" => match prop_type {
                     Some(t) => {
                         match t {
                             PropertyType::Bool => {
                                 self.booleans
                                     .insert(name,
                                             match get_string(&a.value)? {
                                                 "true" => true,
                                                 "false" => false,
                                                 _ => { return Err(TiledError::ParseError(
                                                     String::from(
                                                         "bool didnt have true or false value"
                                                     )));
                                                 },
                                             }
                                     );
                             },
                             PropertyType::Int => { self.integers.insert(name, get_value(&a.value)?); },
                         };
                         break;
                     },
                     None => { return Err(TiledError::UnsupportedType()); },
                 }
                 _ => println!("warning: unrecognized atrribute {:?}", a.key),
             }
        }
        Ok(())
    }
}

impl HandleXml for Properties {
    fn empty(&mut self, e : &BytesStart) -> Result<(), TiledError> {
        match e.name().as_ref() {
            b"property" => self.add_property(collect_attribs(&e)?)?,
            _ => println!("unrecognized empty tag {:?}", e.name()),
        }
        Ok(())
    }
    fn self_tag() -> &'static str {
        "properties"
    }
}
