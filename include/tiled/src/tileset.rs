use super::helper::*;
use super::error::TiledError;
use super::Tileset;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesStart;
use quick_xml::reader::Reader;

impl Tileset {
    fn blank() -> Tileset {
        Tileset {
            first_tile_id : 0,
            name : String::from(""),
            tile_width : 0,
            tile_height : 0,
            tile_count : 0,
            column_count : 0,
            margin : 0,
            spacing : 0,
            image_path : String::from(""),
            image_width : 0,
            image_height : 0,
            version : String::new(),
            tiledversion : String::new(),
            }
    }

    fn parse_tileset_attribs(&mut self, attribs : Vec<Attribute>) -> Result<(), TiledError> {
        for a in attribs {
            match a.key.as_ref() {
                b"name" => self.name = get_value(&a.value)?,
                b"tilewidth" => self.tile_width = get_value(&a.value)?,
                b"tileheight" => self.tile_height = get_value(&a.value)?,
                b"spacing" => self.spacing = get_value(&a.value)?,
                b"margin" => self.margin = get_value(&a.value)?,
                b"tilecount" => self.tile_count = get_value(&a.value)?,
                b"columns" => self.column_count = get_value(&a.value)?,
                b"version" => self.version = get_string(&a.value)?.to_string(),
                b"tiledversion" => self.tiledversion = get_string(&a.value)?.to_string(),
                _ => println!("warning: unrecognized attribute {:?}", a.key),
            }
        }
        Ok(())
    }

    fn parse_image_attribs(&mut self, attribs : Vec<Attribute>) -> Result<(), TiledError> {
        for a in attribs {
            match a.key.as_ref() {
                b"source" => self.image_path.push_str(get_string(&a.value)?),
                b"width" => self.image_width = get_value(&a.value)?,
                b"height" => self.image_height = get_value(&a.value)?,
                _ => println!("warning: unrecognized attribute {:?}", a.key),
            }
        }
        Ok(())
    }
    
    fn parse_xml(tileset : &mut Self, tsx_text : String) -> Result<(), TiledError>{
        let mut reader = Reader::from_str(&tsx_text);
        parse_xml(tileset, &mut reader)
    }
    
    pub fn new(attribs : Vec<Attribute>, path : String) -> Result<Tileset, TiledError> {
        let mut tmx_path = path.clone();
        let mut tileset = Self::blank();
        tileset.image_path = path;
        for a in attribs {
            match a.key.as_ref() {
                b"firstgid" => tileset.first_tile_id = get_value(&a.value)?,
                b"source" => {
                    Self::parse_xml(
                        &mut tileset,
                        read_file_to_string( {
                            tmx_path.push_str(get_string(&a.value)?);
                            &tmx_path
                        })?
                    )?;
                }
                _  => println!("warning: unrecognized atrribute {:?}", a.key),
            }
        }
        
        Ok(tileset)
    }
}

impl HandleXml for Tileset {
    fn start(&mut self, e : &BytesStart, _: &mut Reader<&[u8]>) -> Result<(), TiledError> {
        match e.name().as_ref() {
            b"tileset" => self.parse_tileset_attribs(collect_attribs(&e)?)?,
            _ => println!("unrecognized tag {:?}", e.name()),
        }
        Ok(())
    }
    fn empty(&mut self, e : &BytesStart) -> Result<(), TiledError> {
        match e.name().as_ref() {
            b"image" => self.parse_image_attribs(collect_attribs(&e)?)?,
            _ => println!("unrecognized empty tag {:?}", e.name()),
        }
        Ok(())
    }
    fn self_tag() -> &'static str {
        "tileset"
    }
}
