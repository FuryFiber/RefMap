use egui::{Id, Margin, Pos2, UiKind};
use rfd::FileDialog;
use uuid::Uuid;
use crate::core::map::{EdgeType, Node, Tag};
use crate::core::MindMap;
use crate::core::storage::{export_project, load_last_file, load_map, save_last_file, save_map};
use crate::core::pdfparser::Metadata;


pub struct MindMapApp {
    map: MindMap,                       // the mind map data

    // Interaction state
    dragging_node: Option<Uuid>,        // currently dragged node
    connecting_from: Option<Uuid>,      // node from which a connection is being made
    selected_nodes: Vec<Uuid>,          // currently selected nodes
    selected_edges: Vec<Uuid>,          // currently selected edges

    // For marquee selection
    marquee_start: Option<Pos2>,        // where the marquee drag started
    marquee_rect: Option<egui::Rect>,   // current marquee rectangle

    // View state
    pan: egui::Vec2,                    // panning offset
    zoom: f32,                          // zoom level

    // File state
    current_file: Option<String>,       // currently opened file path
    last_save: std::time::Instant,      // last save time for autosave
    dirty: bool,                        // whether there are unsaved changes

    // Right-click context for nodes
    rightclick_node: Option<Uuid>,      // node that was right-clicked
    show_node_context_menu: bool,       // whether context menu should be visible
    context_menu_pos: Pos2,             // position to show context menu

    // Metadata editing state
    show_edit_dialog: bool,             // whether to show metadata edit dialog
    edit_node_id: Option<Uuid>,         // node currently being edited
    edit_metadata: EditableMetadata,    // editable metadata fields

    // Annotation state
    show_annotations_panel: bool,       // whether to show annotations panel
    show_add_annotation_dialog: bool,   // whether to show add annotation dialog
    show_edit_annotation_dialog: bool,  // whether to show edit annotation dialog
    edit_annotation_id: Option<Uuid>,   // annotation currently being edited
    edit_annotation: EditableAnnotation,// editable annotation fields
    annotations_node_id: Option<Uuid>,  // node whose annotations are being viewed/edited

    // Edge creation state
    pending_edge_from: Option<Uuid>,    // node where edge creation started
    pending_edge_to: Option<Uuid>,      // node where edge creation ends
    show_edge_type_menu: bool,          // whether to show edge type selection menu
    selected_edge_type: EdgeType,       // selected edge type for new edge
    edge_type_menu_pos: Option<Pos2>,   // position to show edge type menu

    // Right-click context for edges
    rightclick_edge: Option<Uuid>,      // Edge that was right-clicked
    show_edge_context_menu: bool,       // Whether to show the edge context menu
    edge_context_menu_pos: Pos2,        // Position to show the edge context menu

    // Project creation prompt state
    show_create_project_prompt: bool,   // Whether to show the prompt
    pending_pdf_path: Option<String>,   // PDF path to add after project creation

    // Color picker state
    show_node_color_picker: bool,       // Whether to show node color picker
    node_color_picker_id: Option<Uuid>, // Node ID for which color picker is shown
    selected_node_color: egui::Color32, // Currently selected color

    show_edge_color_picker: bool,       // Whether to show edge color picker
    edge_color_picker_id: Option<Uuid>, // Edge ID for which color picker is shown
    selected_edge_color: egui::Color32, // Currently selected edge color

    // Annotation state
    show_tags_panel: bool,              // whether to show tags panel
    show_add_tag_dialog: bool,         // whether to show add tag dialog
    show_edit_tag_dialog: bool,         // whether to show edit tag dialog
    edit_tag_id: Option<Uuid>,          // tag currently being edited
    edit_tag: EditableTag,              // editable tag fields
    tags_node_id: Option<Uuid>,          // node whose tags are being viewed/edited
}

// Helper struct for editing metadata
#[derive(Debug, Clone, Default)]
struct EditableMetadata {
    title: String,
    authors: String,
    keywords: String,
    date: String,
}

#[derive(Debug, Clone, Default)]
struct EditableTag {
    name: String,
    color: egui::Color32,
}

// Annotation types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[derive(PartialEq)]
pub enum AnnotationType {
    #[default]
    TextNote,
    Question,
    Summary,
    Quote,
    Todo,
    Idea,
    Warning,
}

