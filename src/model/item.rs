//! Typed read + mutate wrapper over a board item protobuf `Any`.

use prost::Message;
use prost_types::Any;

use crate::envelope;
use crate::error::KiCadError;
use crate::proto::kiapi::board::types as bt;

/// Classifies a board item by its protobuf type URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Track,
    Arc,
    Via,
    FootprintInstance,
    Pad,
    BoardGraphicShape,
    BoardText,
    BoardTextBox,
    Field,
    Zone,
    Dimension,
    ReferenceImage,
    Group,
    Unknown(String),
}

impl ItemKind {
    /// Canonical KiCad type name (the suffix of the `Any` type URL).
    pub fn type_name(&self) -> &str {
        match self {
            ItemKind::Track => "kiapi.board.types.Track",
            ItemKind::Arc => "kiapi.board.types.Arc",
            ItemKind::Via => "kiapi.board.types.Via",
            ItemKind::FootprintInstance => "kiapi.board.types.FootprintInstance",
            ItemKind::Pad => "kiapi.board.types.Pad",
            ItemKind::BoardGraphicShape => "kiapi.board.types.BoardGraphicShape",
            ItemKind::BoardText => "kiapi.board.types.BoardText",
            ItemKind::BoardTextBox => "kiapi.board.types.BoardTextBox",
            ItemKind::Field => "kiapi.board.types.Field",
            ItemKind::Zone => "kiapi.board.types.Zone",
            ItemKind::Dimension => "kiapi.board.types.Dimension",
            ItemKind::ReferenceImage => "kiapi.board.types.ReferenceImage",
            ItemKind::Group => "kiapi.board.types.Group",
            ItemKind::Unknown(s) => s.as_str(),
        }
    }
}

/// How an item relates to the board's layer stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerSet {
    /// Item has a single `BoardLayer` enum value, reported as a raw `i32`.
    Single(i32),
    /// Item explicitly enumerates several layers (zones).
    Multi(Vec<i32>),
    /// Item's layer residency is defined by a padstack.
    Padstack,
    /// Item has no layer.
    None,
}

impl LayerSet {
    /// Whether this is a `Single` variant.
    pub fn is_single(&self) -> bool {
        matches!(self, LayerSet::Single(_))
    }
}

/// Round-trippable wrapper over a board item `Any`.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    raw: Any,
}

impl Item {
    /// Wraps a server-produced `Any` payload.
    pub fn from_any(any: Any) -> Self {
        Self { raw: any }
    }

    /// Returns the underlying `Any`, consuming the wrapper.
    pub fn into_any(self) -> Any {
        self.raw
    }

    /// Borrows the underlying `Any`.
    pub fn as_any(&self) -> &Any {
        &self.raw
    }

    /// Returns the full protobuf type URL.
    pub fn type_url(&self) -> &str {
        &self.raw.type_url
    }

    /// Classifies the item by its type URL.
    pub fn kind(&self) -> ItemKind {
        let suffix = self
            .raw
            .type_url
            .rsplit_once('/')
            .map(|(_, s)| s)
            .unwrap_or(self.raw.type_url.as_str());
        match suffix {
            "kiapi.board.types.Track" => ItemKind::Track,
            "kiapi.board.types.Arc" => ItemKind::Arc,
            "kiapi.board.types.Via" => ItemKind::Via,
            "kiapi.board.types.FootprintInstance" => ItemKind::FootprintInstance,
            "kiapi.board.types.Pad" => ItemKind::Pad,
            "kiapi.board.types.BoardGraphicShape" => ItemKind::BoardGraphicShape,
            "kiapi.board.types.BoardText" => ItemKind::BoardText,
            "kiapi.board.types.BoardTextBox" => ItemKind::BoardTextBox,
            "kiapi.board.types.Field" => ItemKind::Field,
            "kiapi.board.types.Zone" => ItemKind::Zone,
            "kiapi.board.types.Dimension" => ItemKind::Dimension,
            "kiapi.board.types.ReferenceImage" => ItemKind::ReferenceImage,
            "kiapi.board.types.Group" => ItemKind::Group,
            other => ItemKind::Unknown(other.to_string()),
        }
    }

