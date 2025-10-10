// Often UX/UI designers will organise visually, without having a clear hierarchy in the underlying
// frame structure. This can make it complex retrospectively to add animations etc., as exported
// svgs are in a non-predictable structure.
//
// We want to use the figma api to sanitize the nodes. This will use coordinates / text information to
// logically group the nodes into frames / groups / components etc.
//
// This is especially useful for staggered entry animations where we need predictable groupings.

use figma_api::models::{Rectangle, SubcanvasNode};

/// Represents a bounding box with absolute coordinates
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn from_rectangle(rect: &Rectangle) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Check if this box contains another box
    pub fn contains(&self, other: &BoundingBox) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && (other.x + other.width) <= (self.x + self.width)
            && (other.y + other.height) <= (self.y + self.height)
    }

    /// Calculate the center point of this bounding box
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Calculate distance between two bounding boxes (center to center)
    pub fn distance_to(&self, other: &BoundingBox) -> f64 {
        let (x1, y1) = self.center();
        let (x2, y2) = other.center();
        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }

    /// Check if two boxes are aligned horizontally (within tolerance)
    pub fn horizontally_aligned_with(&self, other: &BoundingBox, tolerance: f64) -> bool {
        (self.y - other.y).abs() < tolerance
            || (self.center().1 - other.center().1).abs() < tolerance
    }

    /// Check if two boxes are aligned vertically (within tolerance)
    pub fn vertically_aligned_with(&self, other: &BoundingBox, tolerance: f64) -> bool {
        (self.x - other.x).abs() < tolerance
            || (self.center().0 - other.center().0).abs() < tolerance
    }

    /// Calculate overlap area with another box
    pub fn overlap_area(&self, other: &BoundingBox) -> f64 {
        let x_overlap = f64::max(
            0.0,
            f64::min(self.x + self.width, other.x + other.width) - f64::max(self.x, other.x),
        );
        let y_overlap = f64::max(
            0.0,
            f64::min(self.y + self.height, other.y + other.height) - f64::max(self.y, other.y),
        );
        x_overlap * y_overlap
    }
}

/// Represents a node with its spatial information
#[derive(Debug, Clone)]
pub struct SpatialNode {
    pub id: String,
    pub name: String,
    pub bounds: BoundingBox,
    pub node_type: NodeType,
    pub original_node: SubcanvasNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Text,
    Vector,
    Rectangle,
    Ellipse,
    Frame,
    Group,
    Component,
    Instance,
    Other,
}

impl SpatialNode {
    /// Extract spatial information from a SubcanvasNode
    pub fn from_subcanvas_node(node: &SubcanvasNode) -> Option<Self> {
        let (id, name, bounds, node_type) = match node {
            SubcanvasNode::Text(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Text,
            ),
            SubcanvasNode::Vector(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Vector,
            ),
            SubcanvasNode::Rectangle(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Rectangle,
            ),
            SubcanvasNode::Ellipse(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Ellipse,
            ),
            SubcanvasNode::Frame(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Frame,
            ),
            SubcanvasNode::Group(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Group,
            ),
            SubcanvasNode::Component(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Component,
            ),
            SubcanvasNode::Instance(n) => (
                n.id.clone(),
                n.name.clone(),
                n.absolute_bounding_box.as_ref()?,
                NodeType::Instance,
            ),
            _ => return None,
        };

        Some(Self {
            id,
            name,
            bounds: BoundingBox::from_rectangle(bounds),
            node_type,
            original_node: node.clone(),
        })
    }
}

/// Configuration for node sanitization
#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    /// Maximum distance between nodes to consider them part of the same group (in pixels)
    pub proximity_threshold: f64,
    /// Alignment tolerance (in pixels)
    pub alignment_tolerance: f64,
    /// Minimum number of nodes to form a group
    pub min_group_size: usize,
    /// Whether to detect grid layouts (useful for staggered animations)
    pub detect_grids: bool,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            proximity_threshold: 100.0,
            alignment_tolerance: 5.0,
            min_group_size: 2,
            detect_grids: true,
        }
    }
}