impl AnnotationType {
    fn name(&self) -> &'static str {
        match self {
            AnnotationType::TextNote => "Text Note",
            AnnotationType::Question => "Question",
            AnnotationType::Summary => "Summary",
            AnnotationType::Quote => "Quote",
            AnnotationType::Todo => "Todo",
            AnnotationType::Idea => "Idea",
            AnnotationType::Warning => "Warning",
        }
    }

    fn color(&self) -> egui::Color32 {
        match self {
            AnnotationType::TextNote => egui::Color32::LIGHT_GRAY,
            AnnotationType::Question => egui::Color32::LIGHT_BLUE,
            AnnotationType::Summary => egui::Color32::LIGHT_GREEN,
            AnnotationType::Quote => egui::Color32::YELLOW,
            AnnotationType::Todo => egui::Color32::LIGHT_RED,
            AnnotationType::Idea => egui::Color32::from_rgb(255, 200, 100),
            AnnotationType::Warning => egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub annotation_type: AnnotationType,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub page_number: Option<u32>, // For PDF annotations
}

// Add new struct for annotation editing
#[derive(Debug, Clone, Default)]
struct EditableAnnotation {
    title: String,
    content: String,
    annotation_type: AnnotationType,
    page_number: String,
}

impl MindMapApp {
    fn main_view(&mut self, ctx: &egui::Context) {
        let main_frame = egui::containers::Frame {
            inner_margin: Default::default(),
            fill: egui::Color32::from_hex("#30313c").unwrap(),
            stroke: Default::default(),
            corner_radius: Default::default(),
            outer_margin: Default::default(),
            shadow: Default::default(),
        };
        egui::CentralPanel::default().frame(main_frame).show(ctx, |ui| {
            ui.heading("RefMap");

            // Create a canvas region that captures click + drag
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                egui::Sense::click_and_drag(),
            );
            let rect = response.rect;

            // --- Handle panning with middle mouse ---
            self.handle_navigation(ctx, &response, rect);

            // --- Handle keyboard events ---
            self.handle_keyboard_events(ctx);

            // --- Handle mouse events ---
            self.handle_mouse_events(ctx, &response, rect);

            // --- Draw edges ---
            self.draw_edges(rect, &painter);

            // Draw pending edges
            self.draw_pending_edges(rect, &painter);

            // --- Draw temporary connection line (while dragging) ---
            self.draw_connection_line(rect, &painter, &response);

            // --- Draw nodes ---
            self.draw_nodes(rect, &painter, ctx);

            // --- Draw marquee rectangle ---
            self.draw_marquee_rect(&painter);
        });
    }

    fn handle_navigation(&mut self, ctx: &egui::Context, response: &egui::Response, rect: egui::Rect) {
        if response.dragged_by(egui::PointerButton::Middle) || (response.dragged_by(egui::PointerButton::Primary) && ctx.input(|s|s.modifiers.shift)) {
            self.pan += response.drag_delta();
        }

        // --- Handle zoom with scroll wheel ---
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > f32::EPSILON {
            if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                // Zoom relative to cursor
                let canvas_pos = (pointer_pos - rect.min.to_vec2() - self.pan) / self.zoom;
                self.zoom *= zoom_delta.clamp(0.1, 5.0);
                // Adjust pan so zoom centers around cursor
                self.pan = pointer_pos - rect.min.to_vec2() - canvas_pos * self.zoom;
            }
        }
    }

    fn handle_keyboard_events(&mut self, ctx: &egui::Context){
        // --- Handle key input for deletion ---
        self.handle_delete(ctx);

        // unselect all on Escape
        self.handle_esc(ctx);


        // manual save
        self.handle_save(ctx);
    }

    fn handle_save(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            if let Some(path) = self.current_file.clone(){
                let _ = save_map(&self.map, &path);
            }
            else {
                self.save();
            }
        }
    }

    fn handle_esc(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_nodes = Vec::new();
            self.selected_edges= Vec::new();
        }
    }

    fn handle_delete(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            // Remove selected nodes
            for node_id in &self.selected_nodes {
                self.map.remove_node(*node_id);
            }
            self.selected_nodes.clear();

            // Remove selected edges
            self.map.edges.retain(|e| !self.selected_edges.contains(&e.id));
            self.selected_edges.clear();
            self.dirty = true;
        }
    }

    fn handle_mouse_events(&mut self, ctx: &egui::Context, response: &egui::Response, rect: egui::Rect) {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos =
                (pointer_pos - rect.min.to_vec2() - self.pan) / self.zoom;

            // Left click selection / creation
            self.handle_left_click(ctx, &response, rect, pointer_pos, canvas_pos);

            // Drag existing node with left mouse
            self.handle_left_drag(ctx, &response, canvas_pos);

            // Right click handling - Updated to show context menu
            self.handle_right_click(ctx, &response, pointer_pos, canvas_pos);

            // --- Right button: create connections ---
            self.handle_right_drag(ctx, &response, canvas_pos);
        }
    }

    fn handle_right_drag(&mut self, ctx: &egui::Context, response: &egui::Response, canvas_pos: Pos2){
        if response.drag_started_by(egui::PointerButton::Secondary) {
            // start connection from node under cursor
            self.start_edge(ctx, canvas_pos)
        }

        if response.drag_stopped_by(egui::PointerButton::Secondary) {
            if let Some(start_id) = self.connecting_from.take() {
                self.stop_edge(ctx, canvas_pos, start_id);
            }
        }
    }

    fn handle_right_click(&mut self, ctx: &egui::Context, response: &egui::Response, pointer_pos: Pos2, canvas_pos: Pos2) {
        if response.clicked_by(egui::PointerButton::Secondary) {
            let mut clicked_any = false;

            // Check if clicked on a node
            for node in &self.map.nodes {
                let node_rect = get_node_rect(ctx, node, self.zoom);
                if node_rect.contains(canvas_pos) {
                    // Show context menu for this node
                    self.rightclick_node = Some(node.id);
                    self.context_menu_pos = pointer_pos;
                    self.show_node_context_menu = true;
                    clicked_any = true;
                    break;
                }
            }

            if !clicked_any {
                // Check if clicked on an edge
                for edge in &self.map.edges {
                    let from = self.map.nodes.iter().find(|n| n.id == edge.from);
                    let to = self.map.nodes.iter().find(|n| n.id == edge.to);
                    if let (Some(f), Some(t)) = (from, to) {
                        let dist = point_line_distance(
                            egui::pos2(f.x, f.y),
                            egui::pos2(t.x, t.y),
                            canvas_pos,
                        );
                        if dist < 8.0 {
                            // Right-clicked on an edge
                            self.rightclick_edge = Some(edge.id);
                            self.edge_context_menu_pos = pointer_pos;
                            self.show_edge_context_menu = true;
                            clicked_any = true;
                            break;
                        }
                    }
                }
            }

            // If didn't click on anything, close context menu
            if !clicked_any {
                self.show_node_context_menu = false;
                self.show_edge_context_menu = false;
            }
        }
    }

    fn handle_left_drag(&mut self, ctx: &egui::Context, response: &egui::Response, canvas_pos: Pos2) {
        if response.dragged_by(egui::PointerButton::Primary) && !ctx.input(|s| s.modifiers.shift) {
            if ctx.input(|i| i.modifiers.ctrl) {
                // Preserve the connecting_from state during drag
                if self.connecting_from.is_none() {
                    self.start_edge(ctx, canvas_pos);
                }
            }else {
                let pointer_pos = response.interact_pointer_pos().unwrap();

                if let Some(id) = self.dragging_node {
                    if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == id) {
                        node.x += response.drag_delta().x / self.zoom;
                        node.y += response.drag_delta().y / self.zoom;
                        self.dirty = true;
                    }
                } else {
                    for node in &self.map.nodes {
                        let rect = get_node_rect(ctx, node, self.zoom);
                        if rect.contains(canvas_pos) && self.marquee_rect == None {
                            self.dragging_node = Some(node.id);
                            break;
                        }
                    }
                }
                if self.dragging_node.is_none() {
                    // Start or continue marquee
                    if self.marquee_start.is_none() {
                        self.marquee_start = Some(pointer_pos);
                    }
                    self.marquee_rect = Some(egui::Rect::from_two_pos(
                        self.marquee_start.unwrap(),
                        pointer_pos,
                    ));
                }
            }
        }

        // When left button released
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(start_id) = self.connecting_from.take() {
                self.stop_edge(ctx, canvas_pos, start_id);
            } else if let Some(rect) = self.marquee_rect.take() {
                // Convert rect to canvas coordinates
                let rect_min = (rect.min - response.rect.min.to_vec2() - self.pan) / self.zoom;
                let rect_max = (rect.max - response.rect.min.to_vec2() - self.pan) / self.zoom;
                let selection_rect = egui::Rect::from_two_pos(rect_min, rect_max);

                // Select all nodes inside marquee
                self.selected_nodes = Vec::new(); // clear single selection
                self.selected_edges = Vec::new(); // clear edge selection
                for node in &self.map.nodes {
                    let node_pos = egui::pos2(node.x, node.y);
                    if selection_rect.contains(node_pos) {
                        // You can keep a Vec<Uuid> for multiple selection
                        // For simplicity here, we just mark the last node selected
                        self.selected_nodes.push(node.id);
                    }
                }

                // Select edges where both endpoints are inside rect
                for edge in &self.map.edges {
                    let from = self.map.nodes.iter().find(|n| n.id == edge.from);
                    let to = self.map.nodes.iter().find(|n| n.id == edge.to);
                    if let (Some(f), Some(t)) = (from, to) {
                        let p1 = egui::pos2(f.x, f.y);
                        let p2 = egui::pos2(t.x, t.y);
                        if selection_rect.contains(p1) && selection_rect.contains(p2) {
                            self.selected_edges.push(edge.id);
                        }
                    }
                }

                self.marquee_start = None;
            }

            self.dragging_node = None;
        }
    }

    fn handle_left_click(&mut self, ctx: &egui::Context, response: &egui::Response, rect: egui::Rect, pointer_pos: Pos2, canvas_pos: Pos2) {
        if response.clicked_by(egui::PointerButton::Primary) {
            // hide context menu on any left click
            self.show_node_context_menu = false;

            let mut clicked_any = false;

            if ctx.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary)) {  // Check for double click
                let canvas_pos = (pointer_pos - rect.min.to_vec2() - self.pan) / self.zoom;
                let mut clicked_node = false;

                // Check if double-clicked on a node
                for node in &self.map.nodes {
                    let node_rect = get_node_rect(ctx, node, self.zoom);

                    if node_rect.contains(canvas_pos) {
                        if let Some(file_path) = &node.path {
                            if let Some(project_dir) = &self.current_file {
                                let full_path = std::path::Path::new(project_dir).join(file_path);
                                if let Err(e) = opener::open(full_path.to_str().unwrap()) {
                                    eprintln!("Failed to open PDF: {}", e);
                                }
                            } else {
                                eprintln!("No project directory set; cannot open PDF.");
                            }
                        }
                        clicked_node = true;
                        break;
                    }
                }

                if !clicked_node {
                    // Double-clicked on empty space: create new node
                    self.map.add_node("New".into(), canvas_pos.x, canvas_pos.y);
                    self.dirty = true;
                }
            }

            if ctx.input(|i| i.modifiers.ctrl) {
                let canvas_pos = (response.interact_pointer_pos().unwrap() - rect.min.to_vec2() - self.pan) / self.zoom;
                for node in &mut self.map.nodes{
                    let node_rect = get_node_rect(ctx, node, self.zoom);
                    if node_rect.contains(canvas_pos) {
                        if ctx.input(|i| i.modifiers.ctrl) {
                            if node.collapsed{
                                node.collapsed = false;
                            } else {
                                node.collapsed = true;
                            }
                        }
                    }
                }
            }
            else if ctx.input(|i| i.modifiers.shift) {
                let mut clicked_any: bool = false;
                for node in &self.map.nodes {
                    let node_rect = get_node_rect(ctx, node, self.zoom);
                    if node_rect.contains(canvas_pos) {
                        if self.selected_nodes.contains(&node.id){
                            self.selected_nodes.retain(|n| *n != node.id)
                        } else {
                            self.selected_nodes.push(node.id);
                        }
                        clicked_any = true;
                        break;
                    }
                }
                if !clicked_any {
                    for edge in &self.map.edges {
                        let from = self.map.nodes.iter().find(|n| n.id == edge.from);
                        let to = self.map.nodes.iter().find(|n| n.id == edge.to);
                        if let (Some(f), Some(t)) = (from, to) {
                            let dist = point_line_distance(
                                egui::pos2(f.x, f.y),
                                egui::pos2(t.x, t.y),
                                canvas_pos,
                            );
                            if dist < 8.0 {
                                if self.selected_edges.contains(&edge.id){
                                    self.selected_edges.retain(|n| *n != edge.id)
                                } else {
                                    self.selected_edges.push(edge.id);
                                }
                                clicked_any = true;
                                break;
                            }
                        }
                    }
                }
                if !clicked_any {
                    // Shift + left click → open PDF file picker
                    if let Some(path) = FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .pick_file()
                    {
                        let path_str = path.to_str().unwrap().to_string();
                        if let Some(project_dir) = &self.current_file {
                            println!("{}", project_dir);
                            let pdfs_dir = std::path::Path::new(project_dir).join("pdfs");
                            if !pdfs_dir.exists() {
                                std::fs::create_dir_all(&pdfs_dir).expect("failed to create pdfs directory");
                            }
                            let file_name = path.file_name().unwrap().to_str().unwrap();
                            let dest_path = pdfs_dir.join(file_name);
                            std::fs::copy(&path, &dest_path).expect("failed to copy pdf");
                            self.map.add_pdf_node(&format!("{}/{}",pdfs_dir.to_str().unwrap(), file_name), canvas_pos.x, canvas_pos.y).expect("failed to add pdf node");
                            self.dirty = true;
                        } else {
                            // Handle case where no project is saved yet
                            self.pending_pdf_path = Some(path_str);
                            self.show_create_project_prompt = true;
                        }
                    }
                }
            }else{
                // Check if clicked on a node
                for node in &self.map.nodes {
                    let node_rect = get_node_rect(ctx, node, self.zoom);
                    if node_rect.contains(canvas_pos) {
                        self.selected_nodes= Vec::new();
                        self.selected_nodes.push(node.id);
                        self.selected_edges = Vec::new();
                        clicked_any = true;
                        break;
                    }
                }

                // Check if clicked on an edge (line proximity)
                if !clicked_any {
                    for edge in &self.map.edges {
                        let from = self.map.nodes.iter().find(|n| n.id == edge.from);
                        let to = self.map.nodes.iter().find(|n| n.id == edge.to);
                        if let (Some(f), Some(t)) = (from, to) {
                            let dist = point_line_distance(
                                egui::pos2(f.x, f.y),
                                egui::pos2(t.x, t.y),
                                canvas_pos,
                            );
                            if dist < 8.0 {
                                self.selected_edges = Vec::new();
                                self.selected_edges.push(edge.id);
                                self.selected_nodes = Vec::new();
                                clicked_any = true;
                                break;
                            }
                        }
                    }
                }

                // Clicked on empty space: deselect
                if !clicked_any {
                    self.selected_nodes = Vec::new();
                    self.selected_edges = Vec::new();
                }
            }
        }
    }

    fn draw_edges(&mut self, rect: egui::Rect, painter: &egui::Painter) {
        for edge in &self.map.edges {
            let from = self.map.nodes.iter().find(|n| n.id == edge.from);
            let to = self.map.nodes.iter().find(|n| n.id == edge.to);
            if let (Some(f), Some(t)) = (from, to) {
                let p1 = egui::pos2(f.x, f.y) * self.zoom + self.pan + rect.min.to_vec2();
                let p2 = egui::pos2(t.x, t.y) * self.zoom + self.pan + rect.min.to_vec2();
                let fill = if let Some(edge_color) = edge.color {
                    edge_color
                } else {
                    egui::Color32::GRAY.to_array()
                };
                let mut width = 2.0;
                if self.selected_edges.contains(&edge.id) {
                    width = 3.0;
                }

                // Draw line
                painter.line_segment([p1, p2], egui::Stroke::new(width, egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3])));

                // Draw arrowhead for References
                if edge.edge_type == EdgeType::References {
                    let arrow_size = 15.0; // Size of the arrowhead
                    let angle = std::f32::consts::PI / 6.0; // 30 degrees
                    let dir = (p2 - p1).normalized(); // Direction of the edge
                    let perp = egui::Vec2::new(-dir.y, dir.x); // Perpendicular vector

                    // Calculate the midpoint of the edge
                    let midpoint = (p1 + p2.to_vec2()) * 0.5;

                    // Calculate the arrowhead points
                    let p1_arrow = midpoint - dir * arrow_size * 0.5; // Base of the arrowhead
                    let p2_arrow = midpoint + dir * arrow_size * 0.5; // Tip of the arrowhead
                    let p3_arrow = p1_arrow + perp * arrow_size * angle.tan(); // One side of the arrowhead
                    let p4_arrow = p1_arrow - perp * arrow_size * angle.tan(); // Other side of the arrowhead

                    painter.line_segment([p2_arrow, p3_arrow], egui::Stroke::new(width, egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3]))); // Line from tip to one side
                    painter.line_segment([p2_arrow, p4_arrow], egui::Stroke::new(width, egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3]))); // Line from tip to other side
                }
            }
        }
    }

    fn draw_pending_edges(&mut self, rect: egui::Rect, painter: &egui::Painter) {
        if let (Some(from), Some(to)) = (self.pending_edge_from, self.pending_edge_to) {
            if let (Some(f), Some(t)) = (
                self.map.nodes.iter().find(|n| n.id == from),
                self.map.nodes.iter().find(|n| n.id == to),
            ) {
                let p1 = egui::pos2(f.x, f.y) * self.zoom + self.pan + rect.min.to_vec2();
                let p2 = egui::pos2(t.x, t.y) * self.zoom + self.pan + rect.min.to_vec2();
                painter.line_segment([p1, p2], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
            }
        }
    }

    fn draw_connection_line(&mut self, rect: egui::Rect, painter: &egui::Painter, response: &egui::Response) {
        if let Some(start_id) = self.connecting_from {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if let Some(start_node) = self.map.nodes.iter().find(|n| n.id == start_id) {
                    let p1 = egui::pos2(start_node.x, start_node.y) * self.zoom
                        + self.pan
                        + rect.min.to_vec2();
                    let p2 = pointer_pos;
                    painter.line_segment([p1, p2], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                }
            }
        }
    }

    fn draw_nodes(&mut self, rect: egui::Rect, painter: &egui::Painter, ctx: &egui::Context) {
        for node in &mut self.map.nodes {
            let pos = egui::pos2(node.x, node.y) * self.zoom + self.pan + rect.min.to_vec2();
            let font_id = egui::FontId::proportional(14.0 * self.zoom);
            let bold_font_id = egui::FontId::monospace(14.0 * self.zoom);

            let padding = egui::vec2(16.0, 12.0) * self.zoom;

            // Calculate node size based on collapse state
            let mut node_size = if node.collapsed {
                // When collapsed, size based on title
                let title_galley = ctx.fonts_mut(|f| f.layout_no_wrap(node.title.clone(), font_id.clone(), egui::Color32::BLACK));
                let text_size = title_galley.size();
                egui::vec2(text_size.x + padding.x * 2.0, text_size.y + padding.y)
            } else {
                // When expanded, use minimum width or title width, whichever is larger
                let title_galley = ctx.fonts_mut(|f| f.layout_no_wrap(node.title.clone(), font_id.clone(), egui::Color32::BLACK));
                let title_width = title_galley.size().x + padding.x * 2.0;
                let min_width = 300.0 * self.zoom;
                let node_width = title_width.max(min_width);
                egui::vec2(node_width, 0.0) // Height will be calculated below
            };

            let mut metadata_galleys = Vec::new();
            if !node.collapsed {
                if let Some(metadata) = &node.metadata {
                    let max_width = node_size.x - padding.x * 2.0;

                    // Create layout for each metadata field
                    let authors_str = metadata.authors.join(", ");
                    let keywords_str = metadata.keywords.join(", ");
                    let fields = vec![
                        ("Title: ", &metadata.title),
                        ("Authors: ", &authors_str),
                        ("Keywords: ", &keywords_str),
                        ("Date: ", &metadata.date),
                    ];

                    // Find the widest label to align all values
                    let label_width = Self::find_widest_label(ctx, fields.clone(), self.zoom);

                    for (label, value) in fields {
                        // Create label galley (bold)
                        let label_galley = ctx.fonts_mut(|f|
                            f.layout_no_wrap(label.to_string(), bold_font_id.clone(), egui::Color32::BLACK)
                        );

                        // Create wrapped value galley with remaining width
                        let value_galley = ctx.fonts_mut(|f|
                            f.layout(
                                value.to_string(),
                                font_id.clone(),
                                egui::Color32::BLACK,
                                max_width - label_width
                            )
                        );

                        metadata_galleys.push((label_galley, value_galley, label_width));
                    }

                    // Calculate total height needed
                    let line_spacing = 4.0 * self.zoom;
                    node_size.y = padding.y;
                    for (_, value_galley, _) in &metadata_galleys {
                        node_size.y += value_galley.size().y + line_spacing;
                    }
                    node_size.y += padding.y;
                } else {
                    // No metadata, just show title with calculated width
                    let title_galley = ctx.fonts_mut(|f| f.layout_no_wrap(node.title.clone(), font_id.clone(), egui::Color32::BLACK));
                    node_size.y = title_galley.size().y + padding.y;
                }
            }

            let node_rect = egui::Rect::from_center_size(pos, node_size);

            // Draw node background
            let fill = if let Some(color) = node.color {
                color
            } else {
                if self.selected_nodes.contains(&node.id) {
                    egui::Color32::from_rgb(180, 220, 255).to_array()
                } else {
                    egui::Color32::LIGHT_BLUE.to_array()
                }
            };
            let stroke = if self.selected_nodes.contains(&node.id) {
                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 100, 255))
            } else {
                egui::Stroke::new(1.0, egui::Color32::BLACK)
            };

            painter.rect(node_rect, 5.0, egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3]), stroke, egui::StrokeKind::Middle);

            // Draw annotation icon if node has annotations
            if !node.annotations.is_empty(){
                let icon_size = egui::vec2(16.0, 16.0) * self.zoom;
                let icon_pos = node_rect.right_top() - egui::vec2(5.0, 5.0) * self.zoom;
                let icon_rect = egui::Rect::from_min_size(icon_pos, icon_size);

                // Draw a small rectangle with a note symbol
                painter.rect(icon_rect, 5.0, egui::Color32::LIGHT_GRAY, egui::Stroke::new(1.0, egui::Color32::BLACK), egui::StrokeKind::Middle);
                painter.text(icon_rect.center(), egui::Align2::CENTER_CENTER, "📝", font_id.clone(), egui::Color32::BLACK);
            }

            if !node.collapsed && !metadata_galleys.is_empty() {
                // Draw metadata
                let mut y_offset = -node_size.y/2.0 + padding.y;
                let x_pos = pos.x - node_size.x/2.0 + padding.x;

                for (label_galley, value_galley, label_width) in metadata_galleys {
                    // Draw label (bold)
                    painter.galley(
                        egui::pos2(x_pos, pos.y + y_offset),
                        label_galley.clone(),
                        egui::Color32::BLACK
                    );

                    // Draw value (normal, wrapped) aligned after the widest label
                    painter.galley(
                        egui::pos2(x_pos + label_width, pos.y + y_offset),
                        value_galley.clone(),
                        egui::Color32::BLACK
                    );

                    y_offset += value_galley.size().y + 4.0 * self.zoom;
                }
            } else {
                // Draw just the title
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    &node.title,
                    font_id.clone(),
                    egui::Color32::BLACK
                );
            }

            // Draw tags
            let mut tag_offset = 0.0;
            for tag in &node.tags {
                let tag_size = egui::vec2(10.0, 10.0) * self.zoom;
                let tag_pos = node_rect.right_bottom() - egui::vec2(5.0 + tag_offset, 5.0) * self.zoom;
                let tag_rect = egui::Rect::from_min_size(tag_pos, tag_size);

                painter.rect_filled(tag_rect, 5.0, egui::Color32::from_rgba_unmultiplied(tag.color[0], tag.color[1], tag.color[2], tag.color[3]));

                tag_offset += 15.0 * self.zoom;
            }
        }
    }

    fn draw_marquee_rect(&mut self, painter: &egui::Painter) {
        if let Some(rect) = self.marquee_rect {
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 150, 250)),
                egui::StrokeKind::Middle
            );
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        let menu_frame = egui::containers::Frame {
            inner_margin: Margin{
                left: 5,
                right: 0,
                top: 0,
                bottom: 0,
            },
            fill: egui::Color32::from_hex("#30313c").unwrap(),
            stroke: Default::default(),
            corner_radius: Default::default(),
            outer_margin: Default::default(),
            shadow: Default::default(),
        };
        egui::TopBottomPanel::top("menu_bar").frame(menu_frame).show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.map = Default::default();
                        self.current_file = None;
                        ui.close_kind(UiKind::Menu);
                    }

                    if ui.button("Open...").clicked() {
                        if let Some(project_dir) = FileDialog::new().pick_folder() {
                            if let Ok(loaded_map) = load_map(project_dir.to_str().unwrap()) {
                                self.current_file = Some(project_dir.to_str().unwrap().to_string());
                                if let Err(e) = save_last_file(&self.current_file.as_ref().unwrap()) {
                                    eprintln!("Failed to save last file: {}", e);
                                }
                                self.map = loaded_map;
                            }
                        }
                        ui.close_kind(UiKind::Menu);
                    }

                    if ui.button("Save As...").clicked() {
                        self.save();
                        ui.close_kind(UiKind::Menu);
                    }

                    if ui.button("Export Project...").clicked() {
                        if let Some(project_dir) = &self.current_file {
                            if let Some(zip_path) = FileDialog::new()
                                .add_filter("ZIP", &["zip"])
                                .save_file()
                            {
                                if let Err(e) = export_project(project_dir, zip_path.to_str().unwrap()) {
                                    eprintln!("Failed to export project: {}", e);
                                }
                            }
                        } else {
                            eprintln!("No project is currently open.");
                        }
                        ui.close_kind(UiKind::Menu);
                    }
                });
            });
        });
    }

    fn show_node_context_menu(&mut self, ctx: &egui::Context) {
        if self.show_node_context_menu {
            let menu_rect = egui::Rect::from_min_size(self.context_menu_pos, egui::vec2(150.0, 100.0));

            egui::Area::new(Id::from("context_menu"))
                .fixed_pos(self.context_menu_pos)
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_hex("#30313c").unwrap())
                        .show(ui, |ui| {
                            ui.set_min_width(150.0);

                            if ui.button("Edit Metadata").clicked() {
                                self.start_editing_metadata();
                                self.show_node_context_menu = false;
                            }

                            if ui.button("View Annotations").clicked() {
                                if let Some(node_id) = self.rightclick_node {
                                    self.annotations_node_id = Some(node_id);
                                    self.show_annotations_panel = true;
                                }
                                self.show_node_context_menu = false;
                            }

                            if ui.button("Add Annotation").clicked() {
                                if let Some(node_id) = self.rightclick_node {
                                    self.annotations_node_id = Some(node_id);
                                    self.edit_annotation = EditableAnnotation::default();
                                    self.show_add_annotation_dialog = true;
                                }
                                self.show_node_context_menu = false;
                            }

                            if ui.button("View Tags").clicked() {
                                if let Some(node_id) = self.rightclick_node {
                                    self.tags_node_id = Some(node_id);
                                    self.show_tags_panel = true;
                                }
                                self.show_node_context_menu = false;
                            }

                            if ui.button("Add Tag").clicked() {
                                if let Some(node_id) = self.rightclick_node {
                                    self.tags_node_id = Some(node_id);
                                    self.edit_tag = EditableTag::default();
                                    self.show_add_tag_dialog = true;
                                }
                                self.show_node_context_menu = false;
                            }

                            if ui.button("Change Color").clicked() {
                                self.node_color_picker_id = Some(self.rightclick_node.unwrap());
                                self.show_node_color_picker = true;

                                if let Some(_node) = self.map.nodes.iter().find(|n| n.id == self.node_color_picker_id.unwrap()) {
                                    self.selected_node_color = egui::Color32::LIGHT_BLUE;
                                } else {
                                    self.selected_node_color = egui::Color32::LIGHT_BLUE;
                                }

                                self.show_node_context_menu = false;
                            }

                            ui.separator();

                            if ui.button("Delete Node").clicked() {
                                if let Some(node_id) = self.rightclick_node {
                                    self.map.remove_node(node_id);
                                    self.dirty = true;
                                }
                                self.show_node_context_menu = false;
                            }
                        });
                });

            // Close menu if clicked elsewhere
            if ctx.input(|i| i.pointer.any_click()) {
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if !menu_rect.contains(pointer_pos) {
                        self.show_node_context_menu = false;
                    }
                }
            }
        }
    }

    fn show_edge_context_menu(&mut self, ctx: &egui::Context) {
        if self.show_edge_context_menu {
            let menu_rect = egui::Rect::from_min_size(self.edge_context_menu_pos, egui::vec2(150.0, 100.0));

            egui::Area::new(Id::from("edge_context_menu"))
                .fixed_pos(self.edge_context_menu_pos)
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_hex("#30313c").unwrap())
                        .show(ui, |ui| {
                            ui.set_min_width(150.0);

                            // Option to change edge type
                            if ui.button("Change Edge Type").clicked() {
                                if let Some(edge_id) = self.rightclick_edge {
                                    if let Some(edge) = self.map.edges.iter_mut().find(|e| e.id == edge_id) {
                                        self.selected_edge_type = edge.edge_type.clone();
                                        self.show_edge_type_menu = true;
                                    }
                                }
                                self.show_edge_context_menu = false;
                            }
                            
                            // Option to view annotations
                            if ui.button("View Annotations").clicked() {
                                self.show_annotations_panel = true;
                                self.show_edge_context_menu = false;
                            }

                            // Option to add annotation
                            if ui.button("Add Annotation").clicked() {
                                if let Some(edge_id) = self.rightclick_edge {
                                    self.edit_annotation = EditableAnnotation::default();
                                    self.show_add_annotation_dialog = true;
                                    self.rightclick_edge = Some(edge_id);
                                }
                                self.show_edge_context_menu = false;
                            }

                            if ui.button("Change Color").clicked() {
                                self.edge_color_picker_id = Some(self.rightclick_edge.unwrap());
                                self.show_edge_color_picker = true;

                                if let Some(_edge) = self.map.edges.iter().find(|n| n.id == self.edge_color_picker_id.unwrap()) {
                                    self.selected_edge_color = egui::Color32::LIGHT_BLUE;
                                } else {
                                    self.selected_edge_color = egui::Color32::LIGHT_BLUE;
                                }

                                self.show_edge_context_menu = false;
                            }

                            ui.separator();


                            // Option to delete edge
                            if ui.button("Delete Edge").clicked() {
                                if let Some(edge_id) = self.rightclick_edge {
                                    self.map.edges.retain(|e| e.id != edge_id);
                                    self.dirty = true;
                                }
                                self.show_edge_context_menu = false;
                            }
                        });
                });

            // Close menu if clicked elsewhere
            if ctx.input(|i| i.pointer.any_click()) {
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if !menu_rect.contains(pointer_pos) {
                        self.show_edge_context_menu = false;
                    }
                }
            }
        }
    }

    fn show_annotations_panel(&mut self, ctx: &egui::Context) {
        if self.show_annotations_panel {
            if let Some(node_id) = self.annotations_node_id {
                let frame = get_popup_frame();
                egui::Window::new("Annotations")
                    .frame(frame)
                    .collapsible(false)
                    .resizable(true)
                    .default_width(400.0)
                    .default_height(600.0)
                    .show(ctx, |ui| {
                        // Find the node and clone the data we need
                        let node_data = self.map.nodes.iter()
                            .find(|n| n.id == node_id)
                            .map(|n| (n.title.clone(), n.annotations.clone()));

                        if let Some((node_title, annotations)) = node_data {
                            ui.heading(format!("Annotations for: {}", node_title));
                            ui.separator();

                            // Add new annotation button
                            if ui.button("➕ Add New Annotation").clicked() {
                                self.edit_annotation = EditableAnnotation::default();
                                self.show_add_annotation_dialog = true;
                            }

                            ui.separator();

                            // Show existing annotations
                            self.show_existing_annotations(annotations, ui, node_id);

                            ui.horizontal(|ui| {
                                if ui.button("Close").clicked() {
                                    self.show_annotations_panel = false;
                                    self.annotations_node_id = None;
                                }
                            });
                        } else {
                            ui.label("Node not found");
                        }
                    });
            }
            if let Some(edge_id) = self.rightclick_edge {
                egui::Window::new("Annotations")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(400.0)
                    .default_height(600.0)
                    .show(ctx, |ui| {
                        // Find the edge and clone the annotations
                        let edge_data = self.map.edges.iter()
                            .find(|e| e.id == edge_id)
                            .map(|e| e.annotations.clone());

                        if let Some(annotations) = edge_data {
                            ui.heading("Annotations for Edge");
                            ui.separator();

                            // Add new annotation button
                            if ui.button("➕ Add New Annotation").clicked() {
                                self.edit_annotation = EditableAnnotation::default();
                                self.show_add_annotation_dialog = true;
                            }

                            ui.separator();

                            // Show existing annotations
                            self.show_existing_annotations(annotations, ui, edge_id);

                            ui.horizontal(|ui| {
                                if ui.button("Close").clicked() {
                                    self.show_annotations_panel = false;
                                    self.rightclick_edge = None;
                                }
                            });
                        } else {
                            ui.label("Edge not found");
                        }
                    });
            }
        }
    }

    fn show_tags_panel(&mut self, ctx: &egui::Context) {
        if self.show_tags_panel {
            let frame = get_popup_frame();
            if let Some(node_id) = self.tags_node_id {
                egui::Window::new("Tags")
                    .frame(frame)
                    .collapsible(false)
                    .resizable(true)
                    .default_width(400.0)
                    .default_height(600.0)
                    .show(ctx, |ui| {
                        // Find the node and clone the data we need
                        let node_data = self.map.nodes.iter()
                            .find(|n| n.id == node_id)
                            .map(|n| (n.title.clone(), n.tags.clone()));

                        if let Some((node_title, tags)) = node_data {
                            ui.heading(format!("Tags for: {}", node_title));
                            ui.separator();

                            // Add new annotation button
                            if ui.button("➕ Add New Tag").clicked() {
                                self.edit_tag = EditableTag::default();
                                self.show_add_tag_dialog = true;
                            }

                            ui.separator();

                            // Show existing annotations
                            self.show_existing_tags(tags, ui, node_id);

                            ui.horizontal(|ui| {
                                if ui.button("Close").clicked() {
                                    self.show_tags_panel = false;
                                    self.tags_node_id = None;
                                }
                            });
                        } else {
                            ui.label("Node not found");
                        }
                    });
            }
        }
    }

    fn show_annotation_card(&mut self, ui: &mut egui::Ui, annotation: &Annotation, id: Uuid) {
        let frame = egui::Frame::new()
            .inner_margin(egui::Margin::same(8))
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, annotation.annotation_type.color()));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                if !annotation.title.is_empty() {
                    ui.label(egui::RichText::new(&annotation.title).strong().color(egui::Color32::WHITE));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                        // Remove annotation
                        if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == id) {
                            node.annotations.retain(|a| a.id != annotation.id);
                            self.dirty = true;
                        }
                        if let Some(edge) = self.map.edges.iter_mut().find(|n| n.id == id) {
                            edge.annotations.retain(|a| a.id != annotation.id);
                            self.dirty = true;
                        }
                    }

                    if ui.small_button("✏").on_hover_text("Edit").clicked() {
                        self.start_editing_annotation(annotation.clone());
                    }

                    if let Some(page) = annotation.page_number {
                        ui.label(egui::RichText::new(format!("p.{}",page)).strong().color(egui::Color32::DARK_GRAY));
                    }
                });
            });

            if !annotation.content.is_empty() {
                ui.label(egui::RichText::new(&annotation.content).strong().color(egui::Color32::LIGHT_GRAY));
            }

            ui.label(egui::RichText::new(&annotation.created_at).small().color(egui::Color32::DARK_GRAY));
        });
    }

    fn show_tag(&mut self, ui: &mut egui::Ui, tag: &Tag, id: Uuid) {
        let frame = egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(tag.color[0], tag.color[1], tag.color[2], tag.color[3]))
            .inner_margin(egui::Margin::same(8))
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(tag.color[0], tag.color[1], tag.color[2], tag.color[3])));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                if !tag.name.is_empty() {
                    ui.label(egui::RichText::new(&tag.name).strong().color(egui::Color32::WHITE));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                        // Remove annotation
                        if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == id) {
                            node.tags.retain(|a| a.id != tag.id);
                            self.dirty = true;
                        }
                    }

                    if ui.small_button("✏").on_hover_text("Edit").clicked() {
                        self.start_editing_tag(tag.clone());
                    }
                });
            });
        });
    }

    fn show_annotation_dialog(&mut self, ctx: &egui::Context) {
        let is_editing = self.show_edit_annotation_dialog;
        let show_dialog = self.show_add_annotation_dialog || is_editing;
        if show_dialog {
            let title = if is_editing { "Edit Annotation" } else { "Add New Annotation" };
            let frame = get_popup_frame();
            egui::Window::new(title)
                .frame(frame)
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.label("Create a new annotation for this node:");
                    ui.separator();

                    egui::Grid::new("annotation_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_label("")
                                .selected_text(self.edit_annotation.annotation_type.name())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::TextNote, "Text Note");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Question, "Question");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Summary, "Summary");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Quote, "Quote");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Todo, "Todo");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Idea, "Idea");
                                    ui.selectable_value(&mut self.edit_annotation.annotation_type, AnnotationType::Warning, "Warning");
                                });
                            ui.end_row();

                            ui.label("Title:");
                            ui.text_edit_singleline(&mut self.edit_annotation.title);
                            ui.end_row();

                            ui.label("Page (optional):");
                            ui.text_edit_singleline(&mut self.edit_annotation.page_number);
                            ui.end_row();

                            ui.label("Content:");
                            ui.text_edit_multiline(&mut self.edit_annotation.content);
                            ui.end_row();
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let save_text = if is_editing { "Update" } else { "Add" };
                        if ui.button(save_text).clicked() {
                            self.save_annotation();
                            if is_editing {
                                self.show_edit_annotation_dialog = false;
                            } else {
                                self.show_add_annotation_dialog = false;
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            if is_editing {
                                self.show_edit_annotation_dialog = false;
                            } else {
                                self.show_add_annotation_dialog = false;
                            }
                        }
                    });
                });
        }
    }

    fn show_tag_dialog(&mut self, ctx: &egui::Context) {
        let is_editing = self.show_edit_tag_dialog;
        let show_dialog = self.show_add_tag_dialog || is_editing;

        if show_dialog {
            let title = if is_editing { "Edit Tag" } else { "Add New Tag" };
            let frame = get_popup_frame();
            egui::Window::new(title)
                .frame(frame)
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.label("Create a new tag for this node:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.edit_tag.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = self.edit_tag.color;
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            self.edit_tag.color = color;
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let save_text = if is_editing { "Update" } else { "Add" };
                        if ui.button(save_text).clicked() {
                            self.save_tag();
                            if is_editing {
                                self.show_edit_tag_dialog = false;
                            } else {
                                self.show_add_tag_dialog = false;
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            if is_editing {
                                self.show_edit_tag_dialog = false;
                            } else {
                                self.show_add_tag_dialog = false;
                            }
                        }
                    });
                });
        }
    }

    fn start_editing_annotation(&mut self, annotation: Annotation) {
        self.edit_annotation_id = Some(annotation.id);
        self.edit_annotation = EditableAnnotation {
            title: annotation.title,
            content: annotation.content,
            annotation_type: annotation.annotation_type,
            page_number: annotation.page_number.map_or(String::new(), |p| p.to_string()),
        };
        self.show_edit_annotation_dialog = true;
    }

    fn start_editing_tag(&mut self, tag: Tag) {
        self.edit_tag_id = Some(tag.id);
        self.edit_tag = EditableTag {
            name: tag.name,
            color: egui::Color32::from_rgba_unmultiplied(tag.color[0], tag.color[1], tag.color[2], tag.color[3]),
        };
        self.show_edit_tag_dialog = true;
    }

    fn save_annotation(&mut self) {
        if let Some(node_id) = self.annotations_node_id {
            if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == node_id) {
                let page_number = if self.edit_annotation.page_number.trim().is_empty() {
                    None
                } else {
                    self.edit_annotation.page_number.parse().ok()
                };

                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

                if let Some(edit_id) = self.edit_annotation_id {
                    // Editing existing annotation
                    if let Some(annotation) = node.annotations.iter_mut().find(|a| a.id == edit_id) {
                        annotation.title = self.edit_annotation.title.clone();
                        annotation.content = self.edit_annotation.content.clone();
                        annotation.annotation_type = self.edit_annotation.annotation_type.clone();
                        annotation.page_number = page_number;
                    }
                    self.edit_annotation_id = None;
                } else {
                    // Adding new annotation
                    let annotation = Annotation {
                        id: Uuid::new_v4(),
                        annotation_type: self.edit_annotation.annotation_type.clone(),
                        title: self.edit_annotation.title.clone(),
                        content: self.edit_annotation.content.clone(),
                        created_at: now,
                        page_number,
                    };
                    node.annotations.push(annotation);
                }

                self.dirty = true;
            }
        }

        if let Some(edge_id) = self.rightclick_edge {
            if let Some(edge) = self.map.edges.iter_mut().find(|e| e.id == edge_id) {
                let page_number = if self.edit_annotation.page_number.trim().is_empty() {
                    None
                } else {
                    self.edit_annotation.page_number.parse().ok()
                };

                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

                if let Some(edit_id) = self.edit_annotation_id {
                    // Editing existing annotation
                    if let Some(annotation) = edge.annotations.iter_mut().find(|a| a.id == edit_id) {
                        annotation.title = self.edit_annotation.title.clone();
                        annotation.content = self.edit_annotation.content.clone();
                        annotation.annotation_type = self.edit_annotation.annotation_type.clone();
                        annotation.page_number = page_number;
                    }
                    self.edit_annotation_id = None;
                }else {
                    // Add new annotation
                    let annotation = Annotation {
                        id: Uuid::new_v4(),
                        annotation_type: self.edit_annotation.annotation_type.clone(),
                        title: self.edit_annotation.title.clone(),
                        content: self.edit_annotation.content.clone(),
                        created_at: now,
                        page_number,
                    };
                    edge.annotations.push(annotation);
                }
                self.dirty = true;
            }
        }
    }

    fn save_tag(&mut self) {
        if let Some(node_id) = self.tags_node_id {
            if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == node_id) {
               if let Some(edit_id) = self.edit_tag_id {
                    // Editing existing tags
                    if let Some(tag) = node.tags.iter_mut().find(|a| a.id == edit_id) {
                        tag.name = self.edit_tag.name.clone();
                        tag.color = self.edit_tag.color.to_array();
                    }
                    self.edit_tag_id = None;
                } else {
                    // Adding new annotation
                    let tag = Tag {
                        id: Uuid::new_v4(),
                        name: self.edit_tag.name.clone(),
                        color: self.edit_tag.color.to_array(),
                    };
                    node.tags.push(tag);
                }

                self.dirty = true;
            }
        }
    }

    fn start_editing_metadata(&mut self) {
        if let Some(node_id) = self.rightclick_node {
            self.edit_node_id = Some(node_id);

            // Load existing metadata into editable form
            if let Some(node) = self.map.nodes.iter().find(|n| n.id == node_id) {
                if let Some(metadata) = &node.metadata {
                    self.edit_metadata = EditableMetadata {
                        title: metadata.title.clone(),
                        authors: metadata.authors.join(", "),
                        keywords: metadata.keywords.join(", "),
                        date: metadata.date.clone(),
                    };
                } else {
                    // Initialize with node title if no metadata exists
                    self.edit_metadata = EditableMetadata {
                        title: node.title.clone(),
                        authors: String::new(),
                        keywords: String::new(),
                        date: String::new(),
                    };
                }
            }

            self.show_edit_dialog = true;
        }
    }

    fn show_edit_metadata_dialog(&mut self, ctx: &egui::Context) {
        if self.show_edit_dialog {
            egui::Window::new("Edit Node Metadata")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.label("Edit the metadata for this node:");
                    ui.separator();

                    egui::Grid::new("edit_metadata_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Title:");
                            ui.text_edit_singleline(&mut self.edit_metadata.title);
                            ui.end_row();

                            ui.label("Authors:");
                            ui.text_edit_singleline(&mut self.edit_metadata.authors);
                            ui.end_row();

                            ui.label("Keywords:");
                            ui.text_edit_singleline(&mut self.edit_metadata.keywords);
                            ui.end_row();

                            ui.label("Date:");
                            ui.text_edit_singleline(&mut self.edit_metadata.date);
                            ui.end_row();
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_edited_metadata();
                            self.show_edit_dialog = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_edit_dialog = false;
                        }
                    });
                });
        }
    }

    fn save_edited_metadata(&mut self) {
        if let Some(node_id) = self.edit_node_id {
            if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == node_id) {
                // Update node title
                node.title = self.edit_metadata.title.clone();

                // Parse and update metadata
                let authors: Vec<String> = self.edit_metadata.authors
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let keywords: Vec<String> = self.edit_metadata.keywords
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Create or update metadata
                node.metadata = Some(Metadata {
                    title: self.edit_metadata.title.clone(),
                    authors,
                    keywords,
                    date: self.edit_metadata.date.clone(),
                    path: "".to_string(),
                });

                self.dirty = true;
            }
        }
        self.edit_node_id = None;
    }

    fn finalize_edge(&mut self) {
        if let (Some(from), Some(to)) = (self.pending_edge_from, self.pending_edge_to) {
            self.map.add_edge(from, to);
            self.map.edges.last_mut().unwrap().edge_type = self.selected_edge_type.clone();
            self.dirty = true;
        }
        self.show_edge_type_menu = false;
        self.pending_edge_from = None;
        self.pending_edge_to = None;
    }

    fn show_edge_type_menu(&mut self, ctx: &egui::Context) {
        if self.show_edge_type_menu {
            if let Some(menu_pos) = self.edge_type_menu_pos {
                egui::Window::new("Select Edge Type")
                    .collapsible(false)
                    .resizable(false)
                    .fixed_pos(menu_pos) // Use the stored position
                    .show(ctx, |ui| {
                        ui.label("Select edge type:");

                        if ui.button("Normal").clicked() {
                            self.selected_edge_type = EdgeType::Normal;
                            if let Some(edge_id) = self.rightclick_edge {
                                if let Some(edge) = self.map.edges.iter_mut().find(|e| e.id == edge_id) {
                                    edge.edge_type = self.selected_edge_type.clone();
                                    self.dirty = true;
                                    self.rightclick_edge = None;
                                }
                            } else {
                                self.finalize_edge();
                            }
                            self.show_edge_type_menu = false;
                        }

                        if ui.button("References").clicked() {
                            self.selected_edge_type = EdgeType::References;
                            if let Some(edge_id) = self.rightclick_edge {
                                if let Some(edge) = self.map.edges.iter_mut().find(|e| e.id == edge_id) {
                                    edge.edge_type = self.selected_edge_type.clone();
                                    self.dirty = true;
                                    self.rightclick_edge = None;
                                }
                            } else {
                                self.finalize_edge();
                            }
                            self.show_edge_type_menu = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_edge_type_menu = false;
                            self.pending_edge_from = None;
                            self.pending_edge_to = None;
                            self.edge_type_menu_pos = None; // Reset the position
                        }
                    });
            }
        }
    }

    fn show_create_project_promt(&mut self, ctx: &egui::Context) {
        if self.show_create_project_prompt {
            egui::Window::new("Create Project First")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("You need to create or open a project before adding PDFs.");
                    if ui.button("Create New Project").clicked() {
                        self.save(); // This will create a new project and add the pending PDF
                        self.show_create_project_prompt = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_create_project_prompt = false;
                        self.pending_pdf_path = None;
                    }
                });
        }
    }

    fn show_node_color_picker(&mut self, ctx: &egui::Context) {
        if self.show_node_color_picker {
            if let Some(node_id) = self.node_color_picker_id {
                egui::Window::new("Change Node Color")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("Select a color for this node:");

                        // Color editor
                        let mut color = self.selected_node_color;
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            self.selected_node_color = color;
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Apply").clicked() {
                                if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == node_id) {
                                    node.color = Some(self.selected_node_color.to_array());
                                    self.dirty = true;
                                }
                                self.show_node_color_picker = false;
                                self.node_color_picker_id = None;
                            }
                            if ui.button("Reset").clicked() {
                                if let Some(node) = self.map.nodes.iter_mut().find(|e| e.id == node_id) {
                                    node.color = None;
                                    self.dirty = true;
                                }
                                self.show_node_color_picker = false;
                                self.edge_color_picker_id = None;
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_node_color_picker = false;
                                self.node_color_picker_id = None;
                            }
                        });
                    });
            }
        }
    }

    fn show_edge_color_picker(&mut self, ctx: &egui::Context) {
        if self.show_edge_color_picker {
            if let Some(edge_id) = self.edge_color_picker_id {
                egui::Window::new("Change Edge Color")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("Select a color for this edge:");

                        // Color editor
                        let mut color = self.selected_edge_color;
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            self.selected_edge_color = color;
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Apply").clicked() {
                                if let Some(edge) = self.map.edges.iter_mut().find(|n| n.id == edge_id) {
                                    edge.color = Some(self.selected_edge_color.to_array());
                                    self.dirty = true;
                                }
                                self.show_edge_color_picker = false;
                                self.edge_color_picker_id = None;
                            }
                            if ui.button("Reset").clicked() {
                                if let Some(edge) = self.map.edges.iter_mut().find(|e| e.id == edge_id) {
                                    edge.color = None;
                                    self.dirty = true;
                                }
                                self.show_edge_color_picker = false;
                                self.edge_color_picker_id = None;
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_edge_color_picker = false;
                                self.edge_color_picker_id = None;
                            }
                        });
                    });
            }
        }
    }

    fn save(&mut self) {
        if let Some(project_dir) = FileDialog::new().pick_folder() {
            self.current_file = Some(project_dir.to_str().unwrap().to_string());
            if let Err(e) = save_last_file(&self.current_file.as_ref().unwrap()) {
                eprintln!("Failed to save last file: {}", e);
            }

            // Save the map
            if let Err(e) = save_map(&self.map, &project_dir.to_str().unwrap()) {
                eprintln!("Failed to save map: {}", e);
            }

            // Handle pending PDF
            if let Some(pdf_path) = self.pending_pdf_path.take() {
                let pdfs_dir = std::path::Path::new(&project_dir).join("pdfs");
                if !pdfs_dir.exists() {
                    std::fs::create_dir_all(&pdfs_dir).unwrap();
                }
                let file_name = std::path::Path::new(&pdf_path).file_name().unwrap().to_str().unwrap();
                let dest_path = pdfs_dir.join(file_name);
                std::fs::copy(&pdf_path, &dest_path).unwrap();
                self.map.add_pdf_node(&format!("{}/{}",pdfs_dir.to_str().unwrap(), file_name), 0.0, 0.0).unwrap();
                self.dirty = true;
            }
        }
    }

    fn start_edge(&mut self, ctx: &egui::Context, canvas_pos: Pos2){
        for node in &self.map.nodes {
            let node_rect = get_node_rect(ctx, node, self.zoom);
            if node_rect.contains(canvas_pos) {
                self.connecting_from = Some(node.id);
                break;
            }
        }
    }

    fn stop_edge(&mut self, ctx: &egui::Context, canvas_pos: Pos2, start_id: Uuid) {
        let mut found_to_id = None;
        for node in &self.map.nodes {
            let node_rect = get_node_rect(ctx, node, self.zoom);
            if node_rect.contains(canvas_pos) && node.id != start_id {
                found_to_id = Some(node.id);
                break;
            }
        }

        if let Some(to_id) = found_to_id {
            self.pending_edge_from = Some(start_id);
            self.pending_edge_to = Some(to_id);
            self.show_edge_type_menu = true;
            self.edge_type_menu_pos = ctx.input(|i| i.pointer.hover_pos());
        }
    }

    fn find_widest_label(ctx: &egui::Context, fields: Vec<(&str, &String)>, zoom: f32) -> f32 {
        ctx.fonts_mut(|f| {
            let bold_font_id = egui::FontId::monospace(14.0 * zoom);
            fields.iter().map(|(label, _)| {
                f.layout_no_wrap(label.to_string(), bold_font_id.clone(), egui::Color32::BLACK)
                    .size().x
            }).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
        })
    }

    fn show_existing_annotations(&mut self, annotations: Vec<Annotation>, ui: &mut egui::Ui, node_id: Uuid) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for annotation in &annotations {
                self.show_annotation_card(ui, annotation, node_id);
                ui.separator();
            }

            if annotations.is_empty() {
                ui.label("No annotations yet. Click 'Add New Annotation' to get started!");
            }
        });
    }

    fn show_existing_tags(&mut self, tags: Vec<Tag>, ui: &mut egui::Ui, node_id: Uuid) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for tag in &tags {
                self.show_tag(ui, tag, node_id);
                ui.separator();
            }

            if tags.is_empty() {
                ui.label("No tags yet. Click 'Add New Tag' to get started!");
            }
        });
    }
}

