use std::collections::HashMap;
use geometry::{Rect, Vec2};

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesStart;
use quick_xml::reader::Reader;

mod tileset;
mod layer;
mod object_group;
mod properties;
mod image_layer;
mod helper;
use helper::*;
pub mod error;
use error::TiledError;

pub struct Properties {
    pub booleans : HashMap<String, bool>,
    pub integers : HashMap<String, i64>,
}

pub struct LayerData {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub colour : Colour,
    pub tint : Colour,
    pub index_draw_order: bool,
    pub parallax: Vec2,
    pub offset: Vec2,
} 

pub type LayerTiles = Vec<u32>;

pub struct Layer {
    pub props : Properties,
    pub tiles : LayerTiles,
    pub width : i32,
    pub height: i32,
    pub info: LayerData,
}

pub struct ObjData {
    pub id: u32,
    pub name: String,
    pub type_name: String,
    pub visible: bool,
}

pub struct Obj {
    pub props : Properties,
    pub rect : Rect,
    pub rotation: f64,
    pub info: ObjData,
    poly : Option<Box<Poly>>,
    text : Option<Box<Text>>,
    point: bool,
    ellipse: bool,
    pub template: Option<String>,
}

pub struct Poly {
    pub points : Vec<Vec2>,
    pub obj : Obj,
    pub closed : bool,
}

type Point = Obj;
type Ellipse = Obj;

#[derive(Eq, PartialEq)]
pub enum TextHorizontalAlign {
    Left,
    Center,
    Right,
    Justify
}

#[derive(Eq, PartialEq)]
pub enum TextVerticalAlign {
    Top,
    Center,
    Bottom
}

pub struct Text {
    pub obj: Obj,
    pub text: String,
    pub font_family: String,
    pub pixel_size: u32,
    pub wrap: bool,
    pub bold: bool,
    pub italic: bool,
    pub horizontal_align : TextHorizontalAlign,
    pub vertical_align : TextVerticalAlign,
    pub colour: Colour,
}

pub struct ObjGroup {
    pub props : Properties,
    pub objs  : Vec<Obj>,
    pub polys : Vec<Poly>,
    pub points: Vec<Point>,
    pub ellipse: Vec<Ellipse>,
    pub text: Vec<Text>,
    pub info: LayerData,
    path: String,
}

pub struct ImageLayer {
    pub image_path : String,
    pub width: u32,
    pub height: u32,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub info: LayerData,
    pub props: Properties,
}

pub struct Colour {
    pub r : u32,
    pub g : u32,
    pub b : u32,
    pub a : u32,
}

pub struct Tileset {
    pub first_tile_id : u32,
    pub name : String,
    pub tile_width : u32,
    pub tile_height : u32,
    pub tile_count : u32,
    pub column_count : u32,

    pub margin : u32,
    pub spacing : u32,

    pub image_path : String,
    pub image_width : u32,
    pub image_height : u32,

    pub version : String,
    pub tiledversion : String,
}

pub enum Orientation {
    Orthogonal,
    Isometric,
    IsometricStaggered,
    HexagonalStaggered,
}

pub enum RenderOrder {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

pub struct MapMetadata {
    pub version : String,
    pub tiled_version : String,
    pub render_order : RenderOrder,
    pub next_layer_id : u32,
    pub next_object_id : u32,
}

pub struct Map {
    pub width : u32,
    pub height : u32,
    pub tile_width : u32,
    pub tile_height : u32,
    pub total_tiles : u32,
    pub infinite : bool,
    pub orientation : Orientation,

    pub tilesets : Vec<Tileset>,
    pub layers : Vec<Layer>,
    pub obj_groups : Vec<ObjGroup>,
    pub img_layers : Vec<ImageLayer>,
    pub texts : Vec<Text>,

    pub path : String,
    pub metadata : MapMetadata,
}

impl Map {
    pub fn new(filename : &str) -> Result<Map, TiledError> {
        let path = match filename.rsplit_once('/') {
            Some((path, _)) => path,
            None => "",
        };
        let mut path = path.to_owned();
        path.push('/');
        Self::parse_xml(
            read_file_to_string(filename)?,
            &path
        )
    }