/// A logical grouping of nodes based on spatial analysis
#[derive(Debug, Clone)]
pub struct NodeGroup {
    pub id: String,
    pub name: String,
    pub nodes: Vec<SpatialNode>,
    pub bounds: BoundingBox,
    pub group_type: GroupType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {
    /// Nodes arranged in a grid (e.g., card grid)
    Grid { rows: usize, cols: usize },
    /// Nodes arranged horizontally
    HorizontalList,
    /// Nodes arranged vertically
    VerticalList,
    /// Nodes spatially close but no clear pattern
    Cluster,
    /// A single node that should be kept separate
    Single,
}

impl NodeGroup {
    /// Calculate the overall bounding box for a group of nodes
    fn calculate_bounds(nodes: &[SpatialNode]) -> BoundingBox {
        if nodes.is_empty() {
            return BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }

        let min_x = nodes
            .iter()
            .map(|n| n.bounds.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = nodes
            .iter()
            .map(|n| n.bounds.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = nodes
            .iter()
            .map(|n| n.bounds.x + n.bounds.width)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = nodes
            .iter()
            .map(|n| n.bounds.y + n.bounds.height)
            .fold(f64::NEG_INFINITY, f64::max);

        BoundingBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    /// Detect if nodes form a grid pattern
    fn detect_grid_pattern(
        nodes: &[SpatialNode],
        config: &SanitizationConfig,
    ) -> Option<(usize, usize)> {
        if nodes.len() < 4 {
            return None;
        }

        // Group by rows (nodes with similar y coordinates)
        let mut rows: Vec<Vec<&SpatialNode>> = Vec::new();
        let mut sorted_nodes: Vec<&SpatialNode> = nodes.iter().collect();
        sorted_nodes.sort_by(|a, b| a.bounds.y.partial_cmp(&b.bounds.y).unwrap());

        for node in sorted_nodes {
            let mut added = false;
            for row in &mut rows {
                if let Some(first) = row.first() {
                    if first
                        .bounds
                        .horizontally_aligned_with(&node.bounds, config.alignment_tolerance)
                    {
                        row.push(node);
                        added = true;
                        break;
                    }
                }
            }
            if !added {
                rows.push(vec![node]);
            }
        }

        // Check if we have consistent column counts
        if rows.len() < 2 {
            return None;
        }

        let col_count = rows[0].len();
        if rows.iter().all(|row| row.len() == col_count) && col_count >= 2 {
            return Some((rows.len(), col_count));
        }

        None
    }

    fn detect_list_pattern(nodes: &[SpatialNode], config: &SanitizationConfig) -> GroupType {
        // Check if nodes are primarily arranged horizontally or vertically
        let horizontal_alignment = nodes
            .windows(2)
            .filter(|pair| {
                pair[0]
                    .bounds
                    .horizontally_aligned_with(&pair[1].bounds, config.alignment_tolerance)
            })
            .count();

        let vertical_alignment = nodes
            .windows(2)
            .filter(|pair| {
                pair[0]
                    .bounds
                    .vertically_aligned_with(&pair[1].bounds, config.alignment_tolerance)
            })
            .count();

        if horizontal_alignment > vertical_alignment && horizontal_alignment > 0 {
            GroupType::HorizontalList
        } else if vertical_alignment > 0 {
            GroupType::VerticalList
        } else {
            GroupType::Cluster
        }
    }

    /// Create a node group from spatial nodes
    pub fn from_nodes(nodes: Vec<SpatialNode>, config: &SanitizationConfig) -> Self {
        let bounds = Self::calculate_bounds(&nodes);
        let group_type = if nodes.len() == 1 {
            GroupType::Single
        } else if config.detect_grids {
            if let Some((rows, cols)) = Self::detect_grid_pattern(&nodes, config) {
                GroupType::Grid { rows, cols }
            } else {
                Self::detect_list_pattern(&nodes, config)
            }
        } else {
            Self::detect_list_pattern(&nodes, config)
        };

        let name = match &group_type {
            GroupType::Grid { rows, cols } => format!("Grid_{}x{}", rows, cols),
            GroupType::HorizontalList => "HorizontalList".to_string(),
            GroupType::VerticalList => "VerticalList".to_string(),
            GroupType::Cluster => "Cluster".to_string(),
            GroupType::Single => nodes[0].name.clone(),
        };

        // Generate a simple ID (you may want to use uuid crate here)
        let id = format!(
            "group_{}",
            nodes.iter().map(|n| &n.id).collect::<Vec<_>>().join("_")
        );

        Self {
            id,
            name,
            nodes,
            bounds,
            group_type,
        }
    }
}

/// Main sanitizer that analyzes and restructures node hierarchies
pub struct NodeSanitizer {
    config: SanitizationConfig,
}

impl NodeSanitizer {
    pub fn new(config: SanitizationConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(SanitizationConfig::default())
    }

    /// Flatten a node tree into a list of spatial nodes
    pub fn flatten_nodes(&self, nodes: &[SubcanvasNode]) -> Vec<SpatialNode> {
        let mut spatial_nodes = Vec::new();
        self.flatten_recursive(nodes, &mut spatial_nodes);
        spatial_nodes
    }

    fn flatten_recursive(&self, nodes: &[SubcanvasNode], output: &mut Vec<SpatialNode>) {
        for node in nodes {
            if let Some(spatial_node) = SpatialNode::from_subcanvas_node(node) {
                output.push(spatial_node);
            }

            // Recursively process children
            let children = match node {
                SubcanvasNode::Frame(n) => &n.children,
                SubcanvasNode::Group(n) => &n.children,
                SubcanvasNode::Component(n) => &n.children,
                SubcanvasNode::Instance(n) => &n.children,
                SubcanvasNode::Section(n) => &n.children,
                _ => continue,
            };

            self.flatten_recursive(children, output);
        }
    }

    /// Group nodes based on spatial proximity and patterns
    pub fn group_nodes(&self, nodes: Vec<SpatialNode>) -> Vec<NodeGroup> {
        if nodes.is_empty() {
            return Vec::new();
        }

        // Simple clustering based on proximity
        let mut groups: Vec<Vec<SpatialNode>> = Vec::new();
        let mut ungrouped: Vec<SpatialNode> = nodes;

        while !ungrouped.is_empty() {
            let seed = ungrouped.remove(0);
            let mut current_group = vec![seed];

            // Find all nodes within proximity threshold
            let mut i = 0;
            while i < ungrouped.len() {
                let candidate = &ungrouped[i];
                let is_close = current_group.iter().any(|group_node| {
                    group_node.bounds.distance_to(&candidate.bounds)
                        < self.config.proximity_threshold
                });

                if is_close {
                    current_group.push(ungrouped.remove(i));
                } else {
                    i += 1;
                }
            }

            groups.push(current_group);
        }

        // Convert to NodeGroups
        groups
            .into_iter()
            .filter(|g| g.len() >= self.config.min_group_size || g.len() == 1)
            .map(|nodes| NodeGroup::from_nodes(nodes, &self.config))
            .collect()
    }

    /// Main entry point: sanitize a node tree for animation-friendly structure
    pub fn sanitize(&self, nodes: &[SubcanvasNode]) -> Vec<NodeGroup> {
        let spatial_nodes = self.flatten_nodes(nodes);
        self.group_nodes(spatial_nodes)
    }
}