impl Default for MindMapApp {
    fn default() -> Self {
        let map = MindMap::default();
        let mut app = Self {
            map,
            dragging_node: None,
            connecting_from: None,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            marquee_start: None,
            marquee_rect: None,
            pan: egui::vec2(0.0, 0.0),
            zoom: 1.0,
            current_file: None,
            last_save: std::time::Instant::now(),
            dirty: false,
            rightclick_node: None,
            show_node_context_menu: false,
            context_menu_pos: egui::pos2(0.0, 0.0),
            show_edit_dialog: false,
            edit_node_id: None,
            edit_metadata: EditableMetadata::default(),
            show_annotations_panel: false,
            show_add_annotation_dialog: false,
            show_edit_annotation_dialog: false,
            edit_annotation_id: None,
            edit_annotation: EditableAnnotation::default(),
            annotations_node_id: None,
            pending_edge_from: None,
            pending_edge_to: None,
            show_edge_type_menu: false,
            selected_edge_type: EdgeType::Normal,
            edge_type_menu_pos: None,
            rightclick_edge: None,
            show_edge_context_menu: false,
            edge_context_menu_pos: egui::pos2(0.0, 0.0),
            show_create_project_prompt: false,
            pending_pdf_path: None,
            show_node_color_picker: false,
            node_color_picker_id: None,
            selected_node_color: egui::Color32::WHITE,
            show_edge_color_picker: false,
            edge_color_picker_id: None,
            selected_edge_color: egui::Color32::WHITE,
            show_tags_panel: false,
            show_add_tag_dialog: false,
            show_edit_tag_dialog: false,
            edit_tag_id: None,
            edit_tag: EditableTag::default(),
            tags_node_id: None,
        };
        // Load last file if it exists
        if let Ok(last_file) = load_last_file() {
            app.current_file = Some(last_file);
            if let Ok(loaded_map) = load_map(&app.current_file.as_ref().unwrap()) {
                app.map = loaded_map;
            }
        }
        app
    }
}