    fn blank_map(path: String) -> Map {
        Map {
            width : 0,
            height : 0,
            tile_width : 0,
            tile_height : 0,
            total_tiles : 1,
            infinite : false,
            orientation : Orientation::Orthogonal,

            tilesets : Vec::new(),
            layers : Vec::new(),
            obj_groups : Vec::new(),
            img_layers : Vec::new(),
            texts : Vec::new(),
            path,
            metadata : MapMetadata {
                version: "".to_string(),
                tiled_version: "".to_string(),
                render_order: RenderOrder::RightDown,
                next_layer_id: 0,
                next_object_id: 0,
            }
        }
    }

    fn parse_map_attribs(&mut self, attribs : Vec<Attribute>) -> Result<(), TiledError> {
        for a in attribs {
            match a.key.as_ref() {
                b"width" => self.width = get_value(&a.value)?,
                b"height" => self.height = get_value(&a.value)?,
                b"tilewidth" => self.tile_width = get_value(&a.value)?,
                b"tileheight" => self.tile_height = get_value(&a.value)?,
                b"infinite" => self.infinite = get_value::<u32>(&a.value)? == 1,
                b"orientation" => self.orientation =  match a.value.as_ref() {
                    b"orthogonal" => Orientation::Orthogonal,
                    b"isometric" => Orientation::Isometric,
                    b"staggard" => Orientation::IsometricStaggered,
                    b"hexagonal" => Orientation::HexagonalStaggered,
                    _ => panic!("unrecognized map orientation"),
                },
                b"version" => self.metadata.version = get_string(&a.value)?.to_string(),
                b"tiledversion" => self.metadata.tiled_version = get_string(&a.value)?.to_string(),
                b"nextlayerid" => self.metadata.next_layer_id = get_value(&a.value)?,
                b"nextobjectid" => self.metadata.next_object_id = get_value(&a.value)?,
                b"renderorder" => self.metadata.render_order = match a.value.as_ref() {
                    b"right-down" => RenderOrder::RightDown,
                    b"right-up" => RenderOrder::RightUp,
                    b"left-down" => RenderOrder::LeftDown,
                    b"left-up" => RenderOrder::LeftUp,
                    _ => { return Err(TiledError::UnsupportedType()); },
                },
                _ => println!("warning: unrecognized atrribute {:?}", a.key),
            }
        }
        Ok(())
    }

    fn parse_xml(map_file_text : String, path : &str) -> Result<Map, TiledError> {
        let mut reader = Reader::from_str(&map_file_text);
        let mut map = Self::blank_map(path.to_string());
        parse_xml(&mut map, &mut reader)?;
        Ok(map)
    }
}

impl HandleXml for Map {
    fn start(&mut self, e : &BytesStart, reader: &mut Reader<&[u8]>) -> Result<(), TiledError> {
        match e.name().as_ref() {
            b"map" => self.parse_map_attribs(collect_attribs(&e)?)?,
            b"layer" => self.layers.push(Layer::new(collect_attribs(&e)?, reader)?), //add layer properly
            b"objectgroup" => self.obj_groups.push(ObjGroup::new(collect_attribs(&e)?, reader, self.path.clone())?),
            b"imagelayer" => self.img_layers.push(ImageLayer::new(collect_attribs(&e)?, reader)?),
            _ => println!("unrecognized tag {:?}", e.name()),
        }
        Ok(())
    }
    fn empty(&mut self, e : &BytesStart) -> Result<(), TiledError> {
        match e.name().as_ref() {
            b"tileset" => {
                self.tilesets.push(
                    Tileset::new(collect_attribs(&e)?, self.path.clone())?
                );
                self.total_tiles += self.tilesets.last().unwrap().tile_count;
            },
            _ => println!("unrecognized empty tag {:?}", e.name()),
        }
        Ok(())
    }
    fn self_tag() -> &'static str {
        ""
    }
}