    /// Returns the item's KIID, if it carries one.
    pub fn kiid(&self) -> Result<Option<String>, KiCadError> {
        let value = self.raw.value.as_slice();
        let id = match self.kind() {
            ItemKind::Track => bt::Track::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::Arc => bt::Arc::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::Via => bt::Via::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::FootprintInstance => bt::FootprintInstance::decode(value)
                .map_err(decode_err)?
                .id
                .map(|k| k.value),
            ItemKind::Pad => bt::Pad::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::BoardGraphicShape => bt::BoardGraphicShape::decode(value)
                .map_err(decode_err)?
                .id
                .map(|k| k.value),
            ItemKind::BoardText => bt::BoardText::decode(value)
                .map_err(decode_err)?
                .id
                .map(|k| k.value),
            ItemKind::BoardTextBox => bt::BoardTextBox::decode(value)
                .map_err(decode_err)?
                .id
                .map(|k| k.value),
            ItemKind::Zone => bt::Zone::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::Dimension => bt::Dimension::decode(value)
                .map_err(decode_err)?
                .id
                .map(|k| k.value),
            ItemKind::Group => bt::Group::decode(value).map_err(decode_err)?.id.map(|k| k.value),
            ItemKind::Field | ItemKind::ReferenceImage | ItemKind::Unknown(_) => None,
        };
        Ok(id)
    }

    /// Classifies the item's layer residency.
    pub fn layer_set(&self) -> Result<LayerSet, KiCadError> {
        let value = self.raw.value.as_slice();
        let set = match self.kind() {
            ItemKind::Track => LayerSet::Single(bt::Track::decode(value).map_err(decode_err)?.layer),
            ItemKind::Arc => LayerSet::Single(bt::Arc::decode(value).map_err(decode_err)?.layer),
            ItemKind::BoardGraphicShape => {
                LayerSet::Single(bt::BoardGraphicShape::decode(value).map_err(decode_err)?.layer)
            }
            ItemKind::BoardText => {
                LayerSet::Single(bt::BoardText::decode(value).map_err(decode_err)?.layer)
            }
            ItemKind::BoardTextBox => {
                LayerSet::Single(bt::BoardTextBox::decode(value).map_err(decode_err)?.layer)
            }
            ItemKind::Dimension => {
                LayerSet::Single(bt::Dimension::decode(value).map_err(decode_err)?.layer)
            }
            ItemKind::FootprintInstance => LayerSet::Single(
                bt::FootprintInstance::decode(value).map_err(decode_err)?.layer,
            ),
            ItemKind::Zone => LayerSet::Multi(bt::Zone::decode(value).map_err(decode_err)?.layers),
            ItemKind::Via | ItemKind::Pad => LayerSet::Padstack,
            ItemKind::Field | ItemKind::ReferenceImage | ItemKind::Group | ItemKind::Unknown(_) => {
                LayerSet::None
            }
        };
        Ok(set)
    }

