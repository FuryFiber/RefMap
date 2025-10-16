use egui::{Id, UiKind};
use rfd::FileDialog;
use uuid::Uuid;
use crate::core::map::{EdgeType, Node};
use crate::core::MindMap;
use crate::core::storage::{load_map, save_map};
use crate::core::pdfparser::Metadata;


pub struct MindMapApp {
    map: MindMap,                       // the mind map data

    // Interaction state
    dragging_node: Option<Uuid>,        // currently dragged node
    connecting_from: Option<Uuid>,      // node from which a connection is being made
    selected_nodes: Vec<Uuid>,          // currently selected nodes
    selected_edges: Vec<Uuid>,          // currently selected edges

    // For marquee selection
    marquee_start: Option<egui::Pos2>,  // where the marquee drag started
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
    show_node_context_menu: bool,            // whether context menu should be visible
    context_menu_pos: egui::Pos2,       // position to show context menu

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
    edge_type_menu_pos: Option<egui::Pos2>, // position to show edge type menu

    // Right-click context for edges
    rightclick_edge: Option<Uuid>,      // Edge that was right-clicked
    show_edge_context_menu: bool,      // Whether to show the edge context menu
    edge_context_menu_pos: egui::Pos2, // Position to show the edge context menu
}

// Helper struct for editing metadata
#[derive(Debug, Clone, Default)]
struct EditableMetadata {
    title: String,
    authors: String,
    keywords: String,
    date: String,
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
    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.map = Default::default();
                        ui.close_kind(UiKind::Menu);
                    }

                    if ui.button("Open...").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Mind Map", &["json"])
                            .pick_file()
                        {
                            if let Ok(loaded_map) = load_map(path.to_str().unwrap()) {
                                self.current_file = Some(path.to_str().unwrap().to_string());
                                self.map = loaded_map;
                            }
                        }
                        ui.close_kind(UiKind::Menu);
                    }

                    if ui.button("Save As...").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Mind Map", &["json"])
                            .save_file()
                        {
                            self.current_file = Some(path.to_str().unwrap().to_string());
                            let _ = save_map(&self.map, path.to_str().unwrap());
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
                        .show(ui, |ui| {
                            ui.set_min_width(150.0);

                            if ui.button("Edit Metadata").clicked() {
                                self.start_editing();
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

    // Show annotations panel
    fn show_annotations_panel(&mut self, ctx: &egui::Context) {
        if self.show_annotations_panel {
            if let Some(node_id) = self.annotations_node_id {
                egui::Window::new("Annotations")
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
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for annotation in &annotations {
                                    self.show_annotation_card(ui, annotation, node_id);
                                    ui.separator();
                                }

                                if annotations.is_empty() {
                                    ui.label("No annotations yet. Click 'Add New Annotation' to get started!");
                                }
                            });

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
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for annotation in &annotations {
                                    self.show_annotation_card(ui, annotation, edge_id);
                                    ui.separator();
                                }

                                if annotations.is_empty() {
                                    ui.label("No annotations yet. Click 'Add New Annotation' to get started!");
                                }
                            });

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

    // Show individual annotation card
    fn show_annotation_card(&mut self, ui: &mut egui::Ui, annotation: &Annotation, node_id: Uuid) {
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
                        if let Some(node) = self.map.nodes.iter_mut().find(|n| n.id == node_id) {
                            node.annotations.retain(|a| a.id != annotation.id);
                            self.dirty = true;
                        }
                    }

                    if ui.small_button("✏").on_hover_text("Edit").clicked() {
                        self.start_editing_annotation(annotation.clone());
                    }

                    if let Some(page) = annotation.page_number {
                        ui.label(format!("p.{}", page));
                    }
                });
            });

            if !annotation.content.is_empty() {
                ui.label(&annotation.content);
            }

            ui.label(egui::RichText::new(&annotation.created_at).small().color(egui::Color32::DARK_GRAY));
        });
    }

    // Show add/edit annotation dialog
    fn show_annotation_dialog(&mut self, ctx: &egui::Context) {
        let is_editing = self.show_edit_annotation_dialog;
        let show_dialog = self.show_add_annotation_dialog || is_editing;

        if show_dialog {
            let title = if is_editing { "Edit Annotation" } else { "Add New Annotation" };

            egui::Window::new(title)
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

                let annotation = Annotation {
                    id: Uuid::new_v4(),
                    annotation_type: self.edit_annotation.annotation_type.clone(),
                    title: self.edit_annotation.title.clone(),
                    content: self.edit_annotation.content.clone(),
                    created_at: now,
                    page_number,
                };
                edge.annotations.push(annotation);
                self.dirty = true;
            }
        }
    }

    fn start_editing(&mut self) {
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

    fn show_edit_dialog(&mut self, ctx: &egui::Context) {
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

    fn show_edge_context_menu(&mut self, ctx: &egui::Context) {
        if self.show_edge_context_menu {
            let menu_rect = egui::Rect::from_min_size(self.edge_context_menu_pos, egui::vec2(150.0, 100.0));

            egui::Area::new(Id::from("edge_context_menu"))
                .fixed_pos(self.edge_context_menu_pos)
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
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

                            // Option to add annotation
                            if ui.button("Add Annotation").clicked() {
                                if let Some(edge_id) = self.rightclick_edge {
                                    self.edit_annotation = EditableAnnotation::default();
                                    self.show_add_annotation_dialog = true;
                                    self.rightclick_edge = Some(edge_id);
                                }
                                self.show_edge_context_menu = false;
                            }

                            // Option to view annotations
                            if ui.button("View Annotations").clicked() {
                                self.show_annotations_panel = true;
                                self.show_edge_context_menu = false;
                            }

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
}

impl Default for MindMapApp {
    fn default() -> Self {
        let map = MindMap::default();
        Self {
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
            // Initialize new fields
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
        }
    }
}

impl eframe::App for MindMapApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
        self.show_edit_dialog(ctx);

        // Show annotation dialogs
        self.show_annotations_panel(ctx);
        self.show_annotation_dialog(ctx);

        // Show edge context menu if active
        self.show_edge_context_menu(ctx);

        // Edge type selection menu
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

        // main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RefMap");

            // Create a canvas region that captures click + drag
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                egui::Sense::click_and_drag(),
            );

            let rect = response.rect;

            // --- Handle panning with middle mouse ---
            if response.dragged_by(egui::PointerButton::Middle) {
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

            // --- Handle key input for deletion ---
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

            // unselect all on Escape
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.selected_nodes = Vec::new();
                self.selected_edges= Vec::new();
            }


            // manual save
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
                if let Some(path) = self.current_file.clone(){
                    let _ = save_map(&self.map, &path);
                }
                else {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Mind Map", &["json"])
                        .save_file()
                    {
                        self.current_file = Some(path.to_str().unwrap().to_string());
                        let _ = save_map(&self.map, path.to_str().unwrap());
                    }
                }
            }

            // --- Handle mouse actions ---
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos =
                    (pointer_pos - rect.min.to_vec2() - self.pan) / self.zoom;

                // Left click selection / creation
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
                                    // Open PDF with default system viewer
                                    if let Err(e) = opener::open(file_path) {
                                        eprintln!("Failed to open PDF: {}", e);
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
                                self.selected_nodes.push(node.id);
                                clicked_any = true;
                                break;
                            }
                        }
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
                                    self.selected_edges.push(edge.id);
                                    clicked_any = true;
                                    break;
                                }
                            }
                        }
                        if !clicked_any {
                            // Shift + left click → open PDF file picker
                            if let Some(path) = FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .pick_file()
                            {
                                self.map.add_pdf_node(path.to_str().unwrap(), canvas_pos.x, canvas_pos.y).expect("TODO: panic message");
                                self.dirty = true;
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

                // Drag existing node with left mouse
                if response.dragged_by(egui::PointerButton::Primary) {
                    if ctx.input(|i| i.modifiers.ctrl) {
                        // start connection from node under cursor
                        for node in &self.map.nodes {
                            let node_rect = get_node_rect(ctx, node, self.zoom);
                            if node_rect.contains(canvas_pos) {
                                self.connecting_from = Some(node.id);
                                break;
                            }
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

                // Right click handling - Updated to show context menu
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

                // --- Right button: create connections ---
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    // start connection from node under cursor
                    for node in &self.map.nodes {
                        let node_rect = get_node_rect(ctx, node, self.zoom);
                        if node_rect.contains(canvas_pos) {
                            self.connecting_from = Some(node.id);
                            break;
                        }
                    }
                }

                if response.drag_stopped_by(egui::PointerButton::Secondary) {
                    if let Some(start_id) = self.connecting_from.take() {
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
                }
            }

            // --- Draw edges ---
            for edge in &self.map.edges {
                let from = self.map.nodes.iter().find(|n| n.id == edge.from);
                let to = self.map.nodes.iter().find(|n| n.id == edge.to);
                if let (Some(f), Some(t)) = (from, to) {
                    let p1 = egui::pos2(f.x, f.y) * self.zoom + self.pan + rect.min.to_vec2();
                    let p2 = egui::pos2(t.x, t.y) * self.zoom + self.pan + rect.min.to_vec2();
                    let color = if self.selected_edges.contains(&edge.id) {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::DARK_GRAY
                    };
                    let width = 2.0;

                    // Draw line
                    painter.line_segment([p1, p2], egui::Stroke::new(width, color));

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

                        let width = 3.0; // Stroke width
                        painter.line_segment([p2_arrow, p3_arrow], egui::Stroke::new(width, color)); // Line from tip to one side
                        painter.line_segment([p2_arrow, p4_arrow], egui::Stroke::new(width, color)); // Line from tip to other side
                    }
                }
            }

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

            // --- Draw temporary connection line (while dragging) ---
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

            // --- Draw nodes ---
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
                        let label_width = ctx.fonts_mut(|f| {
                            fields.iter().map(|(label, _)| {
                                f.layout_no_wrap(label.to_string(), bold_font_id.clone(), egui::Color32::BLACK)
                                    .size().x
                            }).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
                        });

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
                let fill = if self.selected_nodes.contains(&node.id) {
                    egui::Color32::from_rgb(180, 220, 255)
                } else {
                    egui::Color32::LIGHT_BLUE
                };
                let stroke = if self.selected_nodes.contains(&node.id) {
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 100, 255))
                } else {
                    egui::Stroke::new(1.0, egui::Color32::BLACK)
                };

                painter.rect(node_rect, 5.0, fill, stroke, egui::StrokeKind::Middle);

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
                        egui::Color32::BLACK,
                    );
                }
            }


            // --- Draw marquee rectangle ---
            if let Some(rect) = self.marquee_rect {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 150, 250)),
                    egui::StrokeKind::Middle
                );
            }
        });
    }
}

fn get_node_rect(ctx: &egui::Context, node: &Node, zoom: f32) -> egui::Rect {
    let font_id = egui::FontId::proportional(14.0 * zoom);
    let bold_font_id = egui::FontId::monospace(14.0 * zoom);
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

            // Find label width for alignment
            let label_width = ctx.fonts_mut(|f| {
                fields.iter().map(|(label, _)| {
                    f.layout_no_wrap(label.to_string(), bold_font_id.clone(), egui::Color32::BLACK)
                        .size().x
                }).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)
            });

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

fn point_line_distance(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> f32 {
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