#[cfg(test)]
mod tiled_tests {
    use super::*;
    #[test]
    fn test_map() {
        let map = Map::new("test-resources/test.tmx").unwrap();
        assert!(map.width == 4);
        assert!(map.height == 4);
        assert!(map.tile_width == 10);
        assert!(map.tile_height == 10);
        assert!(!map.infinite);
        assert!(match map.orientation {
            Orientation::Orthogonal => true,
            _ => false,
        });

        assert!(map.tilesets.len() == 1);
        assert!(map.tilesets[0].first_tile_id == 1);
        assert!(map.tilesets[0].tile_width == 10);
        assert!(map.tilesets[0].tile_height == 10);
        assert!(map.tilesets[0].spacing == 2);
        assert!(map.tilesets[0].margin == 5);
        assert!(map.tilesets[0].tile_count == 4);
        assert!(map.tilesets[0].column_count == 2);
        assert!(map.tilesets[0].image_path == "test-resources/test-tileset.png");
        assert!(map.tilesets[0].image_width == 32);
        assert!(map.tilesets[0].image_height == 32);

        assert!(map.layers.len() == 2);
        assert!(map.layers[0].width == 4);
        assert!(map.layers[0].height == 4);
        assert!(
            map.layers[0].tiles == vec![
                4, 4, 0, 0,
                2, 2, 2, 0,
                2, 2, 2, 0,
                4, 4, 0, 0,
            ]
        );
        assert!(map.layers[1].width == 4);
        assert!(map.layers[1].height == 4);
        assert!(map.layers[1].props.booleans["collidable"] == true);
        assert!(
            map.layers[1].tiles == vec![
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
        assert!(map.layers[1].info.id == 1);
        assert!(map.layers[1].info.name == "Tile Layer 1");
        assert!(map.layers[1].info.opacity == 0.90);
        assert!(map.layers[1].info.tint.g == 0);

        assert!(map.obj_groups.len() == 2);
        assert!(map.obj_groups[0].props.booleans["obj_group"] == true);
        assert!(map.obj_groups[0].objs.len() == 2);
        assert!(map.obj_groups[0].polys.len() == 2);
        assert!(map.obj_groups[0].points.len() == 1);
        assert!(map.obj_groups[0].ellipse.len() == 1);
        assert!(map.obj_groups[0].ellipse[0].rotation == 10.0);
        assert!(map.obj_groups[0].points[0].info.name == "dd");
        assert!(map.obj_groups[0].objs[1].props.integers["num"] == 5);
        assert!(map.obj_groups[0].objs[1].props.booleans["test"] == true);
        assert!(map.obj_groups[0].objs[1].rotation == 343.734);
        assert!(map.obj_groups[0].objs[1].info.type_name == "asd");
        assert!(map.obj_groups[0].objs[0].info.name == "barry");
        assert!(map.obj_groups[0].objs[0].info.visible == false);
        assert!(map.obj_groups[0].objs[1].info.id == 2);
        assert!(map.obj_groups[0].objs[1].rect.x == 4.25998);
        assert!(map.obj_groups[0].objs[1].rect.y == 10.0772);
        assert!(map.obj_groups[0].objs[1].rect.w == 15.1719);
        assert!(map.obj_groups[0].objs[1].rect.h == 18.3066);
        assert!(map.obj_groups[0].objs[0].rect.x == 28.6511);
        assert!(map.obj_groups[0].objs[0].rect.y == 10.658);
        assert!(map.obj_groups[0].objs[0].rect.w == 7.71136);
        assert!(map.obj_groups[0].objs[0].rect.h == 12.7269);
        assert!(map.obj_groups[0].polys[1].obj.rect.x == 7.58597);
        assert!(map.obj_groups[0].polys[1].obj.rect.y == 6.33209);
        assert!(map.obj_groups[0].polys[1].obj.rect.w == 0.0);
        assert!(map.obj_groups[0].polys[1].obj.rect.h == 0.0);
        assert!(map.obj_groups[0].polys[1].obj.props.booleans["open"] == true);
        assert!(map.obj_groups[0].polys[1].closed == false);
        assert!(map.obj_groups[0].polys[1].points.iter()
                .map(|Vec2 {x, y}| {
                    (*x as i32,  *y as i32)
                })
                .collect::<Vec<(i32, i32)>>() ==
                vec![
                    (0, 0),
                    (15, -3),
                    (4, 13),
                    ]
        );
        assert!(map.obj_groups[0].polys[0].obj.rect.x == 9.15332);
        assert!(map.obj_groups[0].polys[0].obj.rect.y == 33.7294);
        assert!(map.obj_groups[0].polys[0].obj.rect.w == 0.0);
        assert!(map.obj_groups[0].polys[0].obj.rect.h == 0.0);
        assert!(map.obj_groups[0].polys[0].closed == true);
        assert!(map.obj_groups[0].polys[0].points.iter()
                .map(|Vec2 {x, y}| {
                    (*x as i32,  *y as i32)
                })
                .collect::<Vec<(i32, i32)>>() ==
                vec![
                    (0, 0),
                    (0, -7),
                    (9, -1),
                    ]
        );
        assert!(map.obj_groups[0].text.len() == 1);
        assert!(map.obj_groups[0].text[0].text == "Hello World");
        assert!(map.obj_groups[0].text[0].font_family == "MS Sans Serif");
        assert!(map.obj_groups[0].text[0].colour.r == 98);
        assert!(map.obj_groups[0].text[0].horizontal_align == TextHorizontalAlign::Justify);
        assert!(map.obj_groups[0].text[0].vertical_align == TextVerticalAlign::Center);
        assert!(map.obj_groups[0].text[0].italic == true);
        assert!(map.obj_groups[0].text[0].bold == true);
        assert!(map.obj_groups[0].text[0].wrap == true);
        assert!(map.obj_groups[0].text[0].pixel_size == 29);

        assert!(map.obj_groups[1].info.id == 5);
        assert!(map.obj_groups[1].info.name == "obj2s");
        assert!(map.obj_groups[1].info.index_draw_order == true);
        assert!(map.obj_groups[1].info.offset.x == 5.05);
        assert!(map.obj_groups[1].info.parallax.x == 1.10);
        assert!(map.obj_groups[1].info.colour.r == 85);
        assert!(map.obj_groups[1].info.colour.a == 10);
        assert!(map.obj_groups[1].info.visible == true);
        assert!(map.obj_groups[1].info.locked == true);
        assert!(map.obj_groups[1].info.tint.g == 115);
        assert!(map.obj_groups[1].info.tint.a == 255);
        assert!(map.obj_groups[1].props.booleans["collidable"] == true);
        assert!(map.obj_groups[1].objs.len() == 2);
        assert!(map.obj_groups[1].polys.len() == 0);
        assert!(map.obj_groups[1].objs[1].info.id == 8);
        assert!(map.obj_groups[1].objs[1].rect.x == 10.0);
        assert!(map.obj_groups[1].objs[1].rect.y == 20.0);
        assert!(map.obj_groups[1].objs[1].rect.w == 20.0);
        assert!(map.obj_groups[1].objs[1].rect.h == 0.0);
        assert!(map.obj_groups[1].objs[0].info.id == 9);
        assert!(map.obj_groups[1].objs[0].rect.x == 0.0);
        assert!(map.obj_groups[1].objs[0].rect.y == 0.0);
        assert!(map.obj_groups[1].objs[0].rect.w == 20.0);
        assert!(map.obj_groups[1].objs[0].rect.h == 10.0);
        assert!(map.obj_groups[1].objs[0].props.booleans["test_coll"] == true);

        assert!(map.img_layers.len() == 1);
        assert!(map.img_layers[0].info.offset.x == 19.247);
        assert!(map.img_layers[0].info.offset.y == -10.3445);
        assert!(map.img_layers[0].image_path == "test-tileset.png");
        assert!(map.img_layers[0].width == 32);
        assert!(map.img_layers[0].height == 32);
        assert!(map.img_layers[0].repeat_x == false);
        assert!(map.img_layers[0].repeat_y == true);
        assert!(map.img_layers[0].info.parallax.x == 2.07);
        assert!(map.img_layers[0].info.parallax.y ==  1.0);
        assert!(map.img_layers[0].props.booleans["img"] == false);
    }
}