    /// Sets the layer of a single-layer item by id.
    pub fn set_layer_id(&mut self, layer_id: i32) -> Result<(), KiCadError> {
        let value = self.raw.value.as_slice();
        let new_bytes = match self.kind() {
            ItemKind::Track => {
                let mut m = bt::Track::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::Arc => {
                let mut m = bt::Arc::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::BoardGraphicShape => {
                let mut m = bt::BoardGraphicShape::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::BoardText => {
                let mut m = bt::BoardText::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::BoardTextBox => {
                let mut m = bt::BoardTextBox::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::Dimension => {
                let mut m = bt::Dimension::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            ItemKind::FootprintInstance => {
                let mut m = bt::FootprintInstance::decode(value).map_err(decode_err)?;
                m.layer = layer_id;
                m.encode_to_vec()
            }
            kind => return Err(unsupported("set_layer_id", kind)),
        };
        self.raw.value = new_bytes;
        Ok(())
    }

    /// Replaces a zone's layer list.
    pub fn set_layers(&mut self, layer_ids: Vec<i32>) -> Result<(), KiCadError> {
        match self.kind() {
            ItemKind::Zone => {
                let mut m = bt::Zone::decode(self.raw.value.as_slice()).map_err(decode_err)?;
                m.layers = layer_ids;
                self.raw.value = m.encode_to_vec();
                Ok(())
            }
            kind => Err(unsupported("set_layers", kind)),
        }
    }

    /// Snaps all positional coordinates of this item to the nearest multiple of the grid.
    ///
    /// `grid_x_nm` / `grid_y_nm` are the grid step in nanometres. `grid_x_nm` / `grid_y_nm` of
    /// `0` or below leaves that axis unchanged.
    ///
    /// Returns `Ok(true)` if the item's bytes were rewritten. Unsupported kinds (Zone, Dimension,
    /// Field, Group, ReferenceImage, Unknown) return `Ok(false)`.
    pub fn snap_position(&mut self, grid_x_nm: i64, grid_y_nm: i64) -> Result<bool, KiCadError> {
        let gx = grid_x_nm;
        let gy = grid_y_nm;
        let value = self.raw.value.as_slice();
        let (new_bytes, mutated) = match self.kind() {
            ItemKind::Track => {
                let mut m = bt::Track::decode(value).map_err(decode_err)?;
                snap_opt_v2(&mut m.start, gx, gy);
                snap_opt_v2(&mut m.end, gx, gy);
                (m.encode_to_vec(), true)
            }
            ItemKind::Arc => {
                let mut m = bt::Arc::decode(value).map_err(decode_err)?;
                snap_opt_v2(&mut m.start, gx, gy);
                snap_opt_v2(&mut m.mid, gx, gy);
                snap_opt_v2(&mut m.end, gx, gy);
                (m.encode_to_vec(), true)
            }
            ItemKind::Via => {
                let mut m = bt::Via::decode(value).map_err(decode_err)?;
                snap_opt_v2(&mut m.position, gx, gy);
                (m.encode_to_vec(), true)
            }
            ItemKind::FootprintInstance => {
                let mut m = bt::FootprintInstance::decode(value).map_err(decode_err)?;
                snap_opt_v2(&mut m.position, gx, gy);
                (m.encode_to_vec(), true)
            }
            ItemKind::Pad => {
                let mut m = bt::Pad::decode(value).map_err(decode_err)?;
                snap_opt_v2(&mut m.position, gx, gy);
                (m.encode_to_vec(), true)
            }
            ItemKind::BoardGraphicShape => {
                let mut m = bt::BoardGraphicShape::decode(value).map_err(decode_err)?;
                if let Some(shape) = m.shape.as_mut() {
                    snap_graphic_shape(shape, gx, gy);
                }
                (m.encode_to_vec(), true)
            }
            ItemKind::BoardText => {
                let mut m = bt::BoardText::decode(value).map_err(decode_err)?;
                if let Some(t) = m.text.as_mut() {
                    snap_opt_v2(&mut t.position, gx, gy);
                }
                (m.encode_to_vec(), true)
            }
            ItemKind::BoardTextBox => {
                let mut m = bt::BoardTextBox::decode(value).map_err(decode_err)?;
                if let Some(tb) = m.textbox.as_mut() {
                    snap_opt_v2(&mut tb.top_left, gx, gy);
                    snap_opt_v2(&mut tb.bottom_right, gx, gy);
                }
                (m.encode_to_vec(), true)
            }
            ItemKind::Zone
            | ItemKind::Dimension
            | ItemKind::Field
            | ItemKind::Group
            | ItemKind::ReferenceImage
            | ItemKind::Unknown(_) => (Vec::new(), false),
        };
        if mutated {
            self.raw.value = new_bytes;
        }
        Ok(mutated)
    }

    /// Builds a new `Group` item wrapping the given member KIIDs.
    ///
    /// The returned `Item` has no `id` set, so `CreateItems` assigns a
    /// fresh one when the server receives it.
    pub fn new_group(name: String, member_kiids: Vec<String>) -> Self {
        use crate::proto::kiapi::common::types as ct;
        let g = bt::Group {
            id: None,
            name,
            items: member_kiids
                .into_iter()
                .map(|v| ct::Kiid { value: v })
                .collect(),
        };
        Item {
            raw: Any {
                type_url: envelope::type_url("kiapi.board.types.Group"),
                value: g.encode_to_vec(),
            },
        }
    }

    /// Returns a group's name. `Ok(None)` for non-Group items.
    pub fn group_name(&self) -> Result<Option<String>, KiCadError> {
        if !matches!(self.kind(), ItemKind::Group) {
            return Ok(None);
        }
        let g = bt::Group::decode(self.raw.value.as_slice()).map_err(decode_err)?;
        Ok(Some(g.name))
    }

    /// Returns a group's immediate member KIIDs. `Ok(None)` for non-Group items.
    pub fn group_members(&self) -> Result<Option<Vec<String>>, KiCadError> {
        if !matches!(self.kind(), ItemKind::Group) {
            return Ok(None);
        }
        let g = bt::Group::decode(self.raw.value.as_slice()).map_err(decode_err)?;
        Ok(Some(g.items.into_iter().map(|k| k.value).collect()))
    }
}

impl From<Any> for Item {
    fn from(value: Any) -> Self {
        Item::from_any(value)
    }
}

impl From<Item> for Any {
    fn from(value: Item) -> Self {
        value.into_any()
    }
}

/// Full protobuf type URL for a given `ItemKind`.
pub fn type_url_for(kind: &ItemKind) -> String {
    envelope::type_url(kind.type_name())
}

fn decode_err(e: prost::DecodeError) -> KiCadError {
    KiCadError::ProtobufDecode(e.to_string())
}

#[inline]
fn snap_coord(v: i64, grid: i64) -> i64 {
    if grid <= 0 { return v; }
    let q = v.div_euclid(grid);
    let r = v.rem_euclid(grid);
    if r * 2 >= grid { (q + 1) * grid } else { q * grid }
}

fn snap_opt_v2(
    v: &mut Option<crate::proto::kiapi::common::types::Vector2>,
    gx: i64,
    gy: i64,
) {
    if let Some(p) = v.as_mut() {
        p.x_nm = snap_coord(p.x_nm, gx);
        p.y_nm = snap_coord(p.y_nm, gy);
    }
}

fn snap_graphic_shape(
    shape: &mut crate::proto::kiapi::common::types::GraphicShape,
    gx: i64,
    gy: i64,
) {
    use crate::proto::kiapi::common::types::graphic_shape::Geometry;
    use crate::proto::kiapi::common::types::poly_line_node::Geometry as NodeGeom;
    if let Some(geo) = shape.geometry.as_mut() {
        match geo {
            Geometry::Segment(s) => {
                snap_opt_v2(&mut s.start, gx, gy);
                snap_opt_v2(&mut s.end, gx, gy);
            }
            Geometry::Rectangle(r) => {
                snap_opt_v2(&mut r.top_left, gx, gy);
                snap_opt_v2(&mut r.bottom_right, gx, gy);
            }
            Geometry::Arc(a) => {
                snap_opt_v2(&mut a.start, gx, gy);
                snap_opt_v2(&mut a.mid, gx, gy);
                snap_opt_v2(&mut a.end, gx, gy);
            }
            Geometry::Circle(c) => {
                snap_opt_v2(&mut c.center, gx, gy);
                snap_opt_v2(&mut c.radius_point, gx, gy);
            }
            Geometry::Polygon(polyset) => {
                for poly in polyset.polygons.iter_mut() {
                    if let Some(outline) = poly.outline.as_mut() {
                        snap_polyline_nodes(&mut outline.nodes, gx, gy);
                    }
                    for h in poly.holes.iter_mut() {
                        snap_polyline_nodes(&mut h.nodes, gx, gy);
                    }
                }
            }
            Geometry::Bezier(b) => {
                snap_opt_v2(&mut b.start, gx, gy);
                snap_opt_v2(&mut b.control1, gx, gy);
                snap_opt_v2(&mut b.control2, gx, gy);
                snap_opt_v2(&mut b.end, gx, gy);
            }
        }
    }

    fn snap_polyline_nodes(
        nodes: &mut [crate::proto::kiapi::common::types::PolyLineNode],
        gx: i64,
        gy: i64,
    ) {
        for node in nodes.iter_mut() {
            if let Some(g) = node.geometry.as_mut() {
                match g {
                    NodeGeom::Point(p) => {
                        p.x_nm = snap_coord(p.x_nm, gx);
                        p.y_nm = snap_coord(p.y_nm, gy);
                    }
                    NodeGeom::Arc(a) => {
                        if let Some(p) = a.start.as_mut() {
                            p.x_nm = snap_coord(p.x_nm, gx);
                            p.y_nm = snap_coord(p.y_nm, gy);
                        }
                        if let Some(p) = a.mid.as_mut() {
                            p.x_nm = snap_coord(p.x_nm, gx);
                            p.y_nm = snap_coord(p.y_nm, gy);
                        }
                        if let Some(p) = a.end.as_mut() {
                            p.x_nm = snap_coord(p.x_nm, gx);
                            p.y_nm = snap_coord(p.y_nm, gy);
                        }
                    }
                }
            }
        }
    }
}

fn unsupported(op: &str, kind: ItemKind) -> KiCadError {
    KiCadError::InvalidResponse {
        reason: format!("{op} not supported for {}", kind.type_name()),
    }
}