impl eframe::App for MindMapApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.visuals.extreme_bg_color = egui::Color32::from_hex("#22222a").unwrap()
        });

        // autosave
        if self.dirty && self.last_save.elapsed().as_secs() > 5 {
            if let Some(path) = &self.current_file {
                let _ = save_map(&self.map, path);
                self.last_save = std::time::Instant::now();
                self.dirty = false;
            }
        }

        // menu bar
        self.menu_bar(ctx);

        // Show context menu if active
        self.show_node_context_menu(ctx);

        // Show edit dialog if active
        self.show_edit_metadata_dialog(ctx);

        // Show annotation dialogs
        self.show_annotations_panel(ctx);
        self.show_annotation_dialog(ctx);

        // Show annotation dialogs
        self.show_tags_panel(ctx);
        self.show_tag_dialog(ctx);

        // Show edge context menu if active
        self.show_edge_context_menu(ctx);

        // Edge type selection menu
        self.show_edge_type_menu(ctx);

        // Show create project prompt if needed
        self.show_create_project_promt(ctx);

        // Show node color picker if active
        self.show_node_color_picker(ctx);

        // Show edge color picker if active
        self.show_edge_color_picker(ctx);

        // main panel
        self.main_view(ctx);
    }
}

fn get_node_rect(ctx: &egui::Context, node: &Node, zoom: f32) -> egui::Rect {
    let font_id = egui::FontId::proportional(14.0 * zoom);
    let padding = egui::vec2(16.0, 12.0) * zoom;

    // Calculate node size based on collapse state
    let node_size = if node.collapsed {
        // When collapsed, size based on title
        let title_galley = ctx.fonts_mut(|f| f.layout_no_wrap(node.title.clone(), font_id.clone(), egui::Color32::BLACK));
        let text_size = title_galley.size();
        egui::vec2(text_size.x + padding.x * 2.0, text_size.y + padding.y)
    } else {
        // When expanded, use minimum width or title width, whichever is larger
        let title_galley = ctx.fonts_mut(|f| f.layout_no_wrap(node.title.clone(), font_id.clone(), egui::Color32::BLACK));
        let title_width = title_galley.size().x + padding.x * 2.0;
        let min_width = 300.0 * zoom;
        let node_width = title_width.max(min_width);

        // Calculate height based on metadata content or just title if no metadata
        let mut height = title_galley.size().y + padding.y * 2.0;

        if let Some(metadata) = &node.metadata {
            let max_width = node_width - padding.x * 2.0;
            let line_spacing = 4.0 * zoom;

            // Calculate space for metadata fields
            let authors_str = metadata.authors.join(", ");
            let keywords_str = metadata.keywords.join(", ");
            let fields = vec![
                ("Title: ", &metadata.title),
                ("Authors: ", &authors_str),
                ("Keywords: ", &keywords_str),
                ("Date: ", &metadata.date),
            ];

            let label_width = MindMapApp::find_widest_label(ctx, fields.clone(), zoom);

            // Calculate height based on wrapped text
            height = padding.y;
            for (_, value) in &fields {
                let value_galley = ctx.fonts_mut(|f|
                    f.layout(
                        value.to_string(),
                        font_id.clone(),
                        egui::Color32::BLACK,
                        max_width - label_width,
                    )
                );
                height += value_galley.size().y + line_spacing;
            }
            height += padding.y;
        }

        egui::vec2(node_width, height)
    };

    egui::Rect::from_center_size(egui::pos2(node.x, node.y), node_size)
}

fn point_line_distance(a: Pos2, b: Pos2, p: Pos2) -> f32 {
    let ap = p - a;
    let ab = b - a;
    let ab_len = ab.length();
    if ab_len < f32::EPSILON {
        return ap.length();
    }
    let t = (ap.dot(ab) / ab_len.powi(2)).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length()
}

fn get_popup_frame() -> egui::Frame {
    let frame = egui::Frame{
        inner_margin: Default::default(),
        fill: egui::Color32::from_hex("#2b2c36").unwrap(),
        stroke: Default::default(),
        corner_radius: Default::default(),
        outer_margin: Default::default(),
        shadow: egui::Shadow{
            offset: [5,5],
            blur: 20,
            spread: 0,
            color: egui::Color32::BLACK,
        },
    };
    frame
}

