use crate::core::pdfparser::Metadata;
use serde::{Serialize, Deserialize};
 use uuid::Uuid;
use crate::app::Annotation;

#[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct Node {
     pub id: Uuid,
     pub title: String,
     pub metadata: Option<Metadata>,
     pub x: f32,
     pub y: f32,
     pub collapsed: bool,
     pub path: Option<String>,
     pub annotations: Vec<Annotation>, // Add this field
 }

 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct Edge {
     pub id: Uuid,
     pub from: Uuid,
     pub to: Uuid,
 }

 #[derive(Debug, Clone, Serialize, Deserialize, Default)]
 pub struct MindMap {
     pub nodes: Vec<Node>,
     pub edges: Vec<Edge>,
 }

 impl MindMap {
     pub fn add_node(&mut self, title: String, x: f32, y: f32) -> Uuid {
         let id = Uuid::new_v4();
         self.nodes.push(Node {
             id,
             title: title.clone(),
             metadata: None,
             x,
             y,
             collapsed: true,
             path: None, 
             annotations: Vec::new()
         });
         id
     }

     pub fn add_pdf_node(&mut self, filename: &str, x: f32, y: f32) -> Result<Uuid, anyhow::Error> {
         let metadata_result = Metadata::from_file(filename);
         if let Ok(metadata) = metadata_result {
             let title = if metadata.title.is_empty() {
                 filename.to_string()
             } else {
                 metadata.title.clone()
             };

             let node = Node {
                 id: Uuid::new_v4(),
                 title,
                 x,
                 y,
                 metadata: Some(metadata),
                 collapsed: true,
                 path: Some(filename.to_string()),
                 annotations: Vec::new()
             };

             let id = node.id;
             self.nodes.push(node);
             Ok(id)
         }
         else { Err(anyhow::anyhow!("Error running pdfinfo")) }
     }

     pub fn add_edge(&mut self, from: Uuid, to: Uuid) -> Uuid {
         let id = Uuid::new_v4();
         self.edges.push(Edge { id, from, to });
         id
     }

     pub fn remove_node(&mut self, node_id: Uuid) {
         self.nodes.retain(|n| n.id != node_id);
         self.edges.retain(|e| e.from != node_id && e.to != node_id);
     }
 }
