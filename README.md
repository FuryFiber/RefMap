# RefMap User Manual

## Table of Contents
1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Creating a New Mind Map](#creating-a-new-mind-map)
4. [Adding Nodes](#adding-nodes)
5. [Connecting Nodes](#connecting-nodes)
6. [Editing Nodes](#editing-nodes)
7. [Annotations](#annotations)
8. [Managing the Mind Map](#managing-the-mind-map)
9. [Zooming and Panning](#zooming-and-panning)
10. [Saving and Loading](#saving-and-loading)
11. [Keyboard Shortcuts](#keyboard-shortcuts)
12. [Advanced Features](#advanced-features)
13. [Troubleshooting](#troubleshooting)

---

## Introduction
RefMap is a mind mapping application designed for organizing research papers. It has support for extracting metadata and annotation capabilities. 
It allows users to create a visual overview of the documentation that was gathered for a research project, link related concepts, and add detailed notes to each node.

---

## Getting Started
1. **Installation**: build from source using `cargo build --release` or run the install script `install.sh` which will build the application and create a `.desktop` entry so RefMap is visible in you launcher.
2. **Launch**: Run the executable (`refmap` or `refmap.exe`) to start the application.

---

## Creating a New Mind Map
- **Menu**: `File → New` to start a new project.
- **Initial State**: The canvas is empty. Double click anywhere to add your first node.

---

## Adding Nodes
### Basic Node
- **Double-click** on the canvas to create a new basic node.
- **Double-click** on a node to open its associated PDF file (if any) in your default pdf viewer.

### PDF Node
- **Shift + Left-click** to open the file picker and add a PDF node. The metadata from the PDF will be automatically populated.

---

## Connecting Nodes
- **Right-click and drag** or **Hold CTRL and drag Left-click** from one node to another to create a connection.
- Release the mouse button on the target node to complete the connection. At this point a menu will appear to select the edge type

---

## Editing Nodes
- **Right-click** on a node to open the context menu:
    - **Edit Metadata**: Modify the node's title, authors, keywords, and date.
    - **View Annotations**: Open the annotations panel for this node.
    - **Add Annotation**: Create a new annotation for this node.
    - **Delete Node**: Remove the selected node.

---
## Editing Edges
- **Right-click** on an edge to open the context menu:
  - **Change Edge Type**: Modify the edge's type.
  - **Add Annotation**: Create a new annotation for this edge.
  - **View Annotations**: Open the annotations panel for this edge.
  - **Delete Node**: Remove the selected edge.
---

## Annotations
Annotations are color-coded notes attached to specific nodes. Each annotation type has a distinct background color:

| Type      | Color       | Icon |
|-----------|-------------|------|
| Text Note | Light Gray  | 📝   |
| Question  | Light Blue  | ❓    |
| Summary   | Light Green | 📚   |
| Quote     | Yellow      | 📝   |
| Todo      | Light Red   | ✅    |
| Idea      | Peach       | 💡   |
| Warning   | Light Red   | ⚠️   |

### Adding Annotations
1. **Right-click** a node → **Add Annotation**.
2. Fill in the title, content, and select a type.
3. Click **Add** to save.

### Editing Annotations
1. Click the **✏** icon in an annotation card.
2. Modify fields and click **Update**.

### Viewing Annotations
- **Right-click** a node → **View Annotations** to open the annotations panel.

---

## Managing the Mind Map
### Selecting Nodes
- **Single Select**: Left-click a node.
- **Multiple Select**:
    - **Marquee Select**: Click and drag to select multiple nodes within a rectangle.
    - **Shift + Click**: Add node or edge to selection

### Moving Nodes
- **Left-click and drag** a selected node to reposition it.

### Deleting Nodes
- **Select nodes** → Press `Delete` key.
- **Right-click** a node → **Delete Node**.

---

## Zooming and Panning
| Mouse               | Trackpad                    | Action          |
|---------------------|-----------------------------|-----------------|
| `Scroll`            | `CTRL + Scroll`             | zoom in and out |
| `Middle click drag` | `CTRL + Primary click drag` | pan             |


---

## Saving and Loading
### Save
- **Ctrl + S**: Save the current map. If no project is set, a dialog will prompt for a location.
- **File → Save As...**: Choose a location to save the project file.

### Load
- **File → Open...**: Select an existing project to open.

### Export
- **File → Export**: Export the current project as a zip archive containing the map data and associated PDFs.

---

## Keyboard Shortcuts
| Shortcut   | Action                |
|------------|-----------------------|
| `Ctrl + S` | Save file             |
| `Delete`   | Delete selected items |
| `Escape`   | Deselect all          |

---

## Advanced Features
### Collapsing Nodes
- **Ctrl + Click** a node to toggle its collapsed state. This hides metadata to show only the title.

### Annotation Panel
- Opens with **View Annotations** from the context menu. Displays all annotations for the selected node with options to edit or delete.

---

## Troubleshooting
### PDF File Not Opening
- Ensure the file path is valid and the PDF is not corrupted.
- RefMap uses the system's default PDF viewer. If it fails, check your system associations.

### Application Crashes on Save
- Verify you have write permissions to the target directory.
- Try saving to a different location.

### Missing Metadata
- PDF metadata extraction requires valid XMP data. Some PDFs may not contain extractable metadata. In this case, manually edit the node to add information.
